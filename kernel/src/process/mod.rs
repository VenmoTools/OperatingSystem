use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use bitflags::_core::sync::atomic::Ordering;
use core::mem;
use core::sync::atomic::AtomicUsize;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};
use system::ia_32e::cpu::control::CR3;
use system::result::{Error, ProcessErrorKind, Result};

use crate::memory::alloc_memory;
use crate::process::process::{Process, Status};
use crate::process::types::{AtomicProcessId, MAX_PROCESS, ProcessId};

pub mod registers;
pub mod process;
pub mod types;
pub mod scheduler;
pub mod memory;


/// A unique number that identifies the current CPU - used for scheduling
#[thread_local]
static CPU_ID: AtomicUsize = AtomicUsize::new(0);

/// Get the current CPU's scheduling ID
#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.load(Ordering::Relaxed)
}

pub static CURRENT_PROCESS: AtomicProcessId = AtomicProcessId::default();
static CONTEXTS: Once<RwLock<ProcessList>> = Once::new();

pub struct ProcessList {
    list: BTreeMap<ProcessId, Arc<RwLock<Process>>>,
    next_id: usize,
}

impl ProcessList {
    pub fn new() -> Self {
        Self {
            list: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn get(&self, id: ProcessId) -> Option<&Arc<RwLock<Process>>> {
        self.list.get(&id)
    }

    pub fn current(&self) -> Option<&Arc<RwLock<Process>>> {
        self.list.get(&CURRENT_PROCESS.load(Ordering::SeqCst))
    }

    pub fn iter(&self) -> ::alloc::collections::btree_map::Iter<ProcessId, Arc<RwLock<Process>>> {
        self.list.iter()
    }

    pub fn new_process(&mut self) -> Result<&Arc<RwLock<Process>>> {
        if self.next_id >= MAX_PROCESS {
            self.next_id = 1;
        }
        while self.list.contains_key(&ProcessId::from(self.next_id)) {
            self.next_id += 1;
        }
        if self.next_id >= MAX_PROCESS {
            return Err(Error::new_process(ProcessErrorKind::CrateNewProcessFailed, Some(String::from("create new process failed: no process id allocate"))));
        }

        let id = ProcessId::from(self.next_id);
        self.next_id += 1;

        if self.list.insert(id, Arc::new(RwLock::new(Process::new(id)))).is_some() {
            return Err(
                Error::new_process(
                    ProcessErrorKind::CrateNewProcessFailed,
                    Some(format!("create new process failed: already create same process pid:[{:?}]", id)),
                )
            );
        }

        Ok(self.list.get(&id).expect("Failed to insert new context. ID is out of bounds."))
    }

    pub fn spawn(&mut self, func: fn()) -> Result<&Arc<RwLock<Process>>> {
        let r_lock = self.new_process()?;
        let mut pro = r_lock.write();
        let fx = unsafe { alloc_memory(512).expect("allocate memory failed") };
        let mut stack = vec![0_u8; 65536].into_boxed_slice();
        let offset = stack.len() - mem::size_of::<usize>();
        unsafe {
            let offset = stack.len() - mem::size_of::<usize>();
            let func_ptr = stack.as_mut_ptr().add(offset);
            *(func_ptr as *mut usize) = func as usize;
        }
        let frame = {
            CR3::read().0.start_address().as_usize()
        };
        pro.register.set_page_table(frame);
        pro.register.set_fx(fx.as_ptr() as usize);
        pro.register.set_stack(stack.as_ptr() as usize + offset);
        pro.kstack = Some(stack);
        pro.kfx = Some(fx);
        Ok(r_lock)
    }

    pub fn remove(&mut self, id: ProcessId) -> Option<Arc<RwLock<Process>>> {
        self.list.remove(&id)
    }
}

/// Initialize contexts, called if needed
fn init_contexts() -> RwLock<ProcessList> {
    RwLock::new(ProcessList::new())
}

/// Get the global schemes list, const
pub fn process() -> RwLockReadGuard<'static, ProcessList> {
    //call once will init_contexts only once during the kernel's exececution, otherwise it will return the current context via a
    //cache.
    CONTEXTS.call_once(init_contexts).read()
}

/// Get the global schemes list, mutable
pub fn process_mut() -> RwLockWriteGuard<'static, ProcessList> {
    CONTEXTS.call_once(init_contexts).write()
}

pub fn init_process() {
    let mut context = process_mut();
    let lock = context.new_process().expect("could not initialize first context");
    let mut context = lock.write();
    let fx = unsafe { alloc_memory(512).expect("allocate memory failed") };
    context.register.set_fx(fx.as_ptr() as usize);
    context.kfx = Some(fx);
    context.status = Status::Runnable;
    context.running = true;
    // context.cpu_id = Some(cpu_id());
    CURRENT_PROCESS.store(context.id, Ordering::SeqCst);
}

pub extern fn userspace() {
    println!("user space");
}
