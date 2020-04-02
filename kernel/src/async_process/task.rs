use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::alloc::boxed::Box;
use crate::process::task::{AtomicProcessId, ProcessId};

pub struct Task {
    pub id: ProcessId,
    future: Pin<Box<dyn Future<Output=()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output=()> + 'static) -> Task {
        static ID: AtomicProcessId = AtomicProcessId::new(ProcessId::from(0));
        Task {
            id: ID.increment(),
            future: Box::pin(future),
        }
    }

    pub(crate) fn poll(&mut self, content: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(content)
    }
}
