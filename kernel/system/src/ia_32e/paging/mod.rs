pub use allocator::{FrameAllocator, UnusedFrame};
pub use frame::Frame;
pub use page::{NotGiantPageSize, Page, Page1GB, Page2MB, Page4KB, PageSize,PageRange,PageRangeInclude};
pub use page_ops::{PageIndex, PageOffset};
pub use page_table::{ENTRY_COUNT, PageTable, PageTableEntry};

use crate::ia_32e::PhysAddr;

///! 提供了内存分页功能
mod page;
mod page_table;
mod page_ops;
pub mod frame;
pub mod allocator;
pub mod mapper;
pub mod result;

pub struct PagingArgs {
    pub pml4t_base_addr: u64,
    // PDPT页表基地址，用于链接到 pml4te中，对齐方式为0x1000
    pub pdpt_base_addr: u64,
    /// pd_base_addr大小为4*4096
    pub pd_base_addr: u64,
}

/// 2MB Paging
pub unsafe fn enable_4_level_paging(args: PagingArgs) {
    use core::ptr::{write_bytes, write};
    use crate::ia_32e::cpu::control::{CR3, CR0, CR4};
    use crate::bits::{CR4Flags, CR0Flags, EferFlags, PageTableFlags};
    use crate::ia_32e::cpu::apic;

    let pml4t = args.pml4t_base_addr;
    // 创建6个4096大小的PML4TE
    write_bytes(pml4t as *mut u8, 0, 6 * 4096);

    let mut base = pml4t;

    // 第一个PML4
    let pdpt_flags = PageTableFlags::from_bits_truncate(args.pdpt_base_addr)
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    // Link first PML4 and second to last PML4 to PDP
    write(base as *mut u64, pdpt_flags.bits());
    write((base + 510 * 8) as *mut u64, pdpt_flags.bits());
    // Link last PML4 to PML4
    write((base + 511 * 8) as *mut u64, pdpt_flags.bits());
    // 移动至PDP
    base += 4096;

    // Link first four PDP to PD
    let pd1 = PageTableFlags::from_bits_truncate(args.pd_base_addr)
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    write(base as *mut u64, pd1.bits());
    let pd2 = PageTableFlags::from_bits_truncate(args.pd_base_addr + 0x1000)
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    write((base + 8) as *mut u64, pd2.bits());
    let pd3 = PageTableFlags::from_bits_truncate(args.pd_base_addr + 0x1000 * 2)
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    write((base + 16) as *mut u64, pd3.bits());
    let pd4 = PageTableFlags::from_bits_truncate(args.pd_base_addr + 0x1000 * 3)
        | PageTableFlags::WRITABLE
        | PageTableFlags::PRESENT;
    write((base + 16) as *mut u64, pd4.bits());

    // Move to PD
    base += 4096;

    // PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE;
    let mut entry = 1 << 7 | 1 << 1 | 1;
    for i in 0..4 * 512 {
        write((base + i * 8) as *mut u64, entry);
        entry += 0x200000;
    }

    let mut flags = CR4::read();
    flags |= CR4Flags::OSXSAVE
        | CR4Flags::PAGE_GLOBAL
        | CR4Flags::PHYSICAL_ADDRESS_EXTENSION
        | CR4Flags::OSFXSR
        | CR4Flags::PAGE_SIZE_EXTENSION;
    CR4::write(flags);


    // Enable Long mode and NX bit
    let mut msr_flag = apic::Efer::read();
    msr_flag |= EferFlags::NO_EXECUTE_ENABLE | EferFlags::LONG_MODE_ENABLE;
    apic::Efer::write(msr_flag);

    // 重新设置新的内存布局
    let cr3_flags = CR3::read().1;
    CR3::write(Frame::from_start_addr(PhysAddr::new(pml4t)).unwrap(), cr3_flags);

    // 启用分页
    let mut cr0_flags = CR0::read();
    cr0_flags |= CR0Flags::PAGING
        | CR0Flags::WRITE_PROTECT
        | CR0Flags::PROTECTED_MODE_ENABLE;
    CR0::write(cr0_flags);
}