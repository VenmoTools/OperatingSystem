#![no_std]
#![no_main]
#![feature(asm)]
#![feature(slice_patterns)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(fn_traits)]
#![allow(dead_code)]
#![allow(unused_must_use)]
#![feature(slice_from_raw_parts)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use alloc::vec::Vec;
use result::{ok, Result, UefiResult};
use system::ia_32e::cpu::control::{CR0, CR3, CR4};
use system::ia_32e::cpu::msr::Efer;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{AllocateType, MemoryMapIter, MemoryMapKey, MemoryType};
use uefi::table::Runtime;
use xmas_elf::ElfFile;
use xmas_elf::program::{ProgramHeader, ProgramHeader64, Type};

use crate::fs::Read;

mod result;
mod fs;
mod paging;
// mod elf;
// mod loader;

pub fn check_cpu() {
    let cpuid = raw_cpuid::CpuId::new();
    let c_info = cpuid.get_vendor_info().unwrap();
    info!("CPU Vendor is: {}", c_info.as_string());
    let infos = cpuid.get_extended_function_info().unwrap();
    info!("Cpu Type is {}", infos.processor_brand_string().unwrap());
    if infos.has_1gib_pages() {
        info!("CPU support 1GB pages")
    }
    if infos.has_64bit_mode() {
        info!("already in long mode");
    }
    info!("physical address bits {}", infos.physical_address_bits().unwrap());
}

pub fn paging_check() {
    if CR0::is_enable_protected_mode() {
        info!("system in protected mode");
    }
    if CR0::is_enable_paging() {
        info!("system enable paging");
        if CR4::is_enable_PAE() && Efer::enable_long_mode() {
            info!("system now in 4-level paging");
        } else if CR4::is_enable_PAE() {
            info!("system now in 32-bit paging");
        } else if CR4::is_enable_PAE() && !Efer::enable_long_mode() {
            info!("system now in PAE paging");
        }
        let (frame, _) = CR3::read();
        info!("base table physical start address:{:?}", frame);
    }
}


#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    if let Err(e) = uefi_services::init(&st).log_warning() {
        info!("{:?}", e);
        return e.status();
    }
    check_cpu();
    paging_check();

    let bt = st.boot_services();
    // 2. 加载内核文件
    let mut f = fs::File::new(bt);
    let mut reader = f.open(r"EFI\Boot\kernel", "r").log_warning().unwrap().unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf);

    let elf = ElfFile::new(buf.as_slice()).unwrap();
    let entry = load_elf(bt, &elf);
    info!("entry address {:#X}", entry as u64);
    let outpu = bt.locate_protocol::<GraphicsOutput>().log_warning().unwrap();
    let display = unsafe { &mut *outpu.get() };

    switch_context(image, st, entry)
}


fn switch_context(image: uefi::Handle, st: SystemTable<Boot>, f: fn(SystemTable<Runtime>, MemoryMapIter) -> !) -> ! {
    let mmap_size = st.boot_services().memory_map_size();
    let mut mmp = vec![0; mmap_size].into_boxed_slice();
    info!("exit boot services...");
    if let Err(e) = st.stdout().reset(false).log_warning() {
        info!("{:?}", e);
        loop {}
    }
    let (st, iter) = st.exit_boot_services(image, &mut mmp).log_warning().unwrap();
    f(st, iter)
}

fn map_memory_layout<F: FnMut(&MemoryMapKey, &mut MemoryMapIter)>(bt: &BootServices, mut f: F) -> UefiResult<Result<()>> {
    let size = bt.memory_map_size();
    let buffer = bt.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size).log_warning()?;
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, size) };
    let (map, mut iter) = bt.memory_map(buffer).log_warning()?;
    f(&map, &mut iter);
    ok(())
}

pub fn load_elf(bt: &BootServices, elf: &ElfFile) -> fn(SystemTable<Runtime>, MemoryMapIter) -> ! {
    for header in elf.program_iter() {
        match header {
            ProgramHeader::Ph64(h) => { load(bt, elf, h) }
            _ => {}
        }
    }
    unsafe { core::mem::transmute(elf.header.pt2.entry_point()) }
}

fn load(bt: &BootServices, elf: &ElfFile, header: &ProgramHeader64) {
    if header.get_type().unwrap() == Type::Load {
        let dest = (header.virtual_addr & !0x0fff) as usize;

        let page_num = {
            let padding = header.virtual_addr & 0x0fff;
            let total = header.mem_size + padding;
            (1 + (total >> 12)) as usize
        };
        assert_eq!(dest as u64, header.virtual_addr & !0x0fff);

        bt.allocate_pages(AllocateType::Address(dest), MemoryType::LOADER_CODE, page_num);

        unsafe { bt.memset(dest as *mut u8, page_num * 4096, 0) };
        let buf = unsafe { core::slice::from_raw_parts_mut(header.virtual_addr as *mut u8, header.mem_size as usize) };
        let data = header.raw_data(elf);
        info!("data: {} buf: {}", data.len(), buf.len());
        if data.len() == buf.len() {
            buf.copy_from_slice(data);
        }
    }
}
