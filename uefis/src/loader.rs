use bitflags::_core::ptr::slice_from_raw_parts_mut;
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
use system::ia_32e::paging::mapper::PageTableOffset;
use uefi::prelude::BootServices;
use uefi::ResultExt;
use uefi::table::boot::{AllocateType, MemoryType};
use xmas_elf::ElfFile;
use xmas_elf::program::{ProgramHeader64, Type};

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
    pub kernel_start_paddr: PhysAddr,
    pub kernel_size: u64,
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

    pub fn load_kernel<S: PageSize>(&mut self, args: &KernelArgs, pt: &mut PageTableOffset<'a>)
                                    -> core::result::Result<(), MapToError<S>> {
        match self.elf {
            Elf::Elf64(ref mut elf) => {
                for ref segment in elf.program_header_iter() {
                    if segment.ph.ph_type() == ProgramType::LOAD {
                        let phy_start_addr = args.kernel_start_paddr + segment.ph.p_offset;
                        let v_addr = VirtAddr::new(segment.ph.p_vaddr);
                        // 获取内核起始页面
                        let start_page: Page = Page::include_address(v_addr);
                        info!("start page:{:?}", start_page);
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
                                pt.map_to(page, UnusedFrame::new(frame), page_flags, &mut self.allocator).unwrap()
                            }.flush();
                            // todo: 需要填充空余的内存
                        }
                    }
                }
                let stack_start = stack + 1;
                let stack_end = stack + stack_size;
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                for page in Page::range_page(stack_start, stack_end) {
                    let frame = self.allocator.alloc().ok_or(MapToError::FrameAllocateFailed)?;
                    pt.map_to(page, frame, flags, &mut self.allocator);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn load(bt: &BootServices, elf: &ElfFile, header: &ProgramHeader64) {
        if header.get_type().unwrap() == Type::Load {
            let mut dest = (header.virtual_addr() & !0x0fff) as usize;

            let page_num = {
                let padding = header.virtual_addr() & 0x0fff;
                let total = header.mem_size() + padding;
                (1 + (total >> 12)) as usize
            };
            bt.allocate_pages(AllocateType::Address(dest), MemoryType::LOADER_CODE, page_num);
            unsafe { bt.memset(dest as *mut u8, page_num * 4096, 0) };
            let mut buf = unsafe { slice_from_raw_parts_mut(header.virtual_addr() as *mut u8, header.mem_size() as usize) };
            buf.copy_from_slice(header.raw_data(elf));
        }
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
