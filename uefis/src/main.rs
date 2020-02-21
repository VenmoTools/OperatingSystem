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

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::slice::Iter;
use osloader::elf::{Elf, Elf64};
use osloader::OsLoaderAlloc;
use result::{AppResultExt, err, Error, ok, Result, UefiResult};
use uefi::Completion;
use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileHandle, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::file::FileType::Regular;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryMapIter, MemoryType};
use uefi::table::Runtime;

use crate::fs::Read;

mod result;
mod fs;


static mut KERNEL_ENTRY: u64 = 0;

//type ENTRY = extern "C" fn(SystemTable<Runtime>, MemoryMapIter) -> !;
type ENTRY = extern "C" fn();

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    if let Err(e) = uefi_services::init(&st).log_warning() {
        info!("{:?}", e);
        return e.status();
    }

    let _res = st.stdout().reset(false);
    let bt = st.boot_services();

    let mut f = fs::File::new(bt);
    let mut reader = f.open(r"EFI\Boot\kernel", "r").log_warning().unwrap().unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf);
    info!("{}", buf.len());

    let mut allocator = osloader::Allocator::new(bt);

    let mut loader = osloader::ElfLoader::new(allocator, Elf::from_bytes(buf.as_slice()).unwrap());
    unsafe { KERNEL_ENTRY = loader.load_memory() as u64 };
    let rs = st.runtime_services();
    shutdown(image, st);
    loop {}
    Status::SUCCESS
}

fn shutdown(image: uefi::Handle, st: SystemTable<Boot>) {
    let mmap_size = st.boot_services().memory_map_size();
    let mut mmp = vec![0; mmap_size].into_boxed_slice();
    let kernel_entry: ENTRY = unsafe { core::mem::transmute::<u64, ENTRY>(KERNEL_ENTRY) };

    info!("{:#X?}", kernel_entry as *const u64);
    info!("exit boot services...");
//    show_memory_layout(st.boot_services());

//    if let Err(e) = st.stdout().reset(false).log_warning() {
//        error!("{:?}", e);
//        loop {}
//    }
    kernel_entry()
//    let (st, it) = st.exit_boot_services(image, &mut mmp[..]).log_warning().unwrap();
//    (kernel_entry)(st, it)
}

fn show_memory_layout(bt: &BootServices) -> UefiResult<Result<()>> {
    let size = bt.memory_map_size();
    let buffer = bt.allocate_pool(MemoryType::BOOT_SERVICES_DATA, size).log_warning()?;
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, size) };
    let (map, mut iter) = bt.memory_map(buffer).log_warning()?;
    while let Some(desc) = iter.next() {
        info!("{:?}", desc);
    }
    ok(())
}