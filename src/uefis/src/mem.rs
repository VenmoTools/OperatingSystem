use uefi::{Completion, Result, ResultExt, Status};
use uefi::table::boot::{AllocateType, BootServices, MemoryType};

pub fn malloc_one_page(bt: &BootServices) -> Result<&mut [u8; 4096]> {
    let page = bt.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1).log_warning()?;
    let data = unsafe {
        &mut *(page as *mut [u8; 4096])
    };
    Ok(Completion::new(Status::SUCCESS, data))
}

pub fn free_one_page(bt: &BootServices, page: &[u8; 4096]) -> Result {
    let addr = unsafe {
        *(page.as_ptr() as *const u64)
    };
    bt.free_pages(addr, 1)
}