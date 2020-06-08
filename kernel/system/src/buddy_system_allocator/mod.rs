/// this mod base on https://github.com/rcore-os/buddy_system_allocator


use alloc::alloc::AllocRef;
use alloc::alloc::{AllocErr, Layout};
// #[rustversion::since(2020-04-02)]
use alloc::alloc::{AllocInit, MemoryBlock};
use core::alloc::GlobalAlloc;
use core::cmp::{max, min};
use core::fmt;
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::NonNull;
use core::marker::{Sync,Send};


mod frame;
pub mod linked_list;
pub use frame::*;
use crate::Mutex;

/// A heap that uses buddy system
///
/// # Usage
///
/// Create a heap and add a memory region to it:
/// ```
/// use buddy_system_allocator::*;
/// # use core::mem::size_of;
/// use system::buddy_system_allocator::Heap;
/// let mut heap = Heap::empty();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.init(begin, size);
///     // or
///     heap.add_to_heap(begin, end);
/// }
/// ```
pub struct Heap {
    // buddy system with max order of 32
    free_list: [linked_list::LinkedList; 32],

    // statistics
    user: usize,
    allocated: usize,
    total: usize,
}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            free_list: [linked_list::LinkedList::new(); 32],
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    /// Create an empty heap
    pub const fn empty() -> Self {
        Self::new()
    }

    /// Add a range of memory [start, end) to the heap
    pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        // avoid unaligned access on some platforms
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end = end & (!size_of::<usize>() + 1);
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start + size_of::<usize>() <= end {
            let lowbit = current_start & (!current_start + 1);
            let size = min(lowbit, prev_power_of_two(end - current_start));
            total += size;

            self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
            current_start += size;
        }

        self.total += total;
    }

    /// Add a range of memory [start, end) to the heap
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.add_to_heap(start, start + size);
    }

    /// Alloc a range of memory from the heap satifying `layout` requirements
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // Find the first non-empty size class
            if !self.free_list[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block) = self.free_list[j].pop() {
                        unsafe {
                            self.free_list[j - 1]
                                .push((block as usize + (1 << (j - 1))) as *mut usize);
                            self.free_list[j - 1].push(block);
                        }
                    } else {
                        return Err(AllocErr {});
                    }
                }

                let result = NonNull::new(
                    self.free_list[class]
                        .pop()
                        .expect("current block should have free space now")
                        as *mut u8,
                );
                if let Some(result) = result {
                    self.user += layout.size();
                    self.allocated += size;
                    return Ok(result);
                } else {
                    return Err(AllocErr {});
                }
            }
        }
        Err(AllocErr {})
    }

    /// Dealloc a range of memory from the heap
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;

        unsafe {
            // Put back into free list
            self.free_list[class].push(ptr.as_ptr() as *mut usize);

            // Merge free buddy lists
            let mut current_ptr = ptr.as_ptr() as usize;
            let mut current_class = class;
            while current_class < self.free_list.len() {
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut() {
                    if block.value() as usize == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }

                // Free buddy found
                if flag {
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr, buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                } else {
                    break;
                }
            }
        }

        self.user -= layout.size();
        self.allocated -= size;
    }

    /// Return the number of bytes that user requests
    pub fn stats_alloc_user(&self) -> usize {
        self.user
    }

    /// Return the number of bytes that are actually allocated
    pub fn stats_alloc_actual(&self) -> usize {
        self.allocated
    }

    /// Return the total number of bytes in the heap
    pub fn stats_total_bytes(&self) -> usize {
        self.total
    }
}

impl fmt::Debug for Heap {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Heap")
            .field("user", &self.user)
            .field("allocated", &self.allocated)
            .field("total", &self.total)
            .finish()
    }
}

unsafe impl AllocRef for Heap {
    // fn alloc(&mut self, layout: Layout) -> Result<(NonNull<u8>, usize), AllocErr> {
    //     self.alloc(layout).map(|p| (p, layout.size()))
    // }

    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        self.alloc(layout).map(|p| {
            let block = MemoryBlock {
                ptr: p,
                size: layout.size(),
            };
            unsafe {
                init.init(block);
            }
            block
        })
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr, layout)
    }
}

/// A locked version of `Heap`
///
/// # Usage
///
/// Create a locked heap and add a memory region to it:
/// ```
/// use buddy_system_allocator::*;
/// # use core::mem::size_of;
/// use system::buddy_system_allocator::LockedHeap;
/// let mut heap = LockedHeap::new();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.lock().init(begin, size);
///     // or
///     heap.lock().add_to_heap(begin, end);
/// }
/// ```
pub struct LockedHeap(Mutex<Heap>);

impl LockedHeap {
    /// Creates an empty heap
    pub const fn new() -> LockedHeap {
        LockedHeap(Mutex::new(Heap::new()))
    }

    /// Creates an empty heap
    pub const fn empty() -> LockedHeap {
        LockedHeap(Mutex::new(Heap::new()))
    }
}

unsafe impl Sync for LockedHeap{}
unsafe impl Send for LockedHeap{}

impl Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// A locked version of `Heap` with rescue before oom
///
/// # Usage
///
/// Create a locked heap:
/// ```
/// use buddy_system_allocator::*;
/// use system::buddy_system_allocator::{LockedHeapWithRescue, Heap};
/// let heap = LockedHeapWithRescue::new(|heap: &mut Heap| {});
/// ```
///
/// Before oom, the allocator will try to call rescue function and try for one more time.
pub struct LockedHeapWithRescue {
    inner: Mutex<Heap>,
    rescue: fn(&mut Heap),
}

impl LockedHeapWithRescue {
    /// Creates an empty heap
    pub const fn new(rescue: fn(&mut Heap)) -> LockedHeapWithRescue {
        LockedHeapWithRescue {
            inner: Mutex::new(Heap::new()),
            rescue,
        }
    }
}

impl Deref for LockedHeapWithRescue {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Mutex<Heap> {
        &self.inner
    }
}

unsafe impl GlobalAlloc for LockedHeapWithRescue {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();
        match inner.alloc(layout) {
            Ok(allocation) => allocation.as_ptr(),
            Err(_) => {
                (self.rescue)(&mut inner);
                inner
                    .alloc(layout)
                    .ok()
                    .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner
            .lock()
            .dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

pub(crate) fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}
