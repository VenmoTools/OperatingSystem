use lazy_static::lazy_static;
use spin::Mutex;
use system::ia_32e::paging::{PageIndex, PageTable};
use system::ia_32e::paging::mapper::RecursivePageTable;

pub const PML4T: usize = 0xffffffff_fffff000;

lazy_static! {
    pub static ref RECU_PAGE_TABLE:Mutex<RecursivePageTable<'static>> = init_page();
}
pub fn init_page() -> Mutex<RecursivePageTable<'static>> {
    let res = Mutex::new(unsafe { RecursivePageTable::new_unchecked(&mut *(PML4T as *mut PageTable), PageIndex::new(511)) });
    println!("enable paging... done");
    res
}
