#![no_std]
#![cfg_attr(test, no_main)]
#![feature(exclusive_range_pattern)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

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


pub fn loop_hlt() -> ! {
    loop {
        system::ia_32e::instructions::interrupt::hlt();
    }
}