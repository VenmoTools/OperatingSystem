#![no_std]
#![allow(dead_code)]
#![feature(exclusive_range_pattern)]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![deny(warnings)]
#![feature(naked_functions)]

#[macro_use]
pub extern crate alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;

use buddy_system_allocator::LockedHeap;
use system::ia_32e::instructions::interrupt::enable;
use system::KernelArgs;
use uefi::table::boot::MemoryMapIter;

#[cfg(feature = "bios")]
pub mod bios;
#[cfg(feature = "efi")]
pub mod efis;

#[cfg(feature = "efi")]
pub mod graphic;
pub mod process;
pub mod paging;
#[macro_use]
pub mod serial;
pub mod memory;
pub mod apic;

#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();


#[alloc_error_handler]
fn handler(layout: Layout) -> ! {
    println!("allocate memory Error: align={} ,size={}", layout.align(), layout.size());
    loop_hlt()
}


pub fn init_heap(heap_start: usize, heap_end: usize) {
    let size = heap_end - heap_start;
    unsafe {
        HEAP.lock().init(heap_start, size)
    }
}


pub struct Initializer<'a> {
    #[cfg(feature = "efi")]
    args: &'a KernelArgs,
    #[cfg(feature = "efi")]
    iter: &'a MemoryMapIter<'a>,
}

#[cfg(feature = "bios")]
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
}

#[cfg(feature = "efi")]
impl<'a> Initializer<'a> {
    #[cfg(feature = "efi")]
    pub fn new(args: &'a KernelArgs, iter: &'a MemoryMapIter<'a>) -> Self {
        Self {
            args,
            iter,
        }
    }
    #[cfg(feature = "efi")]
    pub fn initialize_all(&self) {
        // BUG! gdt
        // efis::descriptor::init_gdt_and_tss();
        // 初始化idt
        efis::descriptor::init_idt();
        // 初始化内存
        init_heap(0x1400000, 0x3F36000);
        enable();
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