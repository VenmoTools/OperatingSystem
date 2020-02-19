#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(asm)]
#![allow(dead_code)]
#![allow(unused_assignments)]

#![warn(missing_docs)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![cfg_attr(feature = "deny-warnings", deny(missing_docs))]
#![cfg_attr(not(feature = "deny-warnings"), warn(missing_docs))]


#[macro_use]
extern crate alloc;
extern crate bitflags;


pub use mutex::Mutex;

pub mod bits;
mod mutex;
pub mod ia_32e;
pub mod result;


#[cfg(test)]
mod tests;


