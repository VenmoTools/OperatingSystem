pub use error::mem::{MemoryError, MemErrorKind};

use crate::alloc::string::String;

pub mod error;

pub type Result<T> = core::result::Result<T, Error>;

pub trait ResultEx<T> {
    fn unwrap(self) -> T;
}

#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessErrorKind {
    TryAgain
}

#[derive(Debug, Copy, Clone)]
pub enum DevicesErrorKind {
    NotSupport
}

impl Error {
    pub fn new_memory(kind: MemErrorKind, msg: String) -> Self {
        Error { repr: Repr::Memory(MemoryError::new(kind, msg)) }
    }

    pub fn new_process(kind: ProcessErrorKind, msg: Option<String>) -> Self {
        Error { repr: Repr::Process(ProcessError::new(kind, msg)) }
    }

    pub fn new_devices(kind:DevicesErrorKind,msg:Option<String>) ->Self{
        Error { repr: Repr::Devices(DevicesError::new(kind, msg)) }
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
    Memory(MemoryError),
    Process(ProcessError),
    Devices(DevicesError),
}

#[derive(Debug, Clone)]
pub struct DevicesError {
    msg: Option<String>,
    kind: DevicesErrorKind,
}

impl DevicesError {
    pub fn new(kind: DevicesErrorKind, msg: Option<String>) -> Self {
        Self {
            msg,
            kind,
        }
    }
}


#[derive(Debug, Clone)]
pub struct ProcessError {
    msg: Option<String>,
    no: isize,
}

impl ProcessError {
    pub fn new(kind: ProcessErrorKind, msg: Option<String>) -> Self {
        Self {
            msg,
            no: match kind {
                ProcessErrorKind::TryAgain => 11
            },
        }
    }
}

