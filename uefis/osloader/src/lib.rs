#![feature(allocator_api)]
#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;

pub use loader::{Allocator, ElfLoader, OsLoaderAlloc};

pub mod elf;
mod loader;

