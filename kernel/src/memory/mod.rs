pub use allocator::{add_to_heap, alloc_memory, FRAME_ALLOCATOR, HEAP, init_frame_allocator};
pub use page_table::{init_page, PML4T, RECU_PAGE_TABLE};

mod allocator;
mod page_table;

