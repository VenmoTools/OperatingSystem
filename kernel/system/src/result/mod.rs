pub use error::mem::MemoryError;

use crate::alloc::string::String;
use crate::result::error::mem::MemErrorKind;

pub mod error;

pub type Result<T> = core::result::Result<T, Error>;

pub trait ResultEx<T> {
    fn unwrap(self) -> T;
}

#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

impl Error {
    pub fn new_memory(kind: MemErrorKind, msg: String) -> Self {
        Error { repr: Repr::Memory(MemoryError::new(kind, msg)) }
    }
}

impl<T> ResultEx<T> for Result<T> {
    fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e)
        }
    }
}

#[derive(Debug)]
pub enum Repr {
    Memory(MemoryError)
}

