use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::alloc::boxed::Box;
use crate::process::types::{AtomicProcessId, ProcessId};

pub struct Task {
    pub id: ProcessId,
    future: Pin<Box<dyn Future<Output=()>>>,
    pub counter: usize,
}

impl Task {
    pub fn new(future: impl Future<Output=()> + 'static) -> Task {
        static ID: AtomicProcessId = AtomicProcessId::new(ProcessId::from(0));
        Task {
            id: ID.increment(),
            future: Box::pin(future),
            counter: 100,
        }
    }

    pub(crate) fn poll(&mut self, content: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(content)
    }
}
