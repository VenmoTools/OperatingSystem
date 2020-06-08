use core::future::Future;
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;

use crate::alloc::collections::{BTreeMap, VecDeque};
use crate::alloc::sync::Arc;
use crate::alloc::task::Wake;
use crate::process::types::ProcessId;

use super::task::Task;

pub struct Executor {
    task_list: VecDeque<Task>,
    waiting_queue: BTreeMap<ProcessId, Task>,
    wake_queue: Arc<ArrayQueue<ProcessId>>,
    /// 创建任务后会缓存Waker，
    waker_cache: BTreeMap<ProcessId, Waker>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_list: VecDeque::new(),
            waiting_queue: BTreeMap::new(),
            wake_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_list.push_back(task)
    }

    pub fn spawn(&mut self, task: impl Future<Output=()> + 'static) {
        self.task_list.push_back(Task::new(task))
    }

    pub fn run_ready_task(&mut self) {
        while let Some(mut task) = self.task_list.pop_front() {
            let task_id = task.id;
            if !self.waker_cache.contains_key(&task_id) {
                self.waker_cache.insert(task_id, self.create_walker(task_id));
            }

            let waker = self.waker_cache.get(&task_id).expect("should exist");
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {
                    if self.waiting_queue.insert(task_id, task).is_some() {
                        panic!("same task id already in waiting tasks");
                    }
                }
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.wake_task();
            self.run_ready_task();
            self.sleep_if_idle();
        }
    }

    pub fn create_walker(&self, id: ProcessId) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            id,
            wake_queue: self.wake_queue.clone(),
        }))
    }

    pub fn wake_task(&mut self) {
        while let Ok(id) = self.wake_queue.pop() {
            if let Some(task) = self.waiting_queue.remove(&id) {
                self.task_list.push_back(task);
            }
        }
    }

    fn sleep_if_idle(&self) {
        use system::ia_32e::instructions::interrupt::{enable_interrupt_and_hlt, enable_interrupt};

        if !self.wake_queue.is_empty() {
            return;
        }

        if self.wake_queue.is_empty() {
            enable_interrupt_and_hlt();
        } else {
            enable_interrupt();
        }
    }
}

struct TaskWaker {
    id: ProcessId,
    wake_queue: Arc<ArrayQueue<ProcessId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.wake_queue.push(self.id).expect("wake queue  full")
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
