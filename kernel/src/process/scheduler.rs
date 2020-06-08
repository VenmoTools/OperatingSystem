use alloc::collections::VecDeque;
use alloc::sync::Arc;

use bitflags::_core::sync::atomic::{AtomicBool, Ordering};
use spin::{Mutex, RwLock};
use system::ia_32e::instructions::interrupt::{disable_interrupt, enable_interrupt, system_pause};

use lazy_static::lazy_static;

use crate::descriptor::TICKS;
use crate::process::{CURRENT_PROCESS, process_mut};
use crate::process::process::{Process, Status};
use crate::process::types::ProcessId;

lazy_static! {
    pub static ref SCHEDULER:Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub const SWITCH_LOCK: AtomicBool = AtomicBool::new(false);

pub struct Scheduler {
    queue: VecDeque<ProcessId>,
    count: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            count: 10,
        }
    }

    pub fn add_process(&mut self, id: ProcessId) {
        self.queue.push_back(id)
    }

    pub fn next_process(&mut self) -> Option<ProcessId> {
        self.queue.pop_front()
    }

    pub fn wait_for_switch(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        } else {
            self.switch_to_next()
        }
    }

    pub fn switch_to_next(&mut self) {
        if self.queue.is_empty() {
            panic!("no process found");
        }
        disable_interrupt();
        let list = process_mut();
        let next_process_id = self.next_process().expect("empty queue");
        let mut next_process = list.get(next_process_id).expect("no process found").write();
        let mut process = list.current().expect("no process run").write();
        next_process.running = true;
        next_process.status = Status::Runnable;
        self.count = 100;
        process.status = Status::Blocked;
        process.running = false;
        let id = CURRENT_PROCESS.swap(next_process_id, Ordering::SeqCst);
        self.queue.push_back(id);
        unsafe {
            process.register.switch_to(&mut next_process.register);
        }
        enable_interrupt();
    }
}

// use too many lock
pub fn switch() -> bool {
    // lock here
    while SWITCH_LOCK.compare_and_swap(false, true, Ordering::SeqCst) {
        system_pause();
    }
    let current_process;
    let mut next_process: Option<&Arc<RwLock<Process>>> = None;
    // get lock here
    let process = process_mut();
    {
        let lock = process.current().expect("not process found");
        current_process = lock;
    }// `lock` will release here
    {
        let current = current_process.write();
        // find next runnable process
        for (id, proc) in process.iter() {
            if *id > current.id {
                let lock = proc.write();
                if !lock.running && lock.status == Status::Runnable {
                    next_process = Some(proc);
                    break;
                }
            }
        }
    }// `current` will release here

    // if not found process in last time, then do that again
    if next_process.is_none() {
        let current = current_process.write();
        for (id, proc) in process.iter() {
            if *id > current.id {
                let lock = proc.write();
                if !lock.running && lock.status == Status::Runnable {
                    next_process = Some(proc);
                    break;
                }
            }
        }
    }// `current` will release here

    // if found next runnable process, change process status
    if next_process.is_some() {
        let mut current = current_process.write();
        current.running = false;
        let next = next_process.unwrap();
        {
            let mut n = next.write();
            n.running = true;
            // need set tss stack?
            CURRENT_PROCESS.store(n.id, Ordering::SeqCst);
        }
    }// `current` will release here

    SWITCH_LOCK.store(false, Ordering::SeqCst);
    let res = if next_process.is_none() {
        false
    } else {
        let mut next = next_process.unwrap().write();
        let mut current = current_process.write();
        unsafe {
            current.register.switch_to(&mut next.register)
        }
        true
    };// `next` will release here
    TICKS.store(0, Ordering::SeqCst);
    res
}
