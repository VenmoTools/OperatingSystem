use system::bits::flags::PageTableFlags;
use system::ia_32e::paging::{Page, PageRangeInclude};
use system::ia_32e::VirtAddr;

pub struct ProcessMemory {
    start_addr: VirtAddr,
    size: usize,
    flags: PageTableFlags,
}

impl ProcessMemory {
    pub fn new(start: VirtAddr, size: usize, flags: PageTableFlags) -> Self {
        let m = Self {
            size,
            start_addr: start,
            flags,
        };

        m
    }

    pub fn flags(&self) -> PageTableFlags { self.flags }

    pub fn size(&self) -> usize { self.size }

    pub fn start_address(&self) -> VirtAddr { self.start_addr }

    pub fn pages_iter(&self) -> PageRangeInclude {
        let s_page = Page::include_address(self.start_addr);
        let end = Page::include_address(VirtAddr::new(self.start_addr.as_u64() + self.size as u64 - 1));
        Page::range_include(s_page, end)
    }
}