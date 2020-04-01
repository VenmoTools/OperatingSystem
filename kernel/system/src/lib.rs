#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(asm)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

#[macro_use]
extern crate alloc;


pub use mutex::Mutex;

pub mod bits;
mod mutex;
pub mod ia_32e;
pub mod result;
pub mod devices;
pub mod macros;
#[macro_use]
pub mod console;

#[repr(C)]
#[derive(Debug)]
pub struct KernelArgs {
    pub st: u64,
    pub iter: u64,
    pub kernel_start: u64,
    pub kernel_end: u64,
    pub stack_start: u64,
    pub stack_end: u64,
    pub frame_ptr: *mut u8,
    pub frame_size: usize,
}


#[cfg(test)]
mod tests;


