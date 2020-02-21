pub use page_ops::{PageIndex, PageOffset};
pub use page_table::{ENTRY_COUNT, PageTable, PageTableEntry};

use crate::ia_32e::PhysAddr;
use crate::result::ResultEx;

///! 提供了内存分页功能
mod page;
mod page_table;
mod page_ops;
pub mod frame;
pub mod allocator;

pub struct PagingInfo {
    pt_base: PhysAddr,
    pml4: PhysAddr,
}

/// 启动4KB分页
pub unsafe fn enable_paging(pt_base: PhysAddr) {
    use core::ptr::{write, write_bytes};
    use crate::ia_32e::cpu::control::{CR3, CR0, CR4};
    use crate::bits::{CR4Flags, CR0Flags, EferFlags};
    use crate::ia_32e::paging::frame::Frame;
    use crate::ia_32e::cpu::msr;

    // Zero PML4, PDP, and 4 PD
    write_bytes(pt_base.as_mut(), 0, 6 * 4096);

    let mut flags = CR4::read();
    flags |= CR4Flags::OSXSAVE
        | CR4Flags::PAGE_GLOBAL
        | CR4Flags::PHYSICAL_ADDRESS_EXTENSION
        | CR4Flags::OSFXSR
        | CR4Flags::PAGE_SIZE_EXTENSION;
    CR4::write(flags);


    // Enable Long mode and NX bit
    let mut msr_flag = msr::Efer::read();
    msr_flag |= EferFlags::NO_EXECUTE_ENABLE | EferFlags::LONG_MODE_ENABLE;
    msr::Efer::write(msr_flag);

    // 重新设置新的内存布局
    let cr3_flags = CR3::read().1;
    CR3::write(Frame::from_start_addr(pt_base).unwrap(), cr3_flags);

    // 启用分页 保护模式
    let mut cr0_flags = CR0::read();
    cr0_flags |= CR0Flags::PAGING
        | CR0Flags::WRITE_PROTECT
        | CR0Flags::PROTECTED_MODE_ENABLE;
    CR0::write(cr0_flags);
}