#![feature(const_fn)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![cfg_attr(feature = "deny-warnings", deny(missing_docs))]
#![cfg_attr(not(feature = "deny-warnings"), warn(missing_docs))]
#![deny(missing_debug_implementations)]

pub mod bits;
pub mod mutex;
pub mod ia_32e;


#[cfg(test)]
mod tests;


