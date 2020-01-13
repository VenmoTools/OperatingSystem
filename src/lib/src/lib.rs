#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(asm)]
#![allow(dead_code)]
#![warn(missing_docs)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![cfg_attr(feature = "deny-warnings", deny(missing_docs))]
#![cfg_attr(not(feature = "deny-warnings"), warn(missing_docs))]


extern crate bitflags;

pub mod bits;
mod mutex;
pub mod ia_32e;
pub use mutex::Mutex;


#[cfg(test)]
mod tests;


