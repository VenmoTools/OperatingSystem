use core::fmt;
use core::fmt::{Error, Formatter};

use crate::alloc::string::String;

#[derive(Debug)]
pub struct MemoryError {
    kind: MemErrorKind,
    msg: String,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            MemErrorKind::NotAligned => write!(f, "{}", self.msg)
        }
    }
}

impl MemoryError {
    pub fn new(kind: MemErrorKind, msg: String) -> Self {
        MemoryError { kind, msg }
    }
}

#[derive(Debug)]
pub enum MemErrorKind {
    NotAligned,
}