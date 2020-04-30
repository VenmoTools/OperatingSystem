use lazy_static::lazy_static;
use system::ia_32e::descriptor::{Descriptor, GlobalDescriptorTable, SegmentSelector, TaskStateSegment};
use system::ia_32e::VirtAddr;

pub const DOUBLE_FAULT_LIST_INDEX: usize = 0;


pub struct Selectors {
    pub kernel_data_selector: SegmentSelector,
    pub kernel_code_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

lazy_static! {
   pub static ref TSS: TaskStateSegment = load_tss();
}

lazy_static! {
   pub static ref GDT:(GlobalDescriptorTable,Selectors) = load_gdt();
}

fn load_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();
    let double_fault_stack = {
        // 这里STACK_SIZE需要显示写出是usize的，因为我们的VirtAddr只完成了对usize的加法操作
        const STACK_SIZE: usize = 4096;
        // 指定使用的栈大小位4096
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        // 把STACK的转为裸指针（这样可能造成未定义行为）
        let stack_start = VirtAddr::from_pointer(unsafe { &STACK });
        // 栈的生长方向是高地址向低地址我们添加栈的高地址
        let stack_end = stack_start + STACK_SIZE;
        stack_end
    };
    tss.interrupt_stack_table[DOUBLE_FAULT_LIST_INDEX] = double_fault_stack;
    tss
}

fn load_gdt() -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    // feature thread local store
    // kernel code
    let kernel_data_selector = gdt.add_descriptor(Descriptor::kernel_code_segment());
    // kernel data
    let kernel_code_selector = gdt.add_descriptor(Descriptor::kernel_data_segment());
    // user code
    let user_code_selector = gdt.add_descriptor(Descriptor::user_code_segment());
    // user data
    let user_data_selector = gdt.add_descriptor(Descriptor::user_data_segment());
    // tss
    let tss_selector = gdt.add_descriptor(Descriptor::tss_segment(&TSS));
    (gdt, Selectors {
        kernel_data_selector,
        kernel_code_selector,
        user_code_selector,
        user_data_selector,
        tss_selector,
    })
}

pub fn init_tss() {
    use system::ia_32e::instructions::tables::load_tss;
    let selector: &Selectors = &GDT.1;
    unsafe {
        load_tss(selector.tss_selector);
    }
}

pub fn init_gdt() {
    // load gdt
    GDT.0.load();
    use system::ia_32e::instructions::segmention::{load_ds, load_es, load_fs, load_gs};
    let selector: &Selectors = &GDT.1;
    // Load the segment descriptors
    unsafe {
        // for some reason this code can't run
        // use system::ia_32e::instructions::segmention::{ cs, set_cs,load_ss };
        // set_cs(selector.kernel_code_selector);
        // load_ss(selector.user_data_selector);
        load_ds(selector.user_data_selector);
        load_es(selector.user_data_selector);
        load_fs(selector.user_data_selector);
        load_gs(selector.user_data_selector);
    }
}