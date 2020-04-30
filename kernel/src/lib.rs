#![no_std]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![allow(dead_code)]
#![feature(core_intrinsics)]
#![feature(thread_local)]
#![feature(wake_trait)]
// #![deny(warnings)]

#[macro_use]
extern crate alloc;
extern crate multiboot2;
// #[macro_use]
extern crate system;

use core::panic::PanicInfo;
use system::KernelArgs;

use crate::initializer::Initializer;
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

#[cfg(feature = "uefi")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("uefi entry");
    let args = unsafe { &*((args) as *const KernelArgs) };
    let iter = get_mem_iter(args);
    loop_hlt()
}

#[cfg(feature = "mutiboot")]
#[no_mangle]
extern "C" fn kmain(info_addr: usize) -> ! {
    println!("entry kernel");
    let boot = unsafe { multiboot2::load(info_addr) };
    Initializer::new(&boot).initialize();
    loop_hlt()
}


#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop_hlt()
}

#[cfg(feature = "uefi")]
fn get_mem_iter(args: &KernelArgs) -> &mut MemoryMapIter {
    unsafe { &mut *(args.iter as *mut MemoryMapIter) }
}

#[cfg(feature = "uefi")]
#[allow(dead_code)]
fn get_system_table(args: &KernelArgs) -> &SystemTable<Runtime> {
    unsafe { &*(args.st as *const SystemTable<Runtime>) }
}
