use core::alloc::GlobalAlloc;

use spin::RwLock;
use system::ia_32e::cpu::control::CR3;
use system::result::{Error, ProcessErrorKind, Result};

use crate::alloc::alloc::Layout;
use crate::alloc::boxed::Box;
use crate::alloc::collections::BTreeMap;
use crate::alloc::sync::Arc;
use crate::alloc::vec::Vec;
use crate::process::task::{Process, ProcessId};

pub static MAX_PROCESS: usize = 5;

pub struct Stack {
    inner: Vec<usize>,
    size: usize,
}

impl Stack {
    pub const VALUE_SIZE: usize = core::mem::size_of::<usize>();

    pub fn new(size: usize) -> Self {
        Self {
            inner: vec![0; size],
            size: 0,
        }
    }

    pub fn push(&mut self, data: usize) {
        self.inner.insert(self.inner.capacity() - self.size, data);
    }

    pub fn as_ptr(&self) -> *const usize {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut usize {
        self.inner.as_mut_ptr()
    }
    // pub fn into_boxed_slice(self) -> Box<[u8]>{
    //     self.inner.into_boxed_slice()
    // }
}

pub struct ProcessList {
    map: BTreeMap<ProcessId, Arc<RwLock<Process>>>,
    next_id: usize,
}

impl ProcessList {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn get(&self, id: &ProcessId) -> Option<&Arc<RwLock<Process>>> {
        self.map.get(id)
    }

    pub fn iter(&self) -> crate::alloc::collections::btree_map::Iter<ProcessId, Arc<RwLock<Process>>> {
        self.map.iter()
    }

    pub fn new_process(&mut self) -> Result<&Arc<RwLock<Process>>> {
        if self.next_id >= MAX_PROCESS {
            self.next_id = 1
        }
        while self.map.contains_key(&ProcessId::from(self.next_id)) {
            self.next_id += 1;
        }

        if self.next_id >= MAX_PROCESS {
            return Err(Error::new_process(ProcessErrorKind::TryAgain, None));
        }
        let process_id = ProcessId::from(self.next_id);
        self.next_id += 1;

        let res = self.map.insert(process_id, Arc::new(RwLock::new(Process::new(process_id))));
        assert!(res.is_none());

        Ok(self.map.get(&process_id).expect("Failed to insert new context. ID is out of bounds."))
    }

    pub fn spawn(&mut self, func: extern fn()) -> Result<&Arc<RwLock<Process>>> {
        let lock = self.new_process()?;
        {
            let mut process = lock.write();
            let mut p_mem = unsafe {
                let mem = crate::HEAP.alloc(Layout::from_size_align_unchecked(512, 16)) as *mut [u8; 512];
                Box::from_raw(mem)
            };
            for value in p_mem.iter_mut() {
                *value = 0;
            }

            let mut p_stack = vec![0_u8; 65536].into_boxed_slice();
            let len = p_stack.len() - core::mem::size_of::<usize>();
            // set function to stack
            unsafe {
                let offset = p_stack.len() - core::mem::size_of::<usize>();
                let func_ptr = p_stack.as_mut_ptr().add(offset);
                *(func_ptr as *mut usize) = func as usize;
            }
            let (frame, _) = CR3::read();
            process.reg.set_page_table(frame.start_address().as_usize());
            process.reg.set_fx(p_mem.as_ptr() as usize);
            process.reg.set_stack_register(p_stack.as_ptr() as usize + len);
            process.kernel_fx = Some(p_mem);
            process.kernel_stack = Some(p_stack);
        }
        Ok(lock)
    }
}