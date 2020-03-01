#![no_std]
#![no_main]
#![feature(asm)]
#![feature(slice_patterns)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(fn_traits)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use alloc::vec::Vec;
use result::{ok, Result, UefiResult};
use system::ia_32e::paging::{enable_paging_for_uefi, PagingArgs};
use uefi::prelude::*;
use uefi::table::boot::{MemoryMapIter, MemoryMapKey, MemoryType};
use uefi::table::Runtime;

use crate::fs::Read;
use crate::loader::{BootAllocator, OsLoader};

mod result;
mod fs;
mod paging;
mod elf;
mod loader;

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    if let Err(e) = uefi_services::init(&st).log_warning() {
        info!("{:?}", e);
        return e.status();
    }
    // 1. 开启分页
    //todo: BUG!
    let args = PagingArgs {
        pml4t_base_addr: 0x70000,
        pdpt_base_addr: 0x71000,
        pd_base_addr: 0x72000,
    };
    unsafe { enable_paging_for_uefi(args) };
    let _res = st.stdout().reset(false);
    let bt = st.boot_services();
    // 2. 加载内核文件
    let mut f = fs::File::new(bt);
    let mut reader = f.open(r"EFI\Boot\kernel", "r").log_warning().unwrap().unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf);
    info!("{}", buf.len());
    // 3. 解析内核文件
    let allocator = BootAllocator::new(bt);
    let os_loader = OsLoader::with_bytes(allocator, buf.as_slice()).unwrap();
    // 4. 映射UEFI内存布局(分页模式)
    // 6. 启动内核
    switch_context(image, st)
}

fn switch_context(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let mmap_size = st.boot_services().memory_map_size();
    let mut mmp = vec![0; mmap_size].into_boxed_slice();
    let (st, iter) = st.exit_boot_services(image, &mut mmp).log_warning().unwrap();
    //TODO:map memory
    info!("exit boot services...");
    loop {}
}

fn map_memory_layout<F: FnMut(&MemoryMapKey, &mut MemoryMapIter)>(bt: &BootServices, f: F) -> UefiResult<Result<()>> {
    let size = bt.memory_map_size();
    let buffer = bt.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size).log_warning()?;
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, size) };
    let (map, mut iter) = bt.memory_map(buffer).log_warning()?;
    f(&map, &mut iter);
    ok(())
}