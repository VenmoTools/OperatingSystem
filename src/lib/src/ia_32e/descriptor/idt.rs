// Copyright 2017 Philipp Oppermann. See the README.md
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::ia_32e::PrivilegedLevel;
use crate::ia_32e::addr::VirtAddr;
use crate::bits::BitOpt;
use bitflags::bitflags;

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, Index, IndexMut};


#[derive(Clone)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    pub instruction_pointer: VirtAddr,
    pub code_segment: u64,
    pub eflags: u64,
    pub stack_pointer: VirtAddr,
    pub stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }

        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &Hex(self.eflags));
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}
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

pub type HandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame);
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(&mut InterruptStackFrame,code: u64);
pub type PageFaultHandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame, code: PageFaultErrorCode);


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntryOptions(u16);

impl EntryOptions {
    const fn minimal() -> Self {
        EntryOptions(0b1110_0000_0000)
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: PrivilegedLevel) -> &mut Self {
        self.0.set_bits(13..15, dpl as u16);
        self
    }

    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index + 1);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Entry<F> {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<F> Entry<F> {
    pub const fn missing() -> Self {
        Entry {
            pointer_low: 0,
            gdt_selector: 0,
            options: EntryOptions::minimal(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
            phantom: PhantomData,
        }
    }

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
impl Entry<HandlerFunc>{
    pub fn set_handler_fn(&mut self, handler:HandlerFunc) -> &mut EntryOptions{
        self.set_handler_addr(handler as u64)
    }
}

#[cfg(target_arch = "x86_64")]
impl Entry<HandlerFuncWithErrCode>{
    pub fn set_handler_fn(&mut self, handler:HandlerFuncWithErrCode) -> &mut EntryOptions{
        self.set_handler_addr(handler as u64)
    }
}
#[cfg(target_arch = "x86_64")]
impl Entry<PageFaultHandlerFunc>{
    pub fn set_handler_fn(&mut self, handler:PageFaultHandlerFunc) -> &mut EntryOptions{
        self.set_handler_addr(handler as u64)
    }
}

//macro_rules! impl_set_handler_func {
//    ( $h:ty ) => {
//        #[cfg(target_arch = "x86_64")]
//        impl Entry<$h>{
//            pub fn set_handler_fn(&mut self, handler:$h) -> &mut EntryOptions{
//                self.set_handler_addr(handler as u64)
//            }
//        }
//    };
//}
//
//impl_set_handler_func!(HandlerFunc);
//impl_set_handler_func!(HandlerFuncWithErrCode);
//impl_set_handler_func!(PageFaultHandlerFunc);


#[allow(missing_debug_implementations)]
#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFunc>,

    pub debug: Entry<HandlerFunc>,

    pub non_maskable_interrupt: Entry<HandlerFunc>,

    pub breakpoint: Entry<HandlerFunc>,

    pub overflow: Entry<HandlerFunc>,

    pub bound_range_exceeded: Entry<HandlerFunc>,

    pub invalid_opcode: Entry<HandlerFunc>,

    pub device_not_available: Entry<HandlerFunc>,

    pub double_fault: Entry<HandlerFuncWithErrCode>,

    coprocessor_segment_overrun: Entry<HandlerFunc>,

    pub invalid_tss: Entry<HandlerFuncWithErrCode>,

    pub segment_not_present: Entry<HandlerFuncWithErrCode>,

    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,

    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,

    pub page_fault: Entry<PageFaultHandlerFunc>,

    reserved_1: Entry<HandlerFunc>,

    pub x87_floating_point: Entry<HandlerFunc>,

    pub alignment_check: Entry<HandlerFuncWithErrCode>,

    pub machine_check: Entry<HandlerFunc>,

    pub simd_floating_point: Entry<HandlerFunc>,

    pub virtualization: Entry<HandlerFunc>,

    reserved_2: [Entry<HandlerFunc>; 9],

    pub security_exception: Entry<HandlerFuncWithErrCode>,

    reserved_3: Entry<HandlerFunc>,

    interrupts: [Entry<HandlerFunc>; 256 - 32],

}

impl InterruptDescriptorTable {
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
        self.virtualization = Entry::missing();
        self.reserved_2 = [Entry::missing(); 9];
        self.security_exception = Entry::missing();
        self.reserved_3 = Entry::missing();
        self.interrupts = [Entry::missing(); 256 - 32];
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load(&'static self) {
        use crate::ia_32e::instructions::tables::lidt;
        use crate::ia_32e::descriptor::DescriptorTablePointer;
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
            i @ 15 | i @ 31 | i @ 21..=29 => panic!("entry {} is reserved", i),
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
            i @ 15 | i @ 31 | i @ 21..=29 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i => panic!("no entry with index {}", i),
        }
    }
}
