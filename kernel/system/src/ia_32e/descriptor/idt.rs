// Copyright 2017 Philipp Oppermann. See the README.md
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, Index, IndexMut};

use crate::bits::{BitOpt, PageFaultErrorCode};
use crate::ia_32e::PrivilegedLevel;
use crate::ia_32e::VirtAddr;
use crate::ia_32e::descriptor::DescriptorTablePointer;

/// 中断栈帧的值
#[derive(Clone)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    /// RIP寄存器值
    pub instruction_pointer: VirtAddr,
    /// CS寄存器值
    pub code_segment: u64,
    /// 在调用处理器程序前rflags寄存器的值
    pub rflags: u64,
    /// 中断时的堆栈指针。
    pub stack_pointer: VirtAddr,
    /// 中断时的堆栈段描述符（在64位模式下通常为零）
    pub stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &self.rflags);
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}


/// 中断栈帧，对InterruptStackFrameValue的封装
#[repr(C)]
pub struct InterruptStackFrame {
    value: InterruptStackFrameValue,
}

impl InterruptStackFrame {
    pub unsafe fn as_mut(&mut self) -> &mut InterruptStackFrameValue {
        &mut self.value
    }
}

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

/// 异常处理函数（无返回码）
pub type HandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame);
/// 异常处理函数（含返回码）
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(&mut InterruptStackFrame, code: u64);

/// `#PF`异常处理函数（含返回码）
pub type PageFaultHandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame, code: PageFaultErrorCode);

/// IDT段属性
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntryOptions(u16);

/// IDT 属性
/// 原版
/// |  63  -  48   |47|46-45|44|43-40|39-37|36|35|34-32|31  - 16| 15   -   0  |
/// +--------------+--+-----+--+-----+-----+--+--+-----+--------+-------------+
/// | SgemntOffset |P | DPL |0 | Type|  0  |0 |0 | IST |Selecotr|SegmentOffset|
/// +--------------+--+-----+--+-----+-----+--+--+-----+--------+-------------+
/// 简短版：
/// |15|14-13|12|11|10|9|8|7 - 5|4|3|2 - 0|
/// +--+-----+--+--+--+-+-+-----+-+-+-----+
/// |P | DPL |0 |1 |1 |1|1|  0  |0|0| IST |
/// +--+-----+--+--+--+-+-+-----+-+-+-----+
impl EntryOptions {
    /// 创建一个最小选项字段并设置所有必须为1的位
    const fn minimal() -> Self {
        EntryOptions(0b1110_0000_0000)
    }
    /// 用于设置第47位(表示`已存在`)
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }
    /// 用于设置第40位(用于置1表示陷进门,指令表示中断门),所以我们需要使用取反布尔值来完成此操作
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    /// 用于设置第46-45位(注意不包含15实际范围是13-14),用于设置特权级
    pub fn set_privilege_level(&mut self, dpl: PrivilegedLevel) -> &mut Self {
        self.0.set_bits(13..15, dpl as u16);
        self
    }
    /// 设置第34-32位(IST) 用于异常处理的堆栈索引
    /// 如果index的范围不再0-7之间将会panic
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index + 1);
        self
    }
}

/// 中断门结构
/// |   127              -              96             |   95     -         64|
/// +--------------------------------------------------+----------------------+
/// |                    reserved                      |     Segment Offset   |
/// +--------------------------------------------------+----------------------+
///
/// |  63  -  48   |47|46-45|44|43-40|39-37|36|35|34-32|31  - 16| 15   -   0  |
/// +--------------+--+-----+--+-----+-----+--+--+-----+--------+-------------+
/// | SgemntOffset |P | DPL |0 | Type|  0  |0 |0 | IST |Selecotr|SegmentOffset|
/// +--------------+--+-----+--+-----+-----+--+--+-----+--------+-------------+
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Entry<F> {
    /// 段基址（低） 0-16位
    pointer_low: u16,
    /// GDT选择子 16-31位
    gdt_selector: u16,
    /// 中断门属性 32-47位
    options: EntryOptions,
    /// 段基址（中） 48-63位
    pointer_middle: u16,
    /// 段基址（高） 64-95位
    pointer_high: u32,
    /// 保留位 96-127位
    reserved: u32,
    /// 调用函数 PhantomData不占用空间
    handler_func: PhantomData<F>,
}

impl<F> Entry<F> {
    /// 中断门初始化
    pub const fn missing() -> Self {
        Entry {
            pointer_low: 0,
            gdt_selector: 0,
            options: EntryOptions::minimal(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
            handler_func: PhantomData,
        }
    }

    /// 用于注册异常处理函数
    #[cfg(target_arch = "x86_64")]
    fn set_handler_addr(&mut self, addr: u64) -> &mut EntryOptions {
        use crate::ia_32e::instructions::segmention::cs;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;
        self.gdt_selector = cs().0;
        self.options.set_present(true);
        &mut self.options
    }
}


#[cfg(target_arch = "x86_64")]
impl Entry<HandlerFunc> {
    /// 用于注册异常处理函数，无错误码
    pub fn set_handler_fn(&mut self, handler: HandlerFunc) -> &mut EntryOptions {
        self.set_handler_addr(handler as u64)
    }
}

#[cfg(target_arch = "x86_64")]
impl Entry<HandlerFuncWithErrCode> {
    /// 用于注册异常处理函数，含错误码
    pub fn set_handler_fn(&mut self, handler: HandlerFuncWithErrCode) -> &mut EntryOptions {
        self.set_handler_addr(handler as u64)
    }
}

#[cfg(target_arch = "x86_64")]
impl Entry<PageFaultHandlerFunc> {
    /// 用于注册异常处理函数，含页异常错误码
    pub fn set_handler_fn(&mut self, handler: PageFaultHandlerFunc) -> &mut EntryOptions {
        self.set_handler_addr(handler as u64)
    }
}

/// 中断描述符表
#[allow(missing_debug_implementations)]
#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    /// #DE
    pub divide_by_zero: Entry<HandlerFunc>,
    /// #DB
    pub debug: Entry<HandlerFunc>,
    /// NMI 中断
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    /// #BP
    pub breakpoint: Entry<HandlerFunc>,
    /// #OF
    pub overflow: Entry<HandlerFunc>,
    /// #BR
    pub bound_range_exceeded: Entry<HandlerFunc>,
    /// #UD
    pub invalid_opcode: Entry<HandlerFunc>,
    /// #NM
    pub device_not_available: Entry<HandlerFunc>,
    /// #DF
    pub double_fault: Entry<HandlerFuncWithErrCode>,
    /// 协处理器段溢出
    coprocessor_segment_overrun: Entry<HandlerFunc>,
    /// #TS
    pub invalid_tss: Entry<HandlerFuncWithErrCode>,
    /// #NP
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,
    /// #SS
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    /// #GP
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
    /// #PF
    pub page_fault: Entry<PageFaultHandlerFunc>,
    /// 保留
    reserved_1: Entry<HandlerFunc>,
    /// #MF
    pub x87_floating_point: Entry<HandlerFunc>,
    /// #AC
    pub alignment_check: Entry<HandlerFuncWithErrCode>,
    /// #MC
    pub machine_check: Entry<HandlerFunc>,
    /// #XM
    pub simd_floating_point: Entry<HandlerFunc>,
    /// #VE
    pub virtualization: Entry<HandlerFunc>,
    /// 22-31 Intel保留使用
    reserved_2: [Entry<HandlerFunc>; 9],
    pub security_exception: Entry<HandlerFuncWithErrCode>,
    reserved_3: Entry<HandlerFunc>,
    /// 用户自定义中断
    interrupts: [Entry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    /// 创建初始化的中断描述符表
    pub const fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            divide_by_zero: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_1: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            reserved_2: [Entry::missing(); 9],
            security_exception: Entry::missing(),
            reserved_3: Entry::missing(),
            interrupts: [Entry::missing(); 256 - 32],
        }
    }
    /// 重置
    pub fn reset(&mut self) {
        self.divide_by_zero = Entry::missing();
        self.debug = Entry::missing();
        self.non_maskable_interrupt = Entry::missing();
        self.breakpoint = Entry::missing();
        self.overflow = Entry::missing();
        self.bound_range_exceeded = Entry::missing();
        self.invalid_opcode = Entry::missing();
        self.device_not_available = Entry::missing();
        self.double_fault = Entry::missing();
        self.coprocessor_segment_overrun = Entry::missing();
        self.invalid_tss = Entry::missing();
        self.segment_not_present = Entry::missing();
        self.stack_segment_fault = Entry::missing();
        self.general_protection_fault = Entry::missing();
        self.page_fault = Entry::missing();
        self.reserved_1 = Entry::missing();
        self.x87_floating_point = Entry::missing();
        self.alignment_check = Entry::missing();
        self.machine_check = Entry::missing();
        self.simd_floating_point = Entry::missing();
        self.reserved_2 = [Entry::missing(); 9];
        self.security_exception = Entry::missing();
        self.reserved_3 = Entry::missing();
        self.interrupts = [Entry::missing(); 256 - 32];
    }
    /// 使用lidt指令加载IDT
    #[cfg(target_arch = "x86_64")]
    pub fn load(&'static self) {
        use crate::ia_32e::instructions::tables::lidt;
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe {
            lidt(&ptr);
        }
    }
}


impl Index<usize> for InterruptDescriptorTable {
    type Output = Entry<HandlerFunc>;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.divide_by_zero,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            9 => &self.coprocessor_segment_overrun,
            16 => &self.x87_floating_point,
            18 => &self.machine_check,
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            i @ 32..=255 => &self.interrupts[i - 32],
            i @ 15 | i @ 31 | i @ 22..=29 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i => panic!("no entry with index {}", i),
        }
    }
}

impl IndexMut<usize> for InterruptDescriptorTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.divide_by_zero,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            9 => &mut self.coprocessor_segment_overrun,
            16 => &mut self.x87_floating_point,
            18 => &mut self.machine_check,
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            i @ 32..=255 => &mut self.interrupts[i - 32],
            i @ 15 | i @ 31 | i @ 22..=29 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i => panic!("no entry with index {}", i),
        }
    }
}

