use system::ia_32e::paging::PageSize;
use system::KernelArgs;

use crate::memory::frame::PhysicalAllocator;

pub mod frame;


pub fn init<S: PageSize>(a: &mut PhysicalAllocator<S>, args: &KernelArgs) {
    // map to kernel
    // todo:
    use system::ia_32e::paging::{Page, Page2MB, UnusedFrame, Frame};
    use system::ia_32e::VirtAddr;
    let kernel_start: Page<Page2MB> = Page::include_address(VirtAddr::new(args.kernel_start));
    let kernel_end: Page<Page2MB> = Page::include_address(VirtAddr::new(args.kernel_end));
    for p in Page::range_include(kernel_start, kernel_end) {
        let frame = unsafe { UnusedFrame::new(Frame::include_address()) };
        match page_table.map_to(p, frame, PageTableFlags::PRESENT, &mut allocator) {
            Ok(flush) => flush.flush(),
            Err(e) => println!("map memory error {:?}", e)
        };
    }
    // map to kernel stack
    let stack_start: Page<Page2MB> = Page::include_address(VirtAddr::new(args.stack_start));
    let stack_end: Page<Page2MB> = Page::include_address(VirtAddr::new(args.stack_end));
    for p in Page::range_include(stack_start, stack_end) {
        let frame = unsafe { UnusedFrame::new(Frame::include_address(stack_start)) };
        match page_table.map_to(p, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, &mut allocator) {
            Ok(flush) => flush.flush(),
            Err(e) => println!("map memory error {:?}", e)
        };
    }
}