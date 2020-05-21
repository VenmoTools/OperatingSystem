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
use system::ia_32e::instructions::interrupt::{enable_interrupt_and_hlt, enable_interrupt_and_nop};
use system::ia_32e::instructions::segmention::cs;
#[cfg(feature = "efi")]
use system::KernelArgs;
use system::SystemInformation;
#[cfg(feature = "efi")]
use uefi::table::boot::{AllocateType, MemoryMapIter, MemoryMapKey, MemoryType};

use crate::initializer::Initializer;
use crate::process::scheduler::switch;
use crate::utils::loop_hlt;

#[macro_use]
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

#[cfg(feature = "efi")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("uefi entry");
    // 指针地址会偏移？？
    let addr = info_addr - 64;
    let info = SystemInformation::new(addr);
    Initializer::new(info).initialize();
    loop_hlt()
}


#[cfg(feature = "mutiboot")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("entry kernel");
    let info = SystemInformation::new(info_addr);
    Initializer::new(info).initialize();
    let s = cs();
    println!("prev {:?}", s.rpl());

    loop {
        if switch() {
            enable_interrupt_and_nop();
        } else {
            enable_interrupt_and_hlt();
        }
    }
}


#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop_hlt()
}

