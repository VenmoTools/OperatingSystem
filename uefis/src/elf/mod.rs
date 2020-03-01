pub use elf32::*;
pub use elf64::*;
pub use elf_type::*;
pub use flags::*;
pub use segment::*;
pub use traits::*;

// Most of the code for this mod comes from elf_rs crate https://github.com/vincenthouyi/elf_rs
mod elf32;
mod elf64;
mod traits;
mod elf_type;
mod segment;
mod flags;

#[allow(unused_variables)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    BufferTooShort,
    InvalidMagic,
    InvalidClass,
}
