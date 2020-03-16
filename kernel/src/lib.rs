#![no_std]
#![feature(exclusive_range_pattern)]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
// #![deny(warnings)]

#[macro_use]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

use system::KernelArgs;
use uefi::table::boot::MemoryMapIter;

#[cfg(feature = "bios")]
pub mod bios;
#[cfg(feature = "efi")]
pub mod efi;

#[cfg(feature = "efi")]
pub mod graphic;
pub mod apic;
pub mod process;
pub mod paging;
#[macro_use]
pub mod serial;
pub mod memory;


pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!()
    }
}

#[global_allocator]
pub static mut ALLOCATOR: Allocator = Allocator;

#[alloc_error_handler]
fn handler(layout: Layout) -> ! {
    println!("allocate memory Error: align={} ,size={}", layout.align(), layout.size());
    loop_hlt()
}


pub fn init_heap() {
    let heap_start = 0x0000;
    let heap_end = 0x0000;
    let _size = heap_end - heap_start;
}


pub struct Initializer<'a> {
    #[cfg(feature = "efi")]
    args: &'a KernelArgs,
    #[cfg(feature = "efi")]
    iter: &'a MemoryMapIter<'a>,
}

impl Initializer {
    #[cfg(feature = "bios")]
    pub fn initialize_all() {
        // 初始化gdt
        bios::descriptor::init_gdt();
        // 初始化idt
        bios::descriptor::init_idt();
        // 初始化pics
        bios::descriptor::init_pics();

        // 初始化内存管理

        // 开启分页

        // 开启中断
        system::ia_32e::instructions::interrupt::enable();
    }
    #[cfg(feature = "efi")]
    pub fn initialize_all() {
        //todo:
    }
}


pub fn loop_hlt() -> ! {
    loop {
        system::ia_32e::instructions::interrupt::hlt();
    }
}

///////////////////////
///// Panic Handler
///////////////////////
/// 用于运行过程中异常处理
#[cfg(feature = "efi")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop_hlt()
}

#[cfg(not(feature = "efi"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Error: {}", info);
    loop_hlt()
}