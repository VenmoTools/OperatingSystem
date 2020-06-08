use crate::ia_32e::paging::mapper::map_pt::{MappedPageTable, PhysicalToVirtual};
use crate::ia_32e::VirtAddr;
use crate::ia_32e::paging::PageTable;
use crate::ia_32e::paging::frame::Frame;
use crate::ia_32e::paging::page::{Page4KB, Page1GB, Page, Page2MB};
use crate::ia_32e::paging::mapper::{Mapper, MapperFlush, MapAllSize};
use crate::ia_32e::paging::allocator::{UnusedFrame, FrameAllocator};
use crate::ia_32e::paging::result::{UnmapError, FlagUpdateError, TranslateError, MapToError, TranslationResult};
use crate::bits::PageTableFlags;

#[derive(Debug)]
pub struct PhysOffset {
    offset: VirtAddr,
}

impl PhysicalToVirtual for PhysOffset {
    fn phy_to_vir(&self, phy_frame: Frame<Page4KB>) -> *mut PageTable {
        let phy = phy_frame.start_address().as_u64();
        let virt_ptr = self.offset + phy;
        virt_ptr.as_mut_ptr()
    }
}

#[derive(Debug)]
pub struct PageTableOffset<'a> {
    inner: MappedPageTable<'a, PhysOffset>
}

impl<'a> PageTableOffset<'a> {
    pub unsafe fn new(level_4_page_table: &'a mut PageTable, virt_offset: VirtAddr) -> Self {
        let offset = PhysOffset { offset: virt_offset };
        Self {
            inner: MappedPageTable::new(level_4_page_table, offset)
        }
    }
}

impl<'a> Mapper<Page4KB> for PageTableOffset<'a> {
    unsafe fn map_to<A>(&mut self, page: Page<Page4KB>, frame: Frame<Page4KB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.inner.map_to(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page4KB>) -> Result<(Frame<Page4KB>, MapperFlush<Page4KB>), UnmapError> {
        self.inner.unmap(page)
    }

    unsafe fn update_flags(&mut self, page: Page<Page4KB>, flags: PageTableFlags) -> Result<MapperFlush<Page4KB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    fn translate_page(&mut self, page: Page<Page4KB>) -> Result<Frame<Page4KB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> Mapper<Page2MB> for PageTableOffset<'a> {
    unsafe fn map_to<A>(&mut self, page: Page<Page2MB>, frame: Frame<Page2MB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.inner.map_to(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page2MB>) -> Result<(Frame<Page2MB>, MapperFlush<Page2MB>), UnmapError> {
        self.inner.unmap(page)
    }

    unsafe fn update_flags(&mut self, page: Page<Page2MB>, flags: PageTableFlags) -> Result<MapperFlush<Page2MB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    fn translate_page(&mut self, page: Page<Page2MB>) -> Result<Frame<Page2MB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> Mapper<Page1GB> for PageTableOffset<'a> {
    unsafe fn map_to<A>(&mut self, page: Page<Page1GB>, frame: Frame<Page1GB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.inner.map_to(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page1GB>) -> Result<(Frame<Page1GB>, MapperFlush<Page1GB>), UnmapError> {
        self.inner.unmap(page)
    }

    unsafe fn update_flags(&mut self, page: Page<Page1GB>, flags: PageTableFlags) -> Result<MapperFlush<Page1GB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    fn translate_page(&mut self, page: Page<Page1GB>) -> Result<Frame<Page1GB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> MapAllSize for PageTableOffset<'a> {
    fn translate(&self, addr: VirtAddr) -> TranslationResult {
        self.inner.translate(addr)
    }
}