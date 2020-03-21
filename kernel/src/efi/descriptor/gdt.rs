use lazy_static::lazy_static;
use system::ia_32e::descriptor::{Descriptor, GdtEntry, GlobalDescriptorTable, SegmentSelector, TaskStateSegment};
use system::ia_32e::VirtAddr;

pub const DOUBLE_FAULT_LIST_INDEX: usize = 0;


struct Selectors {
    kernel_data_selector: SegmentSelector,
    kernel_code_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = load_tss();
}

lazy_static! {
    static ref GDT:(GlobalDescriptorTable,Selectors) = load_gdt();
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

#[allow(dead_code)]
fn load_gdt2() -> (GlobalDescriptorTable, Selectors) {
    use system::bits::flags::{GdtAccessFlags as ACCESS, GdtFlags as Flags};
    let mut gdt = GlobalDescriptorTable::new();
    // kernel code
    let kernel_data_selector = gdt.add_entry(GdtEntry::new(0, 0, ACCESS::PRESENT | ACCESS::RING_0 | ACCESS::SYSTEM | ACCESS::EXECUTABLE | ACCESS::PRIVILEGE, Flags::LONG_MODE));
    // kernel data
    let kernel_code_selector = gdt.add_entry(GdtEntry::new(0, 0, ACCESS::PRESENT | ACCESS::RING_0 | ACCESS::SYSTEM | ACCESS::PRIVILEGE, Flags::LONG_MODE));
    // user code
    let user_code_selector = gdt.add_entry(GdtEntry::new(0, 0, ACCESS::PRESENT | ACCESS::RING_3 | ACCESS::SYSTEM | ACCESS::EXECUTABLE | ACCESS::PRIVILEGE, Flags::LONG_MODE));
    // user data
    let user_data_selector = gdt.add_entry(GdtEntry::new(0, 0, ACCESS::PRESENT | ACCESS::RING_3 | ACCESS::SYSTEM | ACCESS::PRIVILEGE, Flags::LONG_MODE));
    // TSS
    let tss_selector = gdt.add_entry(GdtEntry::new(0, 0, ACCESS::PRESENT | ACCESS::RING_3 | ACCESS::TSS_AVAIL, Flags::empty()));
    gdt.add_entry(GdtEntry::new(0, 0, ACCESS::empty(), Flags::empty()));
    (gdt, Selectors {
        kernel_data_selector,
        kernel_code_selector,
        user_code_selector,
        user_data_selector,
        tss_selector,
    })
}

pub fn init_gdt_and_tss() {
    #[cfg(feature = "bios")]
    use system::ia_32e::instructions::segmention::{cs, set_cs, load_ds, load_es, load_fs, load_gs, load_ss};
    use system::ia_32e::instructions::tables::load_tss;
    // load gdt
    GDT.0.load();
    let selector: &Selectors = &GDT.1;
    // Load the segment descriptors
    unsafe {
        #[cfg(feature = "bios")]
            set_cs(selector.kernel_code_selector);
        load_tss(selector.tss_selector);
        #[cfg(feature = "bios")]
            load_ds(selector.kernel_data_selector);
        #[cfg(feature = "bios")]
            load_es(selector.kernel_data_selector);
        #[cfg(feature = "bios")]
            load_fs(selector.kernel_data_selector);
        #[cfg(feature = "bios")]
            load_gs(selector.kernel_data_selector);
        #[cfg(feature = "bios")]
            load_ss(selector.kernel_data_selector);
    }
}