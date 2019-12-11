#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga;
//pub mod protect;
pub mod idt;
pub mod gdt;


pub fn init_descriptor() {
    idt::init();
    gdt::init();

    unsafe {
        // 初始化PIC 8259
        idt::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

/// 用于测试过程中异常处理
#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    serial_println!("[Failed]\n");
    serial_println!("Error: {}",info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests",tests.len());
    for test in tests {
        test();
    }
}

/// cargo xtest 入口
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_descriptor();
//    test_main();
    loop {}
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum QemuExitCode {
    // 使用0状态码将会出现一些问题，例如 (0 << 1) | 1的结果为1 表示失败
    Succeed = 0x10,
    Failed = 0x11,
}

// 用于退出qemu模拟器
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        //在0xf4创建一个新端口，该端口是isa-debug-exit设备的iobase。
        let mut port = Port::new(0xf4);
        // 向该端口写入成功状态码
        // isa-debug-exit 必须为 4 bytes 因此使用u32
        port.write(exit_code as u32);
    }
}

pub fn loop_hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}