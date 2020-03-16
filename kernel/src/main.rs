#![feature(asm)]
#![no_std]
#![no_main]
// #![deny(warnings)]
// #[cfg(feature = "bios")]
// #[macro_use]
// extern crate kernel;
#[cfg(feature = "efi")]
#[macro_use]
extern crate kernel;

use system::bits::flags::PageTableFlags;
use system::ia_32e::{PhysAddr, VirtAddr};
use system::ia_32e::descriptor::GlobalDescriptorTable;
use system::ia_32e::paging::{Frame, Page, Page2MB, PageTable, UnusedFrame};
use system::ia_32e::paging::mapper::Mapper;
use system::KernelArgs;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::table::{Runtime, SystemTable};
use uefi::table::boot::MemoryMapIter;

use kernel::{Initializer, loop_hlt};
use kernel::graphic::frame::RawFrameBuffer;
use kernel::memory::frame::PhysicalAllocator;
use kernel::paging::RecursivePageTable;

struct Writer<'a> {
    raw: RawFrameBuffer<'a>,
    line: usize,
    col: usize,
}

impl<'a> Writer<'a> {
    pub fn new(frame: RawFrameBuffer<'a>) -> Self {
        Self {
            raw: frame,
            line: 0,
            col: 0,
        }
    }

    fn line_(&self) -> usize {
        self.line * 1024 * 20
    }

    pub fn write<T: Clone + Copy>(&mut self, value: T) {
        let next = self.line_() + self.col;
        assert!(next <= self.raw.max_size());
        for i in self.line_()..next {
            unsafe { self.raw.write_value(i, value) };
        }
        if self.col >= 1024 {
            self.col = 0;
        } else {
            self.col += 20;
        }
    }

    pub fn write_line<T: Clone + Copy>(&mut self, value: T) {
        self.col = 20;
        self.write(value);
        self.line += 1;
    }
}

fn get_mem_iter(args: &KernelArgs) -> &mut MemoryMapIter {
    unsafe { &mut *(args.iter as *mut MemoryMapIter) }
}

fn get_system_table(args: &KernelArgs) -> &SystemTable<Runtime> {
    unsafe { &*(args.st as *const SystemTable<Runtime>) }
}

#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(args: u64) -> ! {
    println!("ptr:{:?}", args);
    //BUG: 不知为何传递的指针参数会偏移
    let args = unsafe { &*((132806768) as *const KernelArgs) };
    // Crate Page table
    let mut pt = PageTable::new();
    let mut page_table = match RecursivePageTable::new(&mut pt) {
        Ok(pt) => pt,
        Err(e) => panic!("{:?}", e)
    };
    // map kernel memory
    let iter = get_mem_iter(args);
    let mut allocator = PhysicalAllocator::new(iter);
    kernel::memory::init(&mut allocator, args);

    Initializer::initialize_all();
    loop_hlt()
}

#[cfg(feature = "bios")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    let cpuid = raw_cpuid::CpuId::new();
    println!("cpu info:{:?}", cpuid.get_vendor_info().unwrap().as_string());
    Initializer::initialize_all();
    loop_hlt()
}


