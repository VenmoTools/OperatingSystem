#![no_std]
#![feature(exclusive_range_pattern)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

#[cfg(feature = "efi")]
pub mod serial;
#[cfg(feature = "efi")]
pub mod descriptor;
pub mod process;
#[cfg(feature = "efi")]
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