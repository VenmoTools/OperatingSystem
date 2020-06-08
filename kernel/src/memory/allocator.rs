use alloc::boxed::Box;
use core::alloc::Layout;

use bitflags::_core::ptr::slice_from_raw_parts_mut;
use spin::Mutex;
use system::buddy_system_allocator::LockedHeap;
use system::ia_32e::paging::frame_allocator::{AdaptationAllocator, BumpAllocator};
use system::result::Result;

use lazy_static::lazy_static;

use crate::utils::loop_hlt;

#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Option<AdaptationAllocator<BumpAllocator>>> = Mutex::new(None);
}

pub fn init_frame_allocator(start: u64, end: u64) {
    *FRAME_ALLOCATOR.lock() = Some(AdaptationAllocator::new(BumpAllocator::new(start, end)))
}

#[alloc_error_handler]
fn handler(layout: Layout) -> ! {
    println!("allocate memory Error: align={} ,size={}", layout.align(), layout.size());
    loop_hlt()
}

pub unsafe fn add_to_heap(start: usize, end: usize) {
    HEAP.lock().add_to_heap(start, end)
}


pub unsafe fn alloc_memory(size: usize) -> Result<Box<[u8]>> {
    let ptr = HEAP.lock().alloc(Layout::from_size_align_unchecked(size, 16))?;
    let slice = slice_from_raw_parts_mut(ptr.as_ptr(), size);
    let mut box_ptr = Box::from_raw(slice);

    for i in box_ptr.iter_mut() {
        *i = 0;
    }
    Ok(box_ptr)
}

