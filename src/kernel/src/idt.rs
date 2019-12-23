use pic8259_simple::ChainedPics;
/// 当一个异常发生后，CPU大概会做一下操作
/// 1. 将某个寄存器的值压入栈中，其中包含指令寄存器和RFLAGS寄存器
/// 2. 从IDT中读取响应的条目，例如当发生了段错误后CPU会读取第13号异常
/// 3. 检查对应条目是否存在，如果没有则再次抛出异常（Double fault）
/// 4. 如果是中断门则关闭硬件中断
/// 5. 从GDT中指定特定的选择子加载到CS寄存器中
/// 6. 跳转到对应的处理函数

/// Preserved 和 Scratch 寄存器
/// 调用将寄存器分为两个部分，Preserved寄存器和Scratch寄存器
///
/// 在Preserved寄存器中的值在函数调用的过程中必须保持不变，被调用方在恢复原始值时才可以改变这些寄存器
/// 因此被称为“被调用者保存”
/// 常见模式为在函数开始时将这些寄存器保存在堆栈中，并在函数返回前恢复
///
/// Scratch寄存器可以允许调用不加限制的更改其中的值，如果调用方想在函数调用时保留暂存寄存器的值，则它需要在函数调用之前备份和还原它 Scratch寄存器时调用者保存的
/// 常见模式为在函数调用前将这些寄存器保存在堆栈中，函数结束后再将其恢复
///
/// 在x86_64位系统中
///     Preserved寄存器（被调用者保存）：rbp, rbx, rsp, r12, r13, r14, r15
///     Scratch 寄存器 （调用者保存） ：rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11
use system::ia_32e::descriptor::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use system::mutex::Mutex;

//use system::ia_32e::cpu::pic::ChainedPics;
use lazy_static::lazy_static;

use crate::{loop_hlt, print, println};
use crate::gdt;

pub const PIC_A_OFFSET: u8 = 32;
pub const PIC_B_OFFSET: u8 = PIC_A_OFFSET + 8;

// 使用Mutex可以安全操作内部可变的数据
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_A_OFFSET, PIC_B_OFFSET) });



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
        idt.divide_by_zero.set_handler_fn(divide_by_zero);
        idt.invalid_tss.set_handler_fn(invalid_tss);
        idt.security_exception.set_handler_fn(security_exception);
        idt.segment_not_present.set_handler_fn(segment_not_present);
        idt.alignment_check.set_handler_fn(alignment_check);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded);
        idt.device_not_available.set_handler_fn(device_not_available);
        idt.general_protection_fault.set_handler_fn(general_protection_fault);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.machine_check.set_handler_fn(machine_check);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt);
        idt.virtualization.set_handler_fn(virtualization);
        idt.x87_floating_point.set_handler_fn(x87_floating_point);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
        idt.simd_floating_point.set_handler_fn(simd_floating_point);
        idt.overflow.set_handler_fn(overflow);
        idt.debug.set_handler_fn(debug);
        // 在IDT中为双重错误处理程序设置堆栈索引
        unsafe {
            idt.double_fault.set_handler_fn(double_fault).set_stack_index(gdt::DOUBLE_FAULT_LIST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt);
        idt[InterruptIndex::KeyBoard.as_usize()].set_handler_fn(keyboard_interrupt);
        idt
    };
}

pub fn init() {
    IDT.load();
}

// 调试中断
extern "x86-interrupt" fn breakpoint(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:BREAKPOINT\n{:#?}", stackframe);
}

// 缺页错误
extern "x86-interrupt" fn page_fault(stackframe: &mut InterruptStackFrame, code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION:PageFault");
    println!("Access Address:{:?}", Cr2::read());
    println!("Error Code:{:?}", code);
    println!("{:#?}", stackframe);
    loop_hlt();
}

/// 在处理先前（第一个）异常处理程序期间发生第二个异常时，可能会发生双重错误异常
/// 只有异常的非常特殊的组合才会导致双重错误
/// +--------------------------+------------------------+
/// | First Exception          |Second Exception        |
/// +--------------------------+------------------------+
/// |Divide-by-zero,           |Invalid TSS,            |
/// |Invalid TSS,              |Segment Not Present,    |
/// |Segment Not Present,      |Stack-Segment Fault,    |
/// |Stack-Segment Fault,      |General Protection Fault|
/// |General Protection Fault  |                        |
/// +--------------------------+------------------------+
/// |                          |Page Fault,             |
/// |                          |Invalid TSS,            |
/// |Page Fault                |Segment Not Present,    |
/// |                          |Stack-Segment Fault,    |
/// |                          |General Protection Fault|
/// +--------------------------+------------------------+
///
///
extern "x86-interrupt" fn double_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:DOUBLE FAULT\n{:#?}", stackframe);
}

/// 时钟中断
extern "x86-interrupt" fn timer_interrupt(_stackframe: &mut InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// 键盘中断
extern "x86-interrupt" fn keyboard_interrupt(_stackframe: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{Keyboard, layouts, ScancodeSet1, DecodedKey};
    use spin::Mutex;


    lazy_static! {
        static ref KEYBOARD:Mutex<Keyboard<layouts::Us104Key,ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key,ScancodeSet1));
    }


    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scan_code: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::KeyBoard.as_u8());
    }
}


extern "x86-interrupt" fn divide_by_zero(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:divide_by_zero\n{:#?}", stackframe);
}

extern "x86-interrupt" fn non_maskable_interrupt(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:non_maskable_interrupt\n{:#?}", stackframe);
}

extern "x86-interrupt" fn debug(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:debug\n{:#?}", stackframe);
}

extern "x86-interrupt" fn overflow(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:overflow\n{:#?}", stackframe);
}

extern "x86-interrupt" fn bound_range_exceeded(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:bound_range_exceeded\n{:#?}", stackframe);
}

extern "x86-interrupt" fn invalid_opcode(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:invalid_opcode\n{:#?}", stackframe);
}

extern "x86-interrupt" fn device_not_available(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:device_not_available\n{:#?}", stackframe);
}

extern "x86-interrupt" fn x87_floating_point(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:x87_floating_point\n{:#?}", stackframe);
}

extern "x86-interrupt" fn machine_check(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:machine_check\n{:#?}", stackframe);
}

extern "x86-interrupt" fn simd_floating_point(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:simd_floating_point\n{:#?}", stackframe);
}

extern "x86-interrupt" fn virtualization(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:virtualization\n{:#?}", stackframe);
}

extern "x86-interrupt" fn invalid_tss(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:invalid_tss\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

extern "x86-interrupt" fn segment_not_present(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:segment_not_present\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

extern "x86-interrupt" fn stack_segment_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:stack_segment_fault\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

extern "x86-interrupt" fn general_protection_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:general_protection_fault\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

extern "x86-interrupt" fn alignment_check(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:alignment_check\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

extern "x86-interrupt" fn security_exception(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:security_exception\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

// 中断索引表
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum InterruptIndex {
    // 时钟中断
    Timer = PIC_A_OFFSET,
    // 键盘中断
    KeyBoard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
