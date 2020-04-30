use alloc::vec::Vec;
use core::alloc::Layout;

use crate::ia_32e::paging::{Frame, MemorySpace, MemoryType, UnusedFrame};
use crate::ia_32e::PhysAddr;

use super::{FrameAllocator, Page4KB, PageSize};
use super::MemoryArea;

pub trait MemoryAreaManagement {
    fn add_area(&mut self, start_addr: u64, end_addr: u64, ty: MemoryType, len: u64);
}

pub struct AdaptationAllocator<F: FrameAllocator<Page4KB> + MemoryAreaManagement> {
    inner: F,
    free: Vec<(usize, usize)>,
}

impl<F: FrameAllocator<Page4KB> + MemoryAreaManagement> AdaptationAllocator<F> {
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            free: Vec::new(),
        }
    }
    pub fn free_count(&self) -> usize {
        let mut size = 0;
        for area in self.free.iter() {
            size += area.1;
        }
        size
    }
    fn merge(&mut self, address: usize, count: usize) -> bool {
        for i in 0..self.free.len() {
            let changed = {
                let free = &mut self.free[i];
                if address + count * Page4KB::P_SIZE as usize == free.0 {
                    free.0 = address;
                    free.1 += count;
                    true
                } else if free.0 + free.1 * Page4KB::P_SIZE as usize == address {
                    free.1 += count;
                    true
                } else {
                    false
                }
            };
            if changed {
                let (address, count) = self.free[i];
                if self.merge(address, count) {
                    self.free.remove(i);
                }
                return true;
            }
        }
        false
    }
}

impl<F: FrameAllocator<Page4KB> + MemoryAreaManagement> MemoryAreaManagement for AdaptationAllocator<F> {
    fn add_area(&mut self, start_addr: u64, end_addr: u64, ty: MemoryType, len: u64) {
        self.inner.add_area(start_addr, end_addr, ty, len)
    }
}

unsafe impl<F: FrameAllocator<Page4KB> + MemoryAreaManagement> FrameAllocator<Page4KB> for AdaptationAllocator<F> {
    fn alloc(&mut self) -> Option<UnusedFrame<Page4KB>> {
        self.inner.alloc()
    }

    fn dealloc(&mut self, frame: UnusedFrame<Page4KB>) {
        self.inner.dealloc(frame)
    }

    fn free_frames(&self) -> usize {
        self.inner.free_frames() + self.free_count()
    }

    fn used_frames(&self) -> usize {
        self.inner.used_frames() - self.free_count()
    }

    fn alloc_size(&mut self, size: Layout) -> Option<UnusedFrame<Page4KB>> {
        self.inner.alloc_size(size)
    }

    fn dealloc_size(&mut self, frame: Frame, count: usize) {
        let address = frame.start_address().as_usize();
        if !self.merge(address, count) {
            self.free.push((address, count));
        }
    }
}

pub struct BumpAllocator {
    next_frame: Frame,
    current_area: Option<MemoryArea>,
    areas: MemorySpace,
    kernel_start: Frame,
    kernel_end: Frame,
}

impl BumpAllocator {
    pub fn new(kernel_start: u64, kernel_end: u64) -> Self {
        let mut allocator = Self {
            next_frame: Frame::include_address(PhysAddr::new(0)),
            current_area: None,
            areas: MemorySpace::new(),
            kernel_start: Frame::include_address(PhysAddr::new(kernel_start)),
            kernel_end: Frame::include_address(PhysAddr::new(kernel_end)),
        };
        allocator.find_next_area();
        allocator
    }


    fn find_next_area(&mut self) {
        // 找到当前区域的起始地址
        self.current_area = self.areas.space.iter().filter(|area| {
            // 获得当前内存区域大小
            let mem_area = area.start_address() + area.size() + 1;
            // 过滤出所有大于当前内存区域
            Frame::include_address(PhysAddr::new(mem_area)) >= self.next_frame
            // 找出地址最低的内存区域
        }).min_by_key(|area| area.start_address()).and_then(|f| {
            Some(*f)
        });

        if let Some(area) = self.current_area {
            let start_frame = Frame::include_address(PhysAddr::new(area.start_address()));
            if self.next_frame < start_frame {
                self.next_frame = start_frame;
            }
        }
    }
}

impl MemoryAreaManagement for BumpAllocator {
    fn add_area(&mut self, start_addr: u64, end_addr: u64, ty: MemoryType, len: u64) {
        self.areas.add_area(start_addr, end_addr, ty, len)
    }
}

unsafe impl FrameAllocator<Page4KB> for BumpAllocator {
    fn alloc(&mut self) -> Option<UnusedFrame<Page4KB>> {
        self.alloc_size(Layout::from_size_align(1, Page4KB::P_SIZE as usize).unwrap())
    }

    fn dealloc(&mut self, _frame: UnusedFrame<Page4KB>) {}

    fn free_frames(&self) -> usize {
        let mut count = 0;
        for area in self.areas.space.iter() {
            let start_frame = Frame::include_address(PhysAddr::new(area.start_address()));
            let end_frame = Frame::include_address(PhysAddr::new(area.start_address() + area.size() - 1));
            for frame in Frame::frame_range_include(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // kernel frame
                } else if frame > self.next_frame {
                    count += 1;
                } else {
                    // used
                }
            }
        }
        count
    }

    fn used_frames(&self) -> usize {
        let mut count = 0;
        for area in self.areas.space.iter() {
            let start_frame = Frame::include_address(PhysAddr::new(area.start_address()));
            let end_frame = Frame::include_address(PhysAddr::new(area.start_address() + area.size() - 1));
            for frame in Frame::frame_range_include(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // kernel frame
                    count += 1;
                } else if frame > self.next_frame {
                    // frame
                } else {
                    count += 1;
                    // used
                }
            }
        }
        count
    }

    fn alloc_size(&mut self, layout: Layout) -> Option<UnusedFrame<Page4KB>> {
        if layout.size() == 0 {
            return None;
        }
        if let Some(area) = self.current_area {
            let start_frame = Frame::include_address(self.next_frame.start_address());
            let end_frame = Frame::include_address(self.next_frame.start_address() + layout.size() * layout.align());
            // get last frame of current area
            let area_last_frame = {
                let address = area.start_address() + area.size() - 1;
                Frame::include_address(PhysAddr::new(address))
            };

            if end_frame > area_last_frame {
                self.find_next_area();
            } else if (start_frame >= self.kernel_start && start_frame <= self.kernel_end)
                || (end_frame >= self.kernel_end && end_frame <= self.kernel_end) {
                // frame use by kernel
                self.next_frame = Frame::include_address(self.kernel_end.start_address() + Page4KB::P_SIZE);
            } else {
                self.next_frame = Frame::include_address(self.next_frame.start_address() + Page4KB::P_SIZE);
                return Some(unsafe { UnusedFrame::new(start_frame) });
            }
            // if not return try it again
            self.alloc_size(layout);
        }
        None
    }

    fn dealloc_size(&mut self, _frame: Frame<Page4KB>, _count: usize) {
        unimplemented!()
    }
}