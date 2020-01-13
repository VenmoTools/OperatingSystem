use system::ia_32e::descriptor::{Descriptor, GlobalDescriptorTable, SegmentSelector, TaskStateSegment};
use system::ia_32e::VirtAddr;

use lazy_static::lazy_static;

pub const DOUBLE_FAULT_LIST_INDEX: u16 = 0;


struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    // 此双重故障堆栈没有保护页面以防止堆栈溢出，因此不能在双重故障处理程序中执行任何占用大量堆栈的操作
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        let stack_len =
        {
            // 这里STACK_SIZE需要显示写出是usize的，因为我们的VirtAddr只完成了对usize的加法操作
            const STACK_SIZE:usize = 4096;
            // 指定使用的栈大小位4096
            static mut STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];
            // 把STACK的转为裸指针（这样可能造成未定义行为）
            let stack_start = VirtAddr::from_pointer(unsafe{&STACK});
            // 栈的生长方向是高地址向低地址我们添加栈的高地址
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_LIST_INDEX as usize] = stack_len;
        tss
    };
}

lazy_static! {
    static ref GDT:(GlobalDescriptorTable,Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_descriptor(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_descriptor(Descriptor::tss_segment(&TSS));
        (gdt,Selectors{code_selector,tss_selector})
    };
}

pub fn init_gdt() {
    use system::ia_32e::instructions::segmention::set_cs;
    use system::ia_32e::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}