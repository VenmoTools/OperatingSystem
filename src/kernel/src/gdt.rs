use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_LIST_INDEX: u16 = 0;


struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    // 此双重故障堆栈没有保护页面以防止堆栈溢出，因此不能在双重故障处理程序中执行任何占用大量堆栈的操作
    static ref TSS:TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_LIST_INDEX as usize] = {
            const StackSize:usize = 4096;
            static mut Stack:[u8;StackSize] = [0;StackSize];

            let stack_start = VirtAddr::from_ptr(unsafe{&Stack});
            let stack_end = stack_start + StackSize;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT:(GlobalDescriptorTable,Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt,Selectors{code_selector,tss_selector})
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}