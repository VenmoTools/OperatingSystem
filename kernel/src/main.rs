#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(feature = "deny-warnings", deny(missing_docs))]


#[macro_use]
extern crate kernel;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

use uefi::prelude::SystemTable;
use uefi::table::boot::MemoryMapIter;
use uefi::table::Runtime;

#[allow(unused_imports)]
use kernel::{devices, Initializer, loop_hlt};

struct Allocate;

unsafe impl GlobalAlloc for Allocate {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!()
    }
}

#[global_allocator]
static ALLOCATOR: Allocate = Allocate;

#[alloc_error_handler]
fn handler(_layout: Layout) -> ! {
    loop_hlt()
}


#[no_mangle]
pub extern "C" fn _start(_st: SystemTable<Runtime>, _it: MemoryMapIter) -> ! {
    println!("Hello World");
    let cpuid = raw_cpuid::CpuId::new();
    println!("cpu info:{:?}", cpuid.get_vendor_info().unwrap().as_string());
//    Initializer::initialize_all();
//    use x86_64::registers::control::Cr3;
//    let (addr, _) = Cr3::read();
//    println!("{:?}", addr);
    loop_hlt()
}

/// 用于运行过程中异常处理
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Error: {}", info);
    loop_hlt()
}

