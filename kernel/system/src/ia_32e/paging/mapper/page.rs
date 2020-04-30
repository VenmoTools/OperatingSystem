use crate::bits::PageTableFlags;
use crate::ia_32e::cpu::control::CR3;
use crate::ia_32e::paging::{Frame, FrameAllocator, NotGiantPageSize, Page, Page1GB, Page2MB, Page4KB, PageIndex, PageSize, PageTable, PageTableEntry, UnusedFrame};
use crate::ia_32e::paging::mapper::{MapAllSize, Mapper, MapperFlush};
use crate::ia_32e::paging::result::{FlagUpdateError, FrameError, MapToError, TranslateError, TranslationResult, UnmapError};
use crate::ia_32e::VirtAddr;

#[derive(Debug)]
pub struct RecursivePageTable<'a> {
    p4: &'a mut PageTable,
    recursive_index: PageIndex,
}

impl<'a> RecursivePageTable<'a> {
    /// Creates a new RecursivePageTable from the passed level 4 PageTable.
    ///
    /// The page table must be recursively mapped, that means:
    ///
    /// - The page table must have one recursive entry, i.e. an entry that points to the table
    ///   itself.
    ///     - The reference must use that “loop”, i.e. be of the form `0o_xxx_xxx_xxx_xxx_0000`
    ///       where `xxx` is the recursive entry.
    /// - The page table must be active, i.e. the CR3 register must contain its physical address.
    ///
    /// Otherwise `Err(())` is returned.
    #[inline]
    pub fn new(table: &'a mut PageTable) -> Result<Self, ()> {
        let page = Page::include_address(VirtAddr::new(table as *const _ as u64));
        let recursive_index = page.p4_index();

        if page.p3_index() != recursive_index
            || page.p2_index() != recursive_index
            || page.p1_index() != recursive_index
        {
            return Err(());
        }
        if Ok(CR3::read().0) != table[recursive_index].frame() {
            return Err(());
        }

        Ok(RecursivePageTable {
            p4: table,
            recursive_index,
        })
    }

    /// Creates a new RecursivePageTable without performing any checks.
    ///
    /// ## Safety
    ///
    /// The `recursive_index` parameter must be the index of the recursively mapped entry.
    #[inline]
    pub unsafe fn new_unchecked(table: &'a mut PageTable, recursive_index: PageIndex) -> Self {
        RecursivePageTable {
            p4: table,
            recursive_index,
        }
    }

    /// Internal helper function to create the page table of the next level if needed.
    ///
    /// If the passed entry is unused, a new frame is allocated from the given allocator, zeroed,
    /// and the entry is updated to that address. If the passed entry is already mapped, the next
    /// table is returned directly.
    ///
    /// The `next_page_table` page must be the page of the next page table in the hierarchy.
    ///
    /// Returns `MapToError::FrameAllocationFailed` if the entry is unused and the allocator
    /// returned `None`. Returns `MapToError::ParentEntryHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    unsafe fn create_next_table<'b, A, S: PageSize>(
        entry: &'b mut PageTableEntry,
        next_table_page: Page,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, MapToError<S>>
        where
            A: FrameAllocator<Page4KB>,
    {
        /// This inner function is used to limit the scope of `unsafe`.
        ///
        /// This is a safe function, so we need to use `unsafe` blocks when we do something unsafe.
        fn inner<'b, A, S: PageSize>(
            entry: &'b mut PageTableEntry,
            next_table_page: Page,
            allocator: &mut A,
        ) -> Result<&'b mut PageTable, MapToError<S>>
            where
                A: FrameAllocator<Page4KB>,
        {
            use crate::bits::flags::PageTableFlags as Flags;
            let created;

            if entry.is_unused() {
                if let Some(frame) = allocator.alloc() {
                    entry.set_frame(frame.frame(), Flags::PRESENT | Flags::WRITABLE);
                    created = true;
                } else {
                    return Err(MapToError::FrameAllocateFailed);
                }
            } else {
                created = false;
            }
            if entry.flags().contains(Flags::HUGE_PAGE) {
                return Err(MapToError::ParentEntryHugePage);
            }

            let page_table_ptr = next_table_page.start_address().as_mut_ptr();
            let page_table: &mut PageTable = unsafe { &mut *(page_table_ptr) };
            if created {
                page_table.zero();
            }
            Ok(page_table)
        }

        inner(entry, next_table_page, allocator)
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_1gib<A>(
        &mut self,
        page: Page<Page1GB>,
        frame: Frame<Page1GB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        use crate::bits::flags::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(unsafe { UnusedFrame::new(frame) }));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_2mib<A>(
        &mut self,
        page: Page<Page2MB>,
        frame: Frame<Page2MB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        use crate::bits::flags::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(unsafe { UnusedFrame::new(frame) }));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_4kib<A>(
        &mut self,
        page: Page<Page4KB>,
        frame: Frame<Page4KB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };

        let p1_page = p1_page(page, self.recursive_index);
        let p1 = unsafe { Self::create_next_table(&mut p2[page.p2_index()], p1_page, allocator)? };

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(unsafe { UnusedFrame::new(frame) }));
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }
}

impl<'a> Mapper<Page1GB> for RecursivePageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Page1GB>,
        frame: Frame<Page1GB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page1GB>, MapToError<Page1GB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        self.map_to_1gib(page, frame, flags, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Page1GB>,
    ) -> Result<(Frame<Page1GB>, MapperFlush<Page1GB>), UnmapError> {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];

        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_addr(p3_entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;

        p3_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    // allow unused_unsafe until https://github.com/rust-lang/rfcs/pull/2585 lands
    #[allow(unused_unsafe)]
    unsafe fn update_flags(
        &mut self,
        page: Page<Page1GB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Page1GB>, FlagUpdateError> {
        use crate::bits::flags::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page1GB>) -> Result<Frame<Page1GB>, TranslateError> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        Frame::from_start_addr(p3_entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
}

impl<'a> Mapper<Page2MB> for RecursivePageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Page2MB>,
        frame: Frame<Page2MB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page2MB>, MapToError<Page2MB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        self.map_to_2mib(page, frame, flags, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Page2MB>,
    ) -> Result<(Frame<Page2MB>, MapperFlush<Page2MB>), UnmapError> {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = Frame::from_start_addr(p2_entry.addr())
            .map_err(|_| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;

        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    // allow unused_unsafe until https://github.com/rust-lang/rfcs/pull/2585 lands
    #[allow(unused_unsafe)]
    unsafe fn update_flags(
        &mut self,
        page: Page<Page2MB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Page2MB>, FlagUpdateError> {
        use crate::bits::flags::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page2MB>) -> Result<Frame<Page2MB>, TranslateError> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p2 = unsafe { &*(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        Frame::from_start_addr(p2_entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
}

impl<'a> Mapper<Page4KB> for RecursivePageTable<'a> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Page4KB>,
        frame: Frame<Page4KB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Page4KB>, MapToError<Page4KB>>
        where
            A: FrameAllocator<Page4KB>,
    {
        self.map_to_4kib(page, frame, flags, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Page4KB>,
    ) -> Result<(Frame<Page4KB>, MapperFlush<Page4KB>), UnmapError> {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page.clone(), self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];
        p2_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p1 = unsafe { &mut *(p1_ptr(page.clone(), self.recursive_index)) };
        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    // allow unused_unsafe until https://github.com/rust-lang/rfcs/pull/2585 lands
    #[allow(unused_unsafe)]
    unsafe fn update_flags(
        &mut self,
        page: Page<Page4KB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Page4KB>, FlagUpdateError> {
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page.clone(), self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *(p2_ptr(page.clone(), self.recursive_index)) };

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p1 = unsafe { &mut *(p1_ptr(page.clone(), self.recursive_index)) };

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&mut self, page: Page<Page4KB>) -> Result<Frame<Page4KB>, TranslateError> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p2 = unsafe { &*(p2_ptr(page.clone(), self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p1 = unsafe { &*(p1_ptr(page.clone(), self.recursive_index)) };
        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        Frame::from_start_addr(p1_entry.addr())
            .map_err(|_| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
}

impl<'a> MapAllSize for RecursivePageTable<'a> {
    #[allow(clippy::inconsistent_digit_grouping)]
    fn translate(&self, addr: VirtAddr) -> TranslationResult {
        let page = Page::include_address(addr);

        let p4 = &self.p4;
        let p4_entry = &p4[addr.page4_index()];
        if p4_entry.is_unused() {
            return TranslationResult::PageNotMapped;
        }
        if p4_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            panic!("level 4 entry has huge page bit set")
        }

        let p3 = unsafe { &*(p3_ptr(page.clone(), self.recursive_index)) };
        let p3_entry = &p3[addr.page3_index()];
        if p3_entry.is_unused() {
            return TranslationResult::PageNotMapped;
        }
        if p3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            let frame = Frame::include_address(p3[addr.page3_index()].addr());
            let offset = addr.as_u64() & 0o_777_777_7777;
            return TranslationResult::Frame1GB { frame, offset };
        }

        let p2 = unsafe { &*(p2_ptr(page.clone(), self.recursive_index)) };
        let p2_entry = &p2[addr.page2_index()];
        if p2_entry.is_unused() {
            return TranslationResult::PageNotMapped;
        }
        if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            let frame = Frame::include_address(p2[addr.page2_index()].addr());
            let offset = addr.as_u64() & 0o_777_7777;
            return TranslationResult::Frame4KB { frame, offset };
        }

        let p1 = unsafe { &*(p1_ptr(page, self.recursive_index)) };
        let p1_entry = &p1[addr.page1_index()];
        if p1_entry.is_unused() {
            return TranslationResult::PageNotMapped;
        }
        if p1_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            panic!("level 1 entry has huge page bit set")
        }
        let frame = Frame::include_address(p1_entry.addr());
        let offset = u64::from(addr.page_offset());
        TranslationResult::Frame4KB { frame, offset }
    }
}

#[inline]
fn p3_ptr<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> *mut PageTable {
    p3_page(page, recursive_index).start_address().as_mut_ptr()
}

#[inline]
fn p3_page<S: PageSize>(page: Page<S>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        recursive_index,
        page.p4_index(),
    )
}

#[inline]
fn p2_ptr<S: NotGiantPageSize>(page: Page<S>, recursive_index: PageIndex) -> *mut PageTable {
    p2_page(page, recursive_index).start_address().as_mut_ptr()
}

#[inline]
fn p2_page<S: NotGiantPageSize>(page: Page<S>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        page.p4_index(),
        page.p3_index(),
    )
}

#[inline]
fn p1_ptr(page: Page<Page4KB>, recursive_index: PageIndex) -> *mut PageTable {
    p1_page(page, recursive_index).start_address().as_mut_ptr()
}

#[inline]
fn p1_page(page: Page<Page4KB>, recursive_index: PageIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        page.p4_index(),
        page.p3_index(),
        page.p2_index(),
    )
}


