use system::ia_32e::descriptor::{GlobalDescriptorTable, SegmentSelector, TaskStateSegment};
use system::ia_32e::instructions::tables::sgdt;

use lazy_static::lazy_static;

// use system::ia_32e::VirtAddr;

pub const DOUBLE_FAULT_LIST_INDEX: u16 = 0;


struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let tss = TaskStateSegment::new();
        //todo: use page table crate double fault stack
        // tss.interrupt_stack_table[DOUBLE_FAULT_LIST_INDEX as usize] = stack_len;
        tss
    };
}

lazy_static! {
    static ref GDT:GlobalDescriptorTable = {
        // todo: Update gdt
        let ptr = sgdt();
        let gdt = unsafe{GlobalDescriptorTable::from_ptr(ptr)};
        // let code_selector = gdt.add_descriptor(Descriptor::kernel_code_segment());
        // let tss_selector = gdt.add_descriptor(Descriptor::tss_segment(&TSS));
        gdt
    };
}

pub fn init_gdt_and_tss() {
    // use system::ia_32e::instructions::segmention::cs;
    // use system::ia_32e::instructions::segmention::set_cs;
    // use system::ia_32e::instructions::tables::load_tss;
    //
    // GDT.0.load();
    //
    // unsafe {
    //     set_cs(GDT.1.code_selector);
    //     load_tss(GDT.1.tss_selector);
    // }
}