use alloc::vec::Vec;
use core::alloc::{Alloc, AllocErr, GlobalAlloc, Layout, LayoutErr};
use core::intrinsics::copy;
use core::ptr::NonNull;
use uefi::ResultExt;
use uefi::table::boot::{AllocateType, BootServices};
use uefi::table::boot::MemoryType;

use crate::elf::{Elf, Error, GenElf, GenProgramHeader, ProgramFlags, ProgramType};

const PAGE: usize = 0x1000;

pub struct Allocator<'a> {
    bt: &'a BootServices
}

impl<'a> Allocator<'a> {
    pub fn new(bt: &'a BootServices) -> Self {
        Allocator {
            bt
        }
    }
}

pub trait OsLoaderAlloc: Alloc + MemOpt {
    fn alloc_os_mem(&mut self, layout: Layout, ty: AllocateType) -> Result<NonNull<u8>, AllocErr>;
}

pub unsafe trait MemOpt {
    unsafe fn memmove(&self, dest: *mut u8, src: *const u8, size: usize);
    unsafe fn memset(&self, buffer: *mut u8, size: usize, data: u8);
}

unsafe impl Alloc for Allocator<'_> {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let ptr = self.bt.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, layout.size()).log_warning();
        match ptr {
            Ok(ptr) => Ok(NonNull::new(ptr as *mut _).unwrap()),
            Err(e) => Err(AllocErr)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.bt.free_pages(ptr.as_ptr() as u64, layout.size());
    }
}

unsafe impl MemOpt for Allocator<'_> {
    unsafe fn memmove(&self, dest: *mut u8, src: *const u8, size: usize) {
        self.bt.memmove(dest, src, size)
    }

    unsafe fn memset(&self, buffer: *mut u8, size: usize, data: u8) {
        self.bt.memset(buffer, size, data)
    }
}

impl OsLoaderAlloc for Allocator<'_> {
    fn alloc_os_mem(&mut self, layout: Layout, ty: AllocateType) -> Result<NonNull<u8>, AllocErr> {
        let ptr = self.bt.allocate_pages(ty, MemoryType::LOADER_DATA, layout.size()).log_warning();
        match ptr {
            Ok(ptr) => Ok(NonNull::new(ptr as *mut _).unwrap()),
            Err(e) => {
                //todo:异常处理
                panic!("{:?}", e);
                Err(AllocErr)
            }
        }
    }
}


pub struct ElfLoader<'a, G: OsLoaderAlloc> {
    allocator: G,
    elf: Elf<'a>,
}

impl<'a, G: OsLoaderAlloc> ElfLoader<'a, G> {
    pub fn new(g: G, elf: Elf<'a>) -> Self {
        ElfLoader {
            elf,
            allocator: g,
        }
    }

    pub fn with_bytes(g: G, bytes: &'a [u8]) -> core::result::Result<Self, Error> {
        Ok(ElfLoader {
            elf: Elf::from_bytes(bytes)?,
            allocator: g,
        })
    }

    pub fn entry_point(&self) -> usize {
        match self.elf {
            Elf::Elf64(ref e) => e.header().entry as usize,
            Elf::Elf32(ref e) => e.header().entry as usize,
        }
    }

    /// ELF加载流程
    /// 1．确定可执行目标文件的入口地址。
    /// 2．根据入口地址找到可执行的段。
    /// 3．根据段在文件中的偏移量和大小，找到属于这个段的最后一个节区。设为A。
    /// 4．将嵌入代码添加到节区A中。
    /// 5．增加段的大小。增加值为嵌入代码的长度。
    /// 6．修改节区A的节区头部，增加节区A的大小。增加值为嵌入代码的长度。
    /// 7．修改位于节区A之后所有节区的节区头部的偏移量，增加值为嵌入代码的长度 。
    /// 8．修改ELF头部的入口地址，指向添加的代码
    pub fn load_memory(&mut self) {
        let header_layout = self.header_layout().unwrap();
//        let mut ptr = self.allocator.alloc_ty(header_layout,AllocateType::Address(),MemoryType::RUNTIME_SERVICES_CODE).unwrap();

        match self.elf {
            Elf::Elf64(ref elf) => {
                let buffer_ptr = elf.as_bytes().as_ptr();
                let mut iter = elf.program_header_iter();
                while let Some(h) = iter.next() {
                    if h.ph.ph_type() == ProgramType::LOAD {
                        // 初始化内存空间
                        unsafe {
                            self.allocator.memset(h.ph.p_paddr as *mut u8, h.ph.p_memsz as usize, 0);
                            let src = buffer_ptr.add(h.ph.p_offset as usize);
                            copy(src, h.ph.p_paddr as *mut u8, h.ph.p_filesz as usize);
                        }
                    }
                    let layout = unsafe { Layout::from_size_align_unchecked(h.ph.p_memsz as usize, h.ph.p_align as usize) };
                    // 只需要加载LOAD指令段
                    if h.ph.ph_type() == ProgramType::LOAD {
                        let mut ptr = match h.ph.flags() {
                            ProgramFlags::PF_R => self.allocator.alloc_os_mem(layout, AllocateType::Address(h.ph.p_paddr as usize)),
                            ProgramFlags::PF_X | ProgramFlags::PF_W | ProgramFlags::PF_RW => self.allocator.alloc_ty(layout, AllocateType::Address(h.ph.p_paddr as usize), MemoryType::LOADER_DATA),
                            _ => {
                                panic!("no support flags right now");
                            }
                        }.unwrap();
                        unsafe {
                            let src = buffer_ptr.add(h.ph.p_offset as usize);
                            copy(src, ptr.as_mut(), h.ph.p_filesz as usize);
                        }
                        // 如果p_memsz数据超过p_filesz需要将不足的部分用0填充
                        if h.ph.p_memsz > h.ph.p_filesz {
                            let fill_size = (h.ph.p_memsz - h.ph.p_filesz) as usize;
                            // 将指针指向buffer_ptr中的p_filesz的位置
                            let fill_ptr = src.add(h.ph.p_filesz as usize) as *mut u8;
                            unsafe { self.allocator.memset(fill_ptr, fill_size, 0) };
                        }
                    }
                }
            }
            Elf::Elf32(ref elf) => {}
        }
    }

    pub fn header_layout(&self) -> Result<Layout, LayoutErr> {
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

        Layout::from_size_align(memory, PAGE)
    }
}


fn align_down(addr: u64, align: u64) -> u64 {
    assert_eq!(align & (align - 1), 0, "`align` must be a power of two");
    addr & !(align - 1)
}

/// 跟读
fn align_up(addr: u64, align: u64) -> u64 {
    assert_eq!(align & (align - 1), 0, "`align` must be a power of two");

    let mask = align - 1;
    if addr & mask == 0 {
        addr
    } else {
        (addr | mask) + 1
    }
}