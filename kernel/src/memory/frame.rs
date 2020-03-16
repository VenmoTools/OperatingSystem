use core::marker::PhantomData;

use system::ia_32e::paging::{FrameAllocator, PageSize, UnusedFrame};
use uefi::table::boot::MemoryMapIter;

pub struct PhysicalAllocator<'a, S: PageSize> {
    iter: &'a MemoryMapIter<'a>,
    _mark: PhantomData<S>,
}

impl<'a, S> PhysicalAllocator<'a, S> {
    pub fn new(iter: &'a MemoryMapIter) -> Self {
        Self {
            iter,
            _mark: PhantomData,
        }
    }
}

unsafe impl<'a, S: PageSize> FrameAllocator<S> for PhysicalAllocator<'a, S> {
    fn alloc(&mut self) -> Option<UnusedFrame<S>> {
        unimplemented!()
    }

    fn dealloc(&mut self, frame: UnusedFrame<S>) {
        unimplemented!()
    }
}