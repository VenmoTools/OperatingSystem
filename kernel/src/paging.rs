use system::bits::PageTableFlags;
use system::ia_32e::cpu::control::CR3;
use system::ia_32e::paging::{Frame, FrameAllocator, NotGiantPageSize, Page, Page1GB, Page2MB, Page4KB, PageIndex, PageSize, PageTable, PageTableEntry, UnusedFrame};
use system::ia_32e::paging::mapper::{Mapper, MapperFlush};
use system::ia_32e::paging::result::{CreatePageTableError, FlagUpdateError, FrameError, MapToError, TranslateError, UnmapError};
use system::ia_32e::VirtAddr;
use system::result::{Error, MemErrorKind};

pub struct RecursivePageTable<'a> {
    pml4t: Option<&'a mut PageTable>,
    index: PageIndex,
}

impl<'a> RecursivePageTable<'a> {
    pub fn empty() -> Self {
        Self {
            pml4t: None,
            index: PageIndex::empty(),
        }
    }

    pub fn init(&mut self, table: &'a mut PageTable) -> Result<(), Error> {
        // 传递的页表必须具有一个递归条目，即指向表本身的条目。
        // 引用必须使用该“循环”，即形式为“ 0o_xxx_xxx_xxx_xxx_0000”，其中“ xxx”是递归条目
        let page = Page::include_address(VirtAddr::new(table as *const _ as u64));
        let index = page.p4_index();
        if page.p3_index() != index || page.p2_index() != index || page.p1_index() != index {
            return Err(
                Error::new_memory(
                    MemErrorKind::PageTableIndexNotMatch,
                    format!("the page index not match p4:{:?} p3:{:?},p2{:?},p1:{:?}",
                            index, page.p3_index(), page.p2_index(), page.p1_index()),
                ));
        }
        let frame = CR3::read().0;
        let table_frame = table[index].frame();
        if Ok(frame) != table_frame {
            return Err(Error::new_memory(
                MemErrorKind::FrameNotMatch,
                format!("frame not match!,cr3 register frame:{:?} page table frame:{:?}", frame, table_frame),
            ));
        }
        self.index = index;
        self.pml4t = Some(table);
        Ok(())
    }

    /// 根据给定的页表创建递归页表
    /// 页表必须是有效，即CR3寄存器必须包含其物理地址。
    pub fn new(table: &'a mut PageTable) -> Result<Self, Error> {
        let mut pt = Self::empty();
        pt.init(table)?;
        Ok(pt)
    }

    fn crate_helper<'help, A>(entry: &'help mut PageTableEntry, next_page: Page, allocator: &mut A) -> Result<&'help mut PageTable, CreatePageTableError>
        where A: FrameAllocator<Page4KB> {
        let mut created = false;
        if entry.is_unused() {
            if let Some(frame) = allocator.alloc() {
                entry.set_frame(frame.frame(), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
                created = true;
            }
        }
        if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Err(CreatePageTableError::MappedToHugePage);
        }
        let ptr = next_page.start_address().as_mut_ptr();
        let pt: &mut PageTable = unsafe { &mut *(ptr) };
        if created {
            pt.zero();
        }
        Ok(pt)
    }

    /// create_next_table使用了crate_helper辅助函数,因为unsafe函数会把整个函数当做unsafe块
    /// 在unsafe块中写大量代码是不安全的
    pub unsafe fn create_next_table<'b, A>(
        entry: &'b mut PageTableEntry, next_page: Page, allocator: &mut A,
    ) -> Result<&'b mut PageTable, CreatePageTableError>
        where A: FrameAllocator<Page4KB> {
        Self::crate_helper(entry, next_page, allocator)
    }


    // 根据给定的帧和页面进行1gb页面映射
    fn map_to_1g<A>(&mut self, page: Page<Page1GB>, frame: UnusedFrame<Page1GB>, flags: PageTableFlags, allocator: &mut A)
                    -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = self.pml4t.as_mut().unwrap();
        // 根据页面创建3级页面
        let p3_page = create_p3_page(page, self.index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        // 将frame与页面做映射
        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    /// 根据给定的帧和页面进行2mb页面映射
    fn map_to_2mb<A>(&mut self, page: Page<Page2MB>, frame: UnusedFrame<Page2MB>, flags: PageTableFlags, allocator: &mut A)
                     -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = self.pml4t.as_mut().unwrap();
        // 创建3级页表
        let p3_page = create_p3_page(page, self.index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };
        // 创建2级页表
        let p2_page = create_p2_page(page, self.index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };
        // 将frame与页面做映射
        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    /// 根据给定的帧和页面进行4kb页面映射
    fn map_to_4kb<A>(&mut self, page: Page<Page4KB>, frame: UnusedFrame<Page4KB>, flags: PageTableFlags, allocator: &mut A)
                     -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>>
        where A: FrameAllocator<Page4KB> {
        let p4 = self.pml4t.as_mut().unwrap();
        // 创建3级页表
        let p3_page = create_p3_page(page, self.index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };
        // 创建2级页表
        let p2_page = create_p2_page(page, self.index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };
        // 创建1级页表
        let p1_page = create_p1_page(page, self.index);
        let p1 = unsafe { Self::create_next_table(&mut p2[page.p2_index()], p1_page, allocator)? };
        // 将frame与页面做映射
        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame.frame(), flags);
        Ok(MapperFlush::new(page))
    }
}

fn create_p3_page<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        recursive_index,
        page.p4_index(),
    )
}

fn create_p2_page<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        page.p4_index(),
        page.p3_index(),
    )
}

fn create_p1_page<S: PageSize + NotGiantPageSize>(page: Page<S>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        page.p4_index(),
        page.p3_index(),
        page.p2_index(),
    )
}

fn p3_ptr<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> *mut PageTable {
    create_p3_page(page, recursive_index).start_address().as_mut_ptr()
}

fn p2_ptr<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> *mut PageTable {
    create_p2_page(page, recursive_index).start_address().as_mut_ptr()
}

fn p1_ptr<S: PageSize + NotGiantPageSize>(page: Page<S>, recursive_index: PageIndex) -> *mut PageTable {
    create_p1_page(page, recursive_index).start_address().as_mut_ptr()
}

impl<'a> Mapper<Page1GB> for RecursivePageTable<'a> {
    fn map_to<A>(&mut self, page: Page<Page1GB>, frame: UnusedFrame<Page1GB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_1g(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page1GB>) -> Result<(Frame<Page1GB>, MapperFlush<Page1GB>), UnmapError> {
        // 获取4级页面项
        let p4 = self.pml4t.as_mut().unwrap();
        let entry = &p4[page.p3_index()];
        // 检查PML4TE 页帧的有效性主要检查 PageTableFlags::PRESENT和PageTableFlags::HUGE_PAGE标志位
        entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;
        // 根据给定的page获得3级页表
        let pt = unsafe { &mut *p3_ptr(page, self.index) };
        let entry = &mut pt[page.p3_index()];

        // 检查PDPTE的有效性
        let flags = entry.flags();
        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }
        // 检查当前PDPTE是否已经被映射
        if entry.is_unused() {
            return Err(UnmapError::PageNotMapped);
        }
        let frame = Frame::from_start_addr(entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(entry.addr()))?;
        entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page1GB>, flags: PageTableFlags) -> Result<MapperFlush<Page1GB>, FlagUpdateError> {
        let p4 = self.pml4t.as_mut().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        // 根据给定的page获得3级页表
        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        // 更新标志位
        p3[page.p3_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page1GB>) -> Result<Frame<Page1GB>, TranslateError> {
        let p4 = self.pml4t.as_mut().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        // 根据给定的page获得3级页表
        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        let entry = &p3[page.p3_index()];
        // 检测是否已经映射到物理页
        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(entry.addr()))
    }
}

impl<'a> Mapper<Page2MB> for RecursivePageTable<'a> {
    fn map_to<A>(&mut self, page: Page<Page2MB>, frame: UnusedFrame<Page2MB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_2mb(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page2MB>) -> Result<(Frame<Page2MB>, MapperFlush<Page2MB>), UnmapError> {
        // 获取并检查4级页表项
        let p4 = self.pml4t.as_mut().unwrap();
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        // 获取3级页表 获取并检查3级页表项
        let p3 = unsafe { &*p3_ptr(page, self.index) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = unsafe { &mut *p2_ptr(page, self.index) };
        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();
        //检查2级页表项标志位
        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }
        if p2_entry.is_unused() {
            return Err(UnmapError::PageNotMapped);
        }

        let frame = Frame::from_start_addr(p2_entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;
        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page2MB>, flags: PageTableFlags) -> Result<MapperFlush<Page2MB>, FlagUpdateError> {
        let p4 = self.pml4t.as_mut().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *p2_ptr(page, self.index) };
        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);
        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page2MB>) -> Result<Frame<Page2MB>, TranslateError> {
        // P4
        let p4 = self.pml4t.as_mut().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        // P3
        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        if p3[page.p3_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        // P2
        let p2 = unsafe { &mut *p2_ptr(page, self.index) };
        let entry = &p2[page.p2_index()];

        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr()).map_err(|_| {
            TranslateError::InvalidFrameAddress(entry.addr())
        })
    }
}

impl<'a> Mapper<Page4KB> for RecursivePageTable<'a> {
    fn map_to<A>(&mut self, page: Page<Page4KB>, frame: UnusedFrame<Page4KB>, flags: PageTableFlags, allocator: &mut A) -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>> where A: FrameAllocator<Page4KB>, Self: Sized {
        self.map_to_4kb(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page<Page4KB>) -> Result<(Frame<Page4KB>, MapperFlush<Page4KB>), UnmapError> {
        // 获取并检查4级页表项
        let p4 = self.pml4t.as_mut().unwrap();
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        // 获取p3页表  获取并检查3级页表项
        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        // 获取p2页表  获取并检查2级页表项
        let p2 = unsafe { &mut *p2_ptr(page, self.index) };
        let p2_entry = &p2[page.p2_index()];
        p2_entry.frame().map_err(|e| match e {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;
        // 获取p2页表
        let p1 = unsafe { &mut *p1_ptr(page, self.index) };
        let entry = &mut p1[page.p1_index()];
        // p1不会引用下层页表，因为没有p0页表不用检测
        let frame = Frame::from_start_addr(entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(entry.addr()))?;
        entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    fn update_flags(&mut self, page: Page<Page4KB>, flags: PageTableFlags) -> Result<MapperFlush<Page4KB>, FlagUpdateError> {
        let p4 = self.pml4t.as_mut().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *p3_ptr(page, self.index) };
        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *p2_ptr(page, self.index) };
        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p1 = unsafe { &mut *p1_ptr(page, self.index) };
        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page4KB>) -> Result<Frame<Page4KB>, TranslateError> {
        let p4 = self.pml4t.as_ref().unwrap();
        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        let p3 = unsafe { &*p3_ptr(page, self.index) };
        if p3[page.p3_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p2 = unsafe { &*p2_ptr(page, self.index) };
        if p2[page.p2_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p1 = unsafe { &*p1_ptr(page, self.index) };
        let entry = &p1[page.p1_index()];
        if entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }
        Frame::from_start_addr(entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(entry.addr()))
    }
}