use core::marker::PhantomData;
use lazy_static::lazy_static;
use spin::Mutex;
use system::ia_32e::paging::{FrameAllocator, Page2MB, PageSize, UnusedFrame};
use uefi::table::boot::MemoryMapIter;

lazy_static! {
    pub static ref ALLOCATOR: Mutex<PhysicalAllocator<'static,Page2MB>> = Mutex::new(PhysicalAllocator::empty());
}

pub fn frame_allocator_init(iter: &'static MemoryMapIter) {
    ALLOCATOR.lock().init(iter);
}

pub struct PhysicalAllocator<'a, S: PageSize> {
    iter: Option<&'a MemoryMapIter<'a>>,
    _mark: PhantomData<S>,
}


impl<'a, S: PageSize> PhysicalAllocator<'a, S> {
    pub fn empty() -> Self {
        Self {
            iter: None,
            _mark: PhantomData,
        }
    }

    pub fn init(&mut self, iter: &'a MemoryMapIter) {
        self.iter = Some(iter);
    }

    pub fn new(iter: &'a MemoryMapIter) -> Self {
        Self {
            iter: Some(iter),
            _mark: PhantomData,
        }
    }
}

unsafe impl<'a, S: PageSize> FrameAllocator<S> for PhysicalAllocator<'a, S> {
    fn alloc(&mut self) -> Option<UnusedFrame<S>> {
        unimplemented!()
    }

    fn dealloc(&mut self, _frame: UnusedFrame<S>) {
        unimplemented!()
    }
}