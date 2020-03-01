use core::{
    alloc::{Layout, LayoutErr},
    marker::PhantomData,
};
use system::bits::{EferFlags, PageTableFlags};
use system::ia_32e::{
    cpu::msr, instructions::page_table::flush_all,
    paging::{
        Frame,
        FrameAllocator,
        mapper::Mapper, Page, Page2MB, Page4KB, PageIndex, PageSize, PageTable, PageTableEntry, result::MapToError, UnusedFrame,
    },
    PhysAddr,
    VirtAddr,
};
use uefi::prelude::BootServices;
use uefi::ResultExt;
use uefi::table::boot::{AllocateType, MemoryType};

use crate::elf::{Elf, Elf64, Error, GenElf, GenProgramHeader, ProgramFlags, ProgramHeader, ProgramType};
use crate::paging::RecursivePageTable;
use crate::result::Result;

pub struct BootAllocator<'a, S: PageSize> {
    bt: &'a BootServices,
    _mark: PhantomData<S>,
}

impl<'a, S: PageSize> BootAllocator<'a, S> {
    const PAGE_SIZE: u64 = S::P_SIZE;
    pub fn new(bt: &'a BootServices) -> Self {
        BootAllocator {
            bt,
            _mark: PhantomData,
        }
    }
}

unsafe impl<'a, S: PageSize> FrameAllocator<S> for BootAllocator<'a, S> {
    fn alloc(&mut self) -> Option<UnusedFrame<S>> {
        let start_ptr = match self.bt.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1).log_warning() {
            Ok(ptr) => ptr,
            Err(e) => return None,
        };
        let frame = match Frame::from_start_addr(PhysAddr::new(start_ptr)) {
            Ok(frame) => frame,
            Err(_) => return None,
        };
        unsafe { Some(UnusedFrame::new(frame)) }
    }

    fn dealloc(&mut self, frame: UnusedFrame<S>) {
        let addr = frame.start_address();
        self.bt.free_pages(addr.as_u64(), frame.size() as usize);
    }
}


pub struct OsLoader<'a, F: FrameAllocator<Page4KB>> {
    allocator: F,
    elf: Elf<'a>,
}

pub struct KernelArgs {
    p4_addr: PhysAddr,
    kernel_start_vaddr: VirtAddr,
    kernel_start_paddr: PhysAddr,
    kernel_size: u64,
}

impl<'a, F: FrameAllocator<Page4KB>> OsLoader<'a, F> {
    pub fn new(g: F, elf: Elf<'a>) -> Self {
        OsLoader {
            elf,
            allocator: g,
        }
    }

    pub fn with_bytes(g: F, bytes: &'a [u8]) -> core::result::Result<Self, Error> {
        Ok(OsLoader {
            elf: Elf::from_bytes(bytes)?,
            allocator: g,
        })
    }

    fn load_segment<S: PageSize>(&mut self, segment: &ProgramHeader<Elf64>, args: &KernelArgs, pt: &mut RecursivePageTable<'a>)
                                 -> core::result::Result<(), MapToError<S>> {
        match segment.ph.ph_type() {
            ProgramType::LOAD => {
                let phy_start_addr = args.kernel_start_paddr + segment.ph.p_offset;
                let v_addr = VirtAddr::new(segment.ph.p_vaddr);
                // 获取内核起始页面
                let start_page: Page = Page::include_address(v_addr);
                let start_frame: Frame = Frame::include_address(phy_start_addr);
                let end_frame = Frame::include_address(phy_start_addr + segment.ph.p_filesz - 1_u64);

                // 根据elf flags创建页面flags
                let flags = segment.ph.flags();
                let mut page_flags = PageTableFlags::PRESENT;
                if flags != ProgramFlags::PF_Execute {
                    page_flags |= PageTableFlags::NO_EXECUTE;
                }
                if flags == ProgramFlags::PF_Write {
                    page_flags |= PageTableFlags::WRITABLE;
                }

                for frame in Frame::frame_range_include(start_frame, end_frame) {
                    // 计算距离起始地址的偏移
                    let offset = frame - start_frame;
                    // 计算起始地址偏移后的页面
                    let page = start_page + offset;
                    unsafe {
                        pt.map_to(page, UnusedFrame::new(frame), page_flags, &mut self.allocator)?
                    }.flush();
                    // todo: 需要填充空余的内存
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }


    fn load_kernel<S: PageSize>(&mut self, stack: Page, stack_size: u64, args: &KernelArgs, pt: &mut RecursivePageTable<'a>)
                                -> core::result::Result<(), MapToError<S>> {
        match self.elf {
            Elf::Elf64(ref elf) => {
                for ref seg in elf.program_header_iter() {
                    self.load_segment::<S>(seg, args, pt);
                }
                let stack_start = stack + 1;
                let stack_end = stack + stack_size;
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                for page in Page::range_page(stack_start, stack_end) {
                    let frame = self.allocator.alloc().ok_or(MapToError::FrameAllocateFailed)?;
                    unsafe {
                        pt.map_to(page, frame, flags, &mut self.allocator)
                    };
                }
                //todo: What next?
            }
            _ => {}
        }
        Ok(())
    }

    fn mark_used_address(&self, marker: &mut [bool]) {
        match self.elf {
            Elf::Elf64(ref elf) => {
                let mut iter = elf.program_header_iter();
                while let Some(header) = iter.next() {
                    let start_page: Page = Page::include_address(VirtAddr::new(header.ph.p_vaddr));
                    let end_page: Page = Page::include_address(VirtAddr::new(header.ph.p_vaddr + header.ph.p_memsz));

                    let start_index = u64::from(start_page.p4_index());
                    let end_index = u64::from(end_page.p4_index());

                    for page_index in start_index..=end_index {
                        marker[page_index as usize] = true;
                    }
                }
            }
            Elf::Elf32(ref elf) => {}
        }
    }

    fn enable_nx() {
        // enable no execute
        let mut msr_flag = msr::Efer::read();
        // already in long mode
        msr_flag |= EferFlags::NO_EXECUTE_ENABLE;
        unsafe { msr::Efer::write(msr_flag) };
    }

    fn get_free_entry(marker: &mut [bool]) -> PageIndex {
        let (index, entry) = marker.iter_mut().enumerate()
            .find(|(_, &mut entry)| { entry == false })
            .expect("not PML4E found!");
        *entry = true;
        PageIndex::new(index as u16)
    }

    pub fn load(&self, args: KernelArgs) -> Result<()> {
        //标记使用过的虚拟地址
        let mut mark_use_page = [false; 512];
        self.mark_used_address(&mut mark_use_page);
        // 页表中启用 no-execute
        Self::enable_nx();
        let index = Self::get_free_entry(&mut mark_use_page);
        // 创建页表所需的页表项
        let mut entry = PageTableEntry::new();
        entry.set_addr(args.p4_addr, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
        // 写入创建的页表项
        let page_table = unsafe { &mut *(args.p4_addr.as_u64() as *mut PageTable) };
        page_table[index] = entry;
        unsafe { flush_all() };
        // 创建递归页表
        let table_addr = Page::from_page_table_indices(
            index,
            index,
            index,
            index,
        ).start_address();
        let table = unsafe { &mut *(table_addr.as_mut_ptr()) };
        let mut page_table = RecursivePageTable::new(table)?;

        let kernel_start_page: Page<Page2MB> = Page::include_address(args.kernel_start_vaddr);
        let kernel_end_page = Page::include_address(args.kernel_start_vaddr + args.kernel_size - 1_u64);
        for page in Page::range_include(kernel_start_page, kernel_end_page) {
            page_table.unmap(page)?.1.flush();
        }

        Ok(())
    }

    pub fn header_layout(&self) -> core::result::Result<Layout, LayoutErr> {
        macro_rules! max_memory {
            ($ex:expr) => {
                $ex.program_header_iter().map(|x| x.ph.p_vaddr + x.ph.p_memsz).max().unwrap();
            };
        }
        // 获取程序头中
        let memory = match self.elf {
            Elf::Elf64(ref e) => max_memory!(e) as usize,
            Elf::Elf32(ref e) => max_memory!(e) as usize,
        };

        Layout::from_size_align(memory, 4096)
    }
}
