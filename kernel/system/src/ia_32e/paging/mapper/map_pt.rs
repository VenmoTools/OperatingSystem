use crate::ia_32e::paging::frame::Frame;
use crate::ia_32e::paging::{PageTable, PageTableEntry};
use crate::ia_32e::paging::page::{Page4KB, Page, Page1GB, Page2MB};
use crate::ia_32e::paging::result::{PageTableWalkError, CreatePageTableError, MapToError, UnmapError, FlagUpdateError, TranslateError, TranslationResult};
use crate::ia_32e::paging::allocator::{FrameAllocator, UnusedFrame};
use crate::bits::PageTableFlags;
use crate::ia_32e::paging::mapper::{MapperFlush, Mapper, MapAllSize};
use crate::ia_32e::paging::result::FrameError;
use crate::ia_32e::VirtAddr;

/// 将给定的物理帧转换为页表裸指针
pub trait PhysicalToVirtual {
    fn phy_to_vir(&self, phy_frame: Frame) -> *mut PageTable;
}

impl<T> PhysicalToVirtual for T where T: Fn(Frame) -> *mut PageTable {
    fn phy_to_vir(&self, phy_frame: Frame<Page4KB>) -> *mut PageTable {
        self(phy_frame)
    }
}

/// 用于遍历页表的结构
#[derive(Debug)]
struct PageTableWalker<P: PhysicalToVirtual> {
    phy_to_vir: P,
}

impl<P: PhysicalToVirtual> PageTableWalker<P> {
    ///
    /// # Safety
    ///
    pub unsafe fn new(p: P) -> Self {
        Self { phy_to_vir: p }
    }

    /// MappedPageTable内部辅助函数可获取对下一级页面表的引用。
    /// 如果未使用该条目，则返回 `PageTableWalkError::NotMapped`。
    /// 如果在传递的条目中设置了`HUGE_PAGE`标志，则返回`PageTableWalkError::MappedToHugePage`。
    fn next_table<'a>(&self, entry: &'a PageTableEntry) -> Result<&'a PageTable, PageTableWalkError> {
        let table_ptr = self.phy_to_vir.phy_to_vir(entry.frame()?);
        let page_table: &PageTable = unsafe { &*table_ptr };
        Ok(page_table)
    }
    /// MappedPageTable内部辅助函数可获取对下一级页面表的可变引用。
    /// 如果未使用该条目，则返回 `PageTableWalkError::NotMapped`。
    /// 如果在传递的条目中设置了`HUGE_PAGE`标志，则返回`PageTableWalkError::MappedToHugePage`。
    fn next_table_mut<'a>(&self, entry: &'a mut PageTableEntry) -> Result<&'a mut PageTable, PageTableWalkError> {
        let table_ptr = self.phy_to_vir.phy_to_vir(entry.frame()?);
        let page_table: &mut PageTable = unsafe { &mut *table_ptr };
        Ok(page_table)
    }

    /// MappedPageTable内部辅助函数可根据需要创建下一级的页表。
    /// 如果传递的`entry`未使用，则从给定的分配器分配一个新帧，将其清零，然后将该`entry`更新到该地址。
    /// 如果传递的`entry`已被映射，则直接返回下一个表。
    /// 如果`entry`未使用并且分配器返回`None`，则返回`MapToError::FrameAllocationFailed`。
    /// 如果在传递的条目中设置了`HUGE_PAGE`标志，则返回`MapToError::ParentEntryHugePage`。
    fn create_next_table<'a, A>(&self, entry: &'a mut PageTableEntry, allocator: &mut A) -> Result<&'a mut PageTable, CreatePageTableError>
        where A: FrameAllocator<Page4KB> {
        let mut created = false;
        // 如果当前entry没有被使用可以创建新的entry
        if entry.is_unused() {
            // 申请新的帧
            if let Some(frame) = allocator.alloc() {
                entry.set_frame(frame.frame(), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
                created = true;
            } else {
                return Err(CreatePageTableError::FrameAllocateFailed);
            }
        }
        let pt = match self.next_table_mut(entry) {
            Ok(table) => table,
            Err(PageTableWalkError::MappedToHugePage) => Err(CreatePageTableError::MappedToHugePage)?,
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
        };
        if created {
            pt.zero();
        }
        Ok(pt)
    }
}

/// 已映射的页表
#[derive(Debug)]
pub struct MappedPageTable<'a, P: PhysicalToVirtual> {
    pt_walker: PageTableWalker<P>,
    level_4_table: &'a mut PageTable,
}

impl<'a, P: PhysicalToVirtual> MappedPageTable<'a, P> {
    // 创建4级页表
    pub unsafe fn new(level_4_table: &'a mut PageTable, phy_to_vir: P) -> Self {
        Self {
            pt_walker: PageTableWalker::new(phy_to_vir),
            level_4_table,
        }
    }
    // 根据给定的帧和页面进行1gb页面映射
    fn map_to_1g<A>(&mut self, page: Page<Page1GB>, frame: UnusedFrame<Page1GB>, flags: PageTableFlags, allocator: &mut A)
                    -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.create_next_table(&mut p4[page.p4_index()], allocator)?;
        // 将frame与页面做映射
        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }
    // 根据给定的帧和页面进行2mb页面映射
    fn map_to_2mb<A>(&mut self, page: Page<Page2MB>, frame: UnusedFrame<Page2MB>, flags: PageTableFlags, allocator: &mut A)
                     -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = &mut self.level_4_table;
        // 创建3级页表
        let p3 = self.pt_walker.create_next_table(&mut p4[page.p4_index()], allocator)?;
        // 创建2级页表
        let p2 = self.pt_walker.create_next_table(&mut p3[page.p3_index()], allocator)?;
        // 将frame与页面做映射
        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
    // 根据给定的帧和页面进行4kb页面映射
    fn map_to_4kb<A>(&mut self, page: Page<Page4KB>, frame: UnusedFrame<Page4KB>, flags: PageTableFlags, allocator: &mut A)
                     -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = &mut self.level_4_table;
        // 创建3级页表
        let p3 = self.pt_walker.create_next_table(&mut p4[page.p4_index()], allocator)?;
        // 创建2级页表
        let p2 = self.pt_walker.create_next_table(&mut p3[page.p3_index()], allocator)?;
        // 创建1级页表
        let p1 = self.pt_walker.create_next_table(&mut p2[page.p2_index()], allocator)?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame.frame(), flags);

        Ok(MapperFlush::new(page))
    }
}

//////////////////////
///// Mapping Memory
/////////////////////

impl<'a, P: PhysicalToVirtual> Mapper<Page4KB> for MappedPageTable<'a, P> {
    fn map_to<A>(&mut self, page: Page<Page4KB>, frame: UnusedFrame<Page4KB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_4kb(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page4KB>) -> Result<(Frame<Page4KB>, MapperFlush<Page4KB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;
        let p1 = self.pt_walker.next_table_mut(&mut p2[page.p1_index()])?;

        let entry = &mut p1[page.p1_index()];

        let frame = entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;
        entry.set_unused();

        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page4KB>, flags: PageTableFlags) -> Result<MapperFlush<Page4KB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;
        let p1 = self.pt_walker.next_table_mut(&mut p2[page.p1_index()])?;

        if !p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page4KB>) -> Result<Frame<Page4KB>, TranslateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;
        let p1 = self.pt_walker.next_table_mut(&mut p2[page.p1_index()])?;

        let entry = p1[page.p1_index()];

        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(entry.addr()))
    }
}

impl<'a, P: PhysicalToVirtual> Mapper<Page2MB> for MappedPageTable<'a, P> {
    fn map_to<A>(&mut self, page: Page<Page2MB>, frame: UnusedFrame<Page2MB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_2mb(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page2MB>) -> Result<(Frame<Page2MB>, MapperFlush<Page2MB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;

        let entry = &mut p2[page.p2_index()];
        let flags = entry.flags();
        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_addr(entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(entry.addr()))?;

        entry.set_unused();

        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page2MB>, flags: PageTableFlags) -> Result<MapperFlush<Page2MB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        let entry = &mut p2[page.p2_index()];

        entry.set_flags(flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page2MB>) -> Result<Frame<Page2MB>, TranslateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let p2 = self.pt_walker.next_table_mut(&mut p3[page.p2_index()])?;
        let entry = &mut p2[page.p2_index()];

        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr()).map_err(|_| {
            TranslateError::InvalidFrameAddress(entry.addr())
        })
    }
}

impl<'a, P: PhysicalToVirtual> Mapper<Page1GB> for MappedPageTable<'a, P> {
    fn map_to<A>(&mut self, page: Page<Page1GB>, frame: UnusedFrame<Page1GB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_1g(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page1GB>) -> Result<(Frame<Page1GB>, MapperFlush<Page1GB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;

        let entry = &mut p3[page.p3_index()];
        let flags = entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_addr(entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(entry.addr()))?;

        entry.set_unused();

        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page1GB>, flags: PageTableFlags) -> Result<MapperFlush<Page1GB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        let entry = &mut p3[page.p3_index()];
        entry.set_flags(flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page1GB>) -> Result<Frame<Page1GB>, TranslateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self.pt_walker.next_table_mut(&mut p4[page.p3_index()])?;
        let entry = &mut p3[page.p3_index()];

        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr()).map_err(|_| {
            TranslateError::InvalidFrameAddress(entry.addr())
        })
    }
}

impl<'a, P: PhysicalToVirtual> MapAllSize for MappedPageTable<'a, P>{
    fn translate(&self, addr: VirtAddr) -> TranslationResult {
        let p4 = &self.level_4_table;
        let p3 = match self.pt_walker.next_table(&p4[addr.page4_index()]){
            Ok(pt)=>pt,
            Err(PageTableWalkError::NotMapped) =>return TranslationResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => panic!("level 4 entry has huge page bit set")
        };
        let p2 = match self.pt_walker.next_table(&p3[addr.page3_index()]){
            Ok(pt)=>pt,
            Err(PageTableWalkError::NotMapped) =>return TranslationResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let frame =Frame::include_address(p3[addr.page3_index()].addr());
                let offset = addr.as_u64() & 0o_777_777_7777;
                return TranslationResult::Frame1GB { frame, offset };
            }
        };
        let p1 = match self.pt_walker.next_table(&p2[addr.page2_index()]){
            Ok(pt)=>pt,
            Err(PageTableWalkError::NotMapped) =>return TranslationResult::PageNotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let frame =Frame::include_address(p2[addr.page2_index()].addr());
                let offset = addr.as_u64() & 0o_777_777_7777;
                return TranslationResult::Frame1GB { frame, offset };
            }
        };

        let entry = &p1[addr.page1_index()];
        if entry.is_unused(){
            return TranslationResult::PageNotMapped;
        }

        let frame = match Frame::from_start_addr(entry.addr()) {
            Ok(frame) => frame,
            Err(_) => return TranslationResult::InvalidFrameAddress(entry.addr()),
        };

        let offset = u64::from(addr.page_offset());
        TranslationResult::Frame4KB { frame, offset }
    }
}