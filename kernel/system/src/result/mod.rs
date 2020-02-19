pub use error::mem::MemoryError;

use crate::alloc::string::String;
use crate::result::error::mem::MemErrorKind;

pub mod error;

pub type Result<T> = core::result::Result<T, Error>;

pub struct Error {
    repr: Repr,
}

impl Error {
    pub fn new_memory(kind: MemErrorKind, msg: String) -> Self {
        Error { repr: Repr::Memory(MemoryError::new(kind, msg)) }
    }
}


pub enum Repr {
    Memory(MemoryError)
}

