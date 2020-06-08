#![no_std]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![allow(dead_code)]
#![feature(core_intrinsics)]
#![feature(thread_local)]
#![feature(wake_trait)]
#![deny(warnings)]

#[macro_use]
extern crate alloc;
extern crate multiboot2;
// #[macro_use]
extern crate system;


use core::panic::PanicInfo;

#[cfg(feature = "efi")]
use system::KernelArgs;
use system::SystemInformation;
#[cfg(feature = "efi")]
use uefi::table::boot::{AllocateType, MemoryMapIter, MemoryMapKey, MemoryType};

use crate::async_process::Executor;
use crate::devices::keyboard::print_scan_code;
use crate::devices::vga::clear_screen;
use crate::initializer::Initializer;
use crate::utils::loop_hlt;

#[macro_use]
mod macros;
mod serial;
mod memory;
mod descriptor;
mod initializer;
mod utils;
mod context_switch;
mod process;
mod async_process;
mod devices;
mod interrupt;
mod syscall;
mod tests;


#[cfg(feature = "efi")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("uefi entry");
    let info = SystemInformation::new(info_addr);
    Initializer::new(info).initialize();
    loop_hlt()
}


#[cfg(feature = "mutiboot")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("entry kernel");
    clear_screen();
    let info = SystemInformation::new(info_addr);
    Initializer::new(info).initialize();
    let mut executor = Executor::new();
    executor.spawn(print_scan_code());
    executor.run();
    loop_hlt()
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop_hlt()
}



