#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(asm)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![feature(abi_x86_interrupt)]


#[macro_use]
extern crate alloc;
extern crate bitflags;


pub use mutex::Mutex;

pub mod bits;
mod mutex;
pub mod ia_32e;
pub mod result;
pub mod devices;


#[cfg(test)]
mod tests;


