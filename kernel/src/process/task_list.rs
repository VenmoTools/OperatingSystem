use spin::RwLock;
use system::result::{Error, ProcessErrorKind, Result};

use crate::alloc::collections::BTreeMap;
use crate::alloc::sync::Arc;
use crate::process::task::{Process, ProcessId};

pub static MAX_PROCESS: usize = 5;


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
}