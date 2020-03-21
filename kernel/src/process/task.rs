use ::core::sync::atomic::AtomicUsize;
use alloc::boxed::Box;

use spin::Mutex;

use crate::alloc::sync::Arc;
use crate::alloc::vec::Vec;
use crate::process::common::ProcessMemory;
use crate::process::register::ProcessRegister;

/// 进程的状态
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Runnable,
    Block,
    Stopped(usize),
    Exited(usize),
}

/// 进程标志
#[repr(u64)]
pub enum Flags {
    KernelThread,
    Thread,
    Process,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ProcessId(usize);


impl ProcessId {
    pub const fn into(self) -> usize {
        self.0
    }
    pub const fn from(x: usize) -> Self {
        Self(x)
    }
}

pub struct AtomicProcessId {
    container: AtomicUsize,
}

impl AtomicProcessId {
    #[allow(dead_code)]
    pub const fn new(id: ProcessId) -> Self {
        Self {
            container: AtomicUsize::new(id.into())
        }
    }
    #[allow(dead_code)]
    pub const fn default() -> Self {
        Self::new(ProcessId::from(0))
    }
    #[allow(dead_code)]
    pub fn load(&self, order: ::core::sync::atomic::Ordering) -> ProcessId {
        ProcessId::from(self.container.load(order))
    }

    #[allow(dead_code)]
    pub fn store(&self, val: ProcessId, order: ::core::sync::atomic::Ordering) {
        self.container.store(val.into(), order)
    }
    #[allow(dead_code)]
    pub fn swap(&self, val: ProcessId, order: ::core::sync::atomic::Ordering) -> ProcessId {
        ProcessId::from(self.container.swap(val.into(), order))
    }

    #[allow(dead_code)]
    pub fn compare_and_swap(&self, current: ProcessId, new: ProcessId, order: ::core::sync::atomic::Ordering) -> ProcessId {
        ProcessId::from(self.container.compare_and_swap(current.into(), new.into(), order))
    }

    #[allow(dead_code)]
    pub fn compare_exchange(&self, current: ProcessId, new: ProcessId, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<ProcessId, ProcessId> {
        match self.container.compare_exchange(current.into(), new.into(), success, failure) {
            Ok(result) => Ok(ProcessId::from(result)),
            Err(result) => Err(ProcessId::from(result))
        }
    }
    #[allow(dead_code)]
    pub fn compare_exchange_weak(&self, current: ProcessId, new: ProcessId, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<ProcessId, ProcessId> {
        match self.container.compare_exchange_weak(current.into(), new.into(), success, failure) {
            Ok(result) => Ok(ProcessId::from(result)),
            Err(result) => Err(ProcessId::from(result))
        }
    }
}


/// 进程主体
#[repr(C)]
pub struct Process {
    /// 进程ID
    pub id: ProcessId,
    // 进程组ID
    pub group_gid: ProcessId,
    // 父进程ID
    pub parent_pid: ProcessId,
    // 用户ID
    pub user_id: usize,
    // 用户组ID
    pub group_id: usize,
    // 用户状态
    pub status: State,
    // 进程所使用的寄存器
    pub reg: ProcessRegister,
    // 当前进程是否在运行
    pub running: bool,
    // CPU ID
    pub cpu_id: Option<usize>,
    // 进程执行次数
    pub ticks: u64,
    // 内核栈
    pub kernel_stack: Option<Box<[u8]>>,
    // 用户态堆
    pub user_heap: Option<Mutex<ProcessMemory>>,
    // 用户态栈
    pub user_stack: Option<Mutex<ProcessMemory>>,
    // 进程名
    pub name: Arc<Mutex<Box<[u8]>>>,
}

impl Process {
    pub fn new(id: ProcessId) -> Self {
        Self {
            id,
            group_gid: ProcessId::from(0),
            parent_pid: ProcessId::from(0),
            user_id: 0,
            group_id: 0,
            status: State::Block,
            running: false,
            cpu_id: None,
            reg: ProcessRegister::new(),
            ticks: 0,
            kernel_stack: None,
            user_heap: None,
            user_stack: None,
            name: Arc::new(Mutex::new(Vec::new().into_boxed_slice())),
        }
    }

    pub fn block(&mut self) -> bool {
        if self.status == State::Runnable {
            self.status = State::Block;
            true
        } else {
            false
        }
    }

    pub fn unblock(&mut self) -> bool {
        if self.status == State::Block {
            self.status = State::Runnable;
            true
        } else {
            false
        }
    }
}