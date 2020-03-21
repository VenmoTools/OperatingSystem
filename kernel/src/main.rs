#![feature(asm)]
#![no_std]
#![no_main]
#![allow(dead_code)]
#![deny(warnings)]
#[macro_use]
extern crate alloc;
#[cfg(feature = "bios")]
#[macro_use]
extern crate kernel;
#[cfg(feature = "efi")]
#[macro_use]
extern crate kernel;

use system::KernelArgs;
use uefi::table::{Runtime, SystemTable};
use uefi::table::boot::MemoryMapIter;

use kernel::{Initializer, loop_hlt};


fn get_mem_iter(args: &KernelArgs) -> &mut MemoryMapIter {
    unsafe { &mut *(args.iter as *mut MemoryMapIter) }
}

fn get_system_table(args: &KernelArgs) -> &SystemTable<Runtime> {
    unsafe { &*(args.st as *const SystemTable<Runtime>) }
}

#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(args: u64) -> ! {
    println!("ptr:{:?}", args);
    //BUG: 不知为何传递的指针参数会偏移
    let args = unsafe { &*((132806768) as *const KernelArgs) };
    let iter = get_mem_iter(args);
    let initializer = Initializer::new(args, iter);
    initializer.initialize_all();
    let s = vec![0; 20];
    println!("{:?}", s);
    kernel_start()
}


fn kernel_start() -> ! {
    loop_hlt()
}

#[cfg(feature = "bios")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    let cpuid = raw_cpuid::CpuId::new();
    println!("cpu info:{:?}", cpuid.get_vendor_info().unwrap().as_string());
    Initializer::initialize_all();
    loop_hlt()
}


