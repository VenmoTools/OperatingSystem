use spin::Mutex;
use system::ia_32e::paging::{PageIndex, PageTable};
use system::ia_32e::paging::mapper::RecursivePageTable;

use lazy_static::lazy_static;

pub const PML4T: usize = 0xffffffff_fffff000;

lazy_static! {
    pub static ref RECU_PAGE_TABLE:Mutex<RecursivePageTable<'static>> = init_page();
}
pub fn init_page() -> Mutex<RecursivePageTable<'static>> {
    let res = Mutex::new(unsafe { RecursivePageTable::new_unchecked(&mut *(PML4T as *mut PageTable), PageIndex::new(511)) });
    println!("enable paging... done");
    res
}

// pub fn map_kernel(info: &SystemInformation) {
//     let mut table = RECU_PAGE_TABLE.lock();
//
//     for area in info.kernel_area_iter() {
//
//         match area.section_ty{
//             KernelSectionType::ProcessorSpecific=> {
//                 let start: Page = Page::include_address(VirtAddr::new(area.start_addr));
//                 let end: Page = Page::include_address(VirtAddr::new(area.end_addr));
//                 let start_frame:Frame = Frame::include_address(PhysAddr::new(area.start_addr));
//                 let end_frame = Frame::include_address(PhysAddr::new(area.start_addr + area.size - 1));
//                 let flag = match area.flags {
//                     KernelSectionFlags::WRITABLE => PageTableFlags::WRITABLE,
//                     KernelSectionFlags::EXECUTABLE => PageTableFlags::ACCESSED,
//                     KernelSectionFlags::ALLOCATED => PageTableFlags::NO_EXECUTE
//                 } | PageTableFlags::PRESENT;
//                 for frame in Frame::frame_range_include(start_frame,end_frame){
//                     let offset = frame - start_frame;
//                     let page = start + offset;
//                     unsafe{
//                         let allocator = FRAME_ALLOCATOR.lock().as_mut().unwrap();
//                         table.map_to(page,frame,flag,allocator).unwrap().flush();
//                     }
//                 }
//             }
//             _ => {println!("no support yet")}
//         }
//     }
// }
