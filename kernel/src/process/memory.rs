use alloc::sync::{Arc, Weak};
use bitflags::_core::alloc::Layout;
use core::intrinsics;
use spin::Mutex;
use system::bits::flags::PageTableFlags;
use system::ia_32e::{PhysAddr, VirtAddr};
use system::ia_32e::paging::{Frame, Page, Page4KB, PageRangeInclude};
use system::ia_32e::paging::mapper::Mapper;

use crate::memory::{FRAME_ALLOCATOR, HEAP};

#[derive(Debug)]
pub struct Memory {
    start: VirtAddr,
    size: usize,
    flags: PageTableFlags,
}

impl Memory {
    pub fn new(start: VirtAddr, size: usize, flags: PageTableFlags, clear: bool) -> Self {
        let memory = Memory {
            start,
            size,
            flags,
        };
        memory.map(clear);
        memory
    }

    pub fn start_address(&self) -> VirtAddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn flags(&self) -> PageTableFlags {
        self.flags
    }

    pub fn pages(&self) -> PageRangeInclude {
        let start_page = Page::include_address(self.start);
        let end_page = Page::include_address(VirtAddr::new(self.start.as_u64() + self.size as u64 - 1));
        Page::range_include(start_page, end_page)
    }

    fn map(&self, clear: bool) {
        use crate::memory::RECU_PAGE_TABLE;
        let mut table = RECU_PAGE_TABLE.lock();
        for page in self.pages() {
            //todo: it should work
            let frame = HEAP.lock().alloc(Layout::from_size_align(page.size() as usize, page.size() as usize).expect("no aligned")).expect("allocate memory failed");
            let frame = Frame::include_address(PhysAddr::new(frame.as_ptr() as u64));
            unsafe {
                table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().as_mut().expect("frame allocator not init")).expect("map memory err").flush();
            }
        }
        if clear {
            assert!(self.flags.contains(PageTableFlags::WRITABLE));
            unsafe {
                intrinsics::write_bytes(self.start_address().as_mut_ptr::<*mut u8>(), 0, self.size);
            }
        }
    }

    fn unmap(&self) {
        use crate::memory::RECU_PAGE_TABLE;
        let mut table = RECU_PAGE_TABLE.lock();
        for page in self.pages() {
            table.unmap(page).expect("unmap page failed").1.flush();
        }
    }

    pub fn resize(&mut self, new_size: usize, clear: bool) {
        use crate::memory::RECU_PAGE_TABLE;
        use system::ia_32e::paging::result::TranslateError;

        let mut table = RECU_PAGE_TABLE.lock();

        if new_size > self.size {
            let start_page: Page<Page4KB> = Page::include_address(VirtAddr::new(self.start.as_u64() + self.size as u64));
            let end_page = Page::include_address(VirtAddr::new(self.start.as_u64() + new_size as u64 - 1));
            for page in Page::range_include(start_page, end_page) {
                match table.translate_page(page.clone()) {
                    Err(err) => {
                        match err {
                            TranslateError::PageNotMapped => {
                                unsafe {
                                    //todo: allocator frame
                                    let frame = HEAP.lock().alloc(Layout::from_size_align(page.size() as usize, page.size() as usize).expect("no aligned")).expect("allocate memory failed");
                                    let frame = Frame::include_address(PhysAddr::new(frame.as_ptr() as u64));
                                    table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().as_mut().expect("frame allocator not init")).expect("map page mapped failed").flush();
                                }
                            }
                            TranslateError::ParentEntryHugePage => { panic!(format!("address {:#?} already mapped", page.start_address()).as_str()) }
                            TranslateError::InvalidFrameAddress(addr) => {
                                panic!(format!("invalid frame address {:#?} ", addr).as_str())
                            }
                        }
                    }
                    _ => {}
                }
            }
            if clear {
                unsafe {
                    intrinsics::write_bytes((self.start.as_usize() + self.size) as *mut u8, 0, new_size - self.size);
                }
            }
        } else if new_size < self.size {
            let start_page: Page<Page4KB> = Page::include_address(VirtAddr::new(self.start.as_u64() + new_size as u64));
            let end_page = Page::include_address(VirtAddr::new(self.start.as_u64() + self.size as u64 - 1));
            for page in Page::range_include(start_page, end_page) {
                if table.translate_page(page.clone()).is_ok() {
                    table.unmap(page).expect("unmap page error").1.flush();
                }
            }
        }
        self.size = new_size;
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        self.unmap();
    }
}

#[derive(Clone, Debug)]
pub enum SharedMemory {
    Owned(Arc<Mutex<Memory>>),
    Borrowed(Weak<Mutex<Memory>>),
}

impl SharedMemory {
    pub fn with<F, T>(&self, f: F) -> T where F: FnOnce(&mut Memory) -> T {
        match *self {
            SharedMemory::Owned(ref memory_lock) => {
                let mut memory = memory_lock.lock();
                f(&mut *memory)
            }
            SharedMemory::Borrowed(ref memory_weak) => {
                let memory_lock = memory_weak.upgrade().expect("SharedMemory::Borrowed no longer valid");
                let mut memory = memory_lock.lock();
                f(&mut *memory)
            }
        }
    }

    pub fn borrow(&self) -> SharedMemory {
        match *self {
            SharedMemory::Owned(ref memory_lock) => SharedMemory::Borrowed(Arc::downgrade(memory_lock)),
            SharedMemory::Borrowed(ref memory_lock) => SharedMemory::Borrowed(memory_lock.clone())
        }
    }
}