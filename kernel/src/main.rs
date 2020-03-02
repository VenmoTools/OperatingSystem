#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]


#[macro_use]
extern crate kernel;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
#[cfg(feature = "efi")]
use kernel::{devices, Initializer, loop_hlt};
use uefi::prelude::SystemTable;
use uefi::Status;
use uefi::table::boot::MemoryMapIter;
use uefi::table::Runtime;
use uefi::table::runtime::ResetType;

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

#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(st: SystemTable<Runtime>, _iter: MemoryMapIter) -> ! {
    st.config_table();
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

/// 用于运行过程中异常处理
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Error: {}", info);
    loop_hlt()
}

