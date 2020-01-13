#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

#[cfg(target_arch = "x86_64")]
pub(crate) use core::arch::x86_64 as arch;
use core::panic::PanicInfo;

pub mod serial;


pub mod descriptor;
pub mod process;
pub mod devices;


pub struct Initializer;

impl Initializer {
    #[cfg(feature = "x86")]
    pub fn initialize_all() {
        // 初始化gdt
        descriptor::init_gdt();
        // 初始化idt
        descriptor::init_idt();
        // 初始化pics
        descriptor::init_pics();

        // 初始化内存管理

        // 开启分页

        // 开启中断
        system::ia_32e::instructions::interrupt::enable();
    }
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
        system::ia_32e::instructions::interrupt::hlt();
    }
}