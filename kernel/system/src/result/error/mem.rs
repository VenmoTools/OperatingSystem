use crate::alloc::string::String;

pub struct MemoryError {
    kind: MemErrorKind,
    msg: String,
}

impl MemoryError {
    pub fn new(kind: MemErrorKind, msg: String) -> Self {
        MemoryError { kind, msg }
    }
}

pub enum MemErrorKind {
    NotAligned,
}