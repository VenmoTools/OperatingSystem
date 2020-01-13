#![no_std]
#![no_main]
#![feature(asm)]
#![feature(slice_patterns)]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::{Completion, Result};
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::media::file::{Directory, File, FileAttribute, FileHandle, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi_services::system_table;

mod mem;

static mut KERNEL_ENTRY: u64 = 0;
static KERNEL_NAME: &'static str = "kernel.bin";
static mut KERNEL_SIZE: usize = 0;

#[no_mangle]
extern "C" fn __rust_probestack() {}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st);
    st.stdout().reset(false);
    // 找到并加载内核入口文件
    let res = files(&st.boot_services());
    if let Err(e) = res {
        error!("{:?}", e);
    }
    unsafe {
        info!("kernel address: {:X?}", KERNEL_ENTRY);
        info!("kernel size: {:?}", KERNEL_SIZE);
    }

    shutdown(image, st)
}

fn read_kernel(bt: &BootServices, files: &FileInfo, k_file: &mut RegularFile) -> Option<Vec<u8>> {
    info!("Read kernel file total size: {}", files.file_size());
    let mut kernel_data: Vec<u8> = Vec::with_capacity(files.file_size() as usize);
    info!("create read buffer");
    let mut buffer = mem::malloc_one_page(bt).log_warning();
    if let Err(e) = buffer {
        info!("create buffer failed! error: {:?}", e);
        return None;
    }
    let mut buffer = buffer.unwrap();
    info!("create {} bytes buffer", buffer.len());
    info!("start read");
    while let Ok(size) = k_file.read(buffer) {
        let (read_status, size) = size.split();
        if read_status != Status::SUCCESS {
            error!("Read kernel failed! status: {:?}", read_status);
            return None;
        }
        if size == 0 {
            break;
        }
        kernel_data.extend(&buffer[..size]);
    }
    info!("read finish total data: {} bytes", kernel_data.len());
    mem::free_one_page(bt, buffer);
    Some(kernel_data)
}

fn handle_kernel_file(bt: &BootServices, name: &str, sub: &mut Directory, files: &FileInfo) -> uefi::Result<()> {
    let kernel_file = sub
        .open(name, FileMode::Read, files.attribute())
        .log_warning()?;
    if let FileType::Regular(mut k_file) = kernel_file.into_type().log_warning()? {
        if let Some(kernel_data) = read_kernel(bt, files, &mut k_file) {
            unsafe {
                KERNEL_SIZE = kernel_data.len();
                KERNEL_ENTRY = *(kernel_data.as_ptr() as *const u64);
            }
        } else {
            return Ok(Completion::from(Status::LOAD_ERROR));
        }
    }
    Ok(Completion::from(Status::SUCCESS))
}

fn files(bt: &BootServices) -> uefi::Result<()> {
    let file = bt.locate_protocol::<SimpleFileSystem>().log_warning()?;
    let f = unsafe { &mut *file.get() };
    let mut volume = f.open_volume().log_warning()?;

    let en_buff = mem::malloc_one_page(bt).log_warning()?;
    let dir = volume.read_entry(en_buff).unwrap().unwrap().unwrap();

    let efi = volume
        .open("EFI", FileMode::Read, dir.attribute())
        .log_warning()?.into_type().log_warning()?;
    // 读取EIF文件夹
    if let FileType::Dir(mut d) = efi {
        info!("try to read `Boot` folder");
        // 读取Boot文件夹
        let f_h = d
            .open("Boot", FileMode::Read, dir.attribute())
            .log_warning()?
            .into_type().log_warning()?;
        // 遍历Boot文件夹中的内容
        if let FileType::Dir(mut sub) = f_h {
            info!("try to find `kernel` file");
            while let Ok(f_info) = sub.read_entry(en_buff) {
                let (status, f) = f_info.split();
                if status == Status::SUCCESS {
                    if let Some(files) = f {
                        let name = String::from_utf16_lossy(files.file_name().to_u16_slice());
                        if name == KERNEL_NAME.to_string() {
                            info!("kernel file found! try to read kernel file");
                            return handle_kernel_file(bt, name.as_str(), &mut sub, &files);
                        }
                    }
                }
            }
        }
    }
    Ok(Completion::from(Status::SUCCESS))
}

fn shutdown(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    loop {}
}

