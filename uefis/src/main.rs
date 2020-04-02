#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(fn_traits)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use alloc::vec::Vec;
use result::{ok, Result, UefiResult};
use system::ia_32e::cpu::control::{CR0, CR3, CR4};
use system::ia_32e::cpu::apic::Efer;
use system::KernelArgs;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{AllocateType, MemoryMapIter, MemoryMapKey, MemoryType};
use xmas_elf::ElfFile;
use xmas_elf::program::{ProgramHeader, ProgramHeader64, Type};

use crate::fs::Read;

mod result;
mod fs;
#[macro_use]
mod serial;

static mut KERNEL_START: u64 = 0;
static mut KERNEL_END: u64 = 0;


type KernelEnterPoint = fn(u64) -> !;


#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    // UEFI服务初始化
    init(&st);
    let bt = st.boot_services();
    // 显示器分辨率设置
    let output = bt.locate_protocol::<GraphicsOutput>().log_warning().unwrap();
    let gop = unsafe { &mut *output.get() };
    switch_display_mode(gop, (1024, 768));
    // CPU检测
    check_cpu();
    // 分页检测
    paging_check();
    // 加载内核
    let entry = load_kernel(bt);
    // 跳转内核
    switch_context(image, gop, st, entry)
}

fn reset_console(st: &SystemTable<Boot>) {
    if let Err(e) = st.stdout().clear().log_warning() {
        info!("{:?}", e);
        loop {}
    }
}

macro_rules! set_stack_pointer {
    () => {
        unsafe {
            asm!(
                "mov rax,0x200000;\
                mov rsp,rax;"
                : : : : "intel", "volatile"
            )
        }
    };
}

fn switch_context(image: uefi::Handle, gop: &mut GraphicsOutput, st: SystemTable<Boot>, f: KernelEnterPoint) -> ! {
    let mmap_size = st.boot_services().memory_map_size();
    let ptr = st.boot_services().allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, mmap_size).log_warning().unwrap();
    let mmp = unsafe { core::slice::from_raw_parts_mut(ptr, mmap_size) };
    let mut frame = gop.frame_buffer();
    info!("exit boot services...");
    reset_console(&st);
    let (ref st, ref mut iter) = st.exit_boot_services(image, mmp).log_warning().unwrap();
    let args = &KernelArgs {
        st:  st as *const _ as u64,
        iter: iter as *mut _ as u64,
        kernel_start: unsafe { KERNEL_START },
        kernel_end: unsafe { KERNEL_END },
        stack_start: 0x200000,
        stack_end: 0x100000, // total 256 pages
        frame_ptr: frame.as_mut_ptr(),
        frame_size: frame.size(),
    };
    let ptr = args as *const _ as u64;
    println!("uefi:{}",ptr);
    set_stack_pointer!();
    f(ptr)
}

fn map_memory_layout<F: FnMut(&MemoryMapKey, &mut MemoryMapIter)>(bt: &BootServices, mut f: F) -> UefiResult<Result<()>> {
    let size = bt.memory_map_size();
    let buffer = bt.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size).log_warning()?;
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, size) };
    let (map, mut iter) = bt.memory_map(buffer).log_warning()?;
    f(&map, &mut iter);
    ok(())
}

pub fn load_elf<'a>(bt: &BootServices, elf: &ElfFile) -> KernelEnterPoint {
    let kernel_end = elf.program_iter().filter(|h| match h {
        ProgramHeader::Ph64(h) => h.get_type().unwrap() == Type::Load,
        _ => { false }
    }).map(|h| h.virtual_addr()).max();

    let kernel_start = elf.program_iter().filter(|h| match h {
        ProgramHeader::Ph64(h) => h.get_type().unwrap() == Type::Load,
        _ => { false }
    }).map(|h| h.virtual_addr()).min();
    info!("start: {:X}", kernel_start.unwrap());
    info!("end: {:X}", kernel_end.unwrap());
    unsafe {
        KERNEL_START = kernel_start.unwrap();
        KERNEL_END = kernel_end.unwrap();
    }
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
        // BUG! when use system crate not use 'call' features it panic!
        if let Err(e) = bt.allocate_pages(AllocateType::Address(dest), MemoryType::LOADER_CODE, page_num).log_warning(){
            panic!("occur when allocate kernel memory{:?}",e);
        }

        unsafe { bt.memset(dest as *mut u8, page_num * 4096, 0) };
        let buf = unsafe { core::slice::from_raw_parts_mut(header.virtual_addr as *mut u8, header.mem_size as usize) };
        let data = header.raw_data(elf);
        if data.len() == buf.len() {
            buf.copy_from_slice(data);
        }
    }
}

fn check_cpu() {
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

fn paging_check() {
    if CR0::is_enable_protected_mode() {
        info!("system in protected mode");
    }
    if CR0::is_enable_paging() {
        info!("system enable paging");
        if CR4::is_enable_PAE() && Efer::enable_long_mode() {
            info!("system now in 4-level paging");
        } else if CR4::is_enable_PAE() && !Efer::enable_long_mode() {
            info!("system now in 32-bit paging");
        } else if CR4::is_enable_PAE() {
            info!("system now in PAE paging");
        } else {
            info!("not paging");
        }
        let (frame, _) = CR3::read();
        info!("base table physical start address:{:?}", frame);
    }
}

fn init(st: &SystemTable<Boot>) {
    if let Err(e) = uefi_services::init(&st).log_warning() {
        info!("{:?}", e);
    }
}

fn switch_display_mode(gop: &mut GraphicsOutput, display_mode: (usize, usize)) {
    let mode = gop.modes()
        .map(|m| m.unwrap())
        .find(|ref m| {
            let info = m.info();
            info.resolution() == display_mode
        }).unwrap();
    info!("mode size: {:?}", mode.info_size());
    info!("mode info: {:?}", mode.info());
    gop.set_mode(&mode).log_warning().unwrap();
    info!("{:?}", gop.current_mode_info());
}

fn load_kernel(bt: &BootServices) -> KernelEnterPoint {
    let mut f = fs::File::new(bt);
    let mut reader = f.open(r"EFI\Boot\kernel", "r").log_warning().unwrap().unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();
    let elf = ElfFile::new(buf.as_slice()).unwrap();
    load_elf(bt, &elf)
}