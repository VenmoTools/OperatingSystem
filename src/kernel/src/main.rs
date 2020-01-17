#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(feature = "deny-warnings", deny(missing_docs))]


#[macro_use]
extern crate kernel;

use core::panic::PanicInfo;

use kernel::{devices, Initializer, loop_hlt};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    let cpuid = raw_cpuid::CpuId::new();
    println!("cpu info:{:?}", cpuid.get_vendor_info().unwrap().as_string());
    Initializer::initialize_all();
    use x86_64::registers::control::Cr3;
    let (addr, _) = Cr3::read();
    println!("{:?}", addr);
    loop_hlt()
}

/// 用于运行过程中异常处理
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Error: {}", info);
    loop_hlt()
}

