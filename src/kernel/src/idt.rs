use pic8259_simple::ChainedPics;
use spin;
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

use lazy_static::lazy_static;

use crate::{loop_hlt, print, println};
use crate::gdt;

pub const PIC_A_OFFSET: u8 = 32;
pub const PIC_B_OFFSET: u8 = PIC_A_OFFSET + 8;

// 使用Mutex可以安全操作内部可变的数据
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_A_OFFSET, PIC_B_OFFSET) });



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
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
