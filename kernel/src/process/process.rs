use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::memory::alloc_memory;
use crate::process::memory::{Memory, SharedMemory};
use crate::process::registers::ProcessRegister;
use crate::process::types::ProcessId;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Runnable,
    Blocked,
    Stopped(usize),
    Exited(usize),
}

pub struct Process {
    pub id: ProcessId,
    // pub name_space:
    /// Signal mask
    pub sigmask: [u64; 2],
    /// Context running or not
    pub running: bool,
    /// Status of context
    pub status: Status,
    /// Head buffer to use when system call buffers are not page aligned
    pub syscall_head: Box<[u8]>,
    /// Tail buffer to use when system call buffers are not page aligned
    pub syscall_tail: Box<[u8]>,
    /// The architecture specific context
    pub register: ProcessRegister,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// User signal stack
    pub sigstack: Option<Memory>,
    /// Executable image
    pub image: Vec<SharedMemory>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<SharedMemory>,
    /// Kernel FX - used to store SIMD and FPU registers on context switch
    pub kfx: Option<Box<[u8]>>,
    /// CPU ID, if locked
    pub cpu_id: Option<usize>,

}

impl Process {
    pub fn new(id: ProcessId) -> Process {
        let syscall_head = unsafe { alloc_memory(4096).expect("allocate memory failed") };
        let syscall_tail = unsafe { alloc_memory(4096).expect("allocate memory failed") };
        Process {
            id,
            sigmask: [0; 2],
            status: Status::Blocked,
            syscall_head,
            syscall_tail,
            register: ProcessRegister::new(),
            kstack: None,
            image: Vec::new(),
            heap: None,
            stack: None,
            sigstack: None,
            kfx: None,
            running: false,
            cpu_id: None,
        }
    }

    /// Block the context, and return true if it was runnable before being blocked
    pub fn block(&mut self) -> bool {
        if self.status == Status::Runnable {
            self.status = Status::Blocked;
            true
        } else {
            false
        }
    }
}
