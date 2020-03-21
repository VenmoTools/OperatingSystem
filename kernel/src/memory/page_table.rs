use lazy_static::lazy_static;
use spin::Mutex;
use system::bits::flags::PageTableFlags;
use system::ia_32e::paging::{FrameAllocator, Page4KB, PageSize, PageTable};
use system::ia_32e::paging::mapper::Mapper;
use system::ia_32e::PhysAddr;
use system::KernelArgs;
use system::result::Error;

use crate::paging::RecursivePageTable;

lazy_static! {
    pub static ref RECU_PAGE_TABLE:Mutex<RecursivePageTable<'static>> = Mutex::new(RecursivePageTable::empty());
}
pub fn init_page(table: &'static mut PageTable) -> Result<(), Error> {
    RECU_PAGE_TABLE.lock().init(table)?;
    Ok(())
}


pub fn map_kernel<S: PageSize, A: FrameAllocator<Page4KB>>(a: &mut A, args: &KernelArgs) {
    // map to kernel
    // todo:
    use system::ia_32e::paging::{Page, Page2MB, UnusedFrame, Frame};
    use system::ia_32e::VirtAddr;
    let kernel_start: Page<Page2MB> = Page::include_address(VirtAddr::new(args.kernel_start));
    let kernel_end: Page<Page2MB> = Page::include_address(VirtAddr::new(args.kernel_end));
    let page_table: &mut RecursivePageTable = &mut RECU_PAGE_TABLE.lock();
    for p in Page::range_include(kernel_start, kernel_end) {
        let frame = unsafe { UnusedFrame::new(Frame::include_address(PhysAddr::new(0))) };
        match page_table.map_to(p, frame, PageTableFlags::PRESENT, a) {
            Ok(flush) => flush.flush(),
            Err(e) => println!("map memory error {:?}", e)
        };
    }
    // map to kernel stack
    let stack_start: Page<Page2MB> = Page::include_address(VirtAddr::new(args.stack_start));
    let stack_end: Page<Page2MB> = Page::include_address(VirtAddr::new(args.stack_end));
    for p in Page::range_include(stack_start, stack_end) {
        let frame = unsafe { UnusedFrame::new(Frame::include_address(PhysAddr::new(0))) };
        match page_table.map_to(p, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, a) {
            Ok(flush) => flush.flush(),
            Err(e) => println!("map memory error {:?}", e)
        };
    }
}