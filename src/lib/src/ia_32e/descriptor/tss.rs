use crate::ia_32e::addr::VirtAddr;

pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [VirtAddr; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [VirtAddr; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub io_map_base: u16,
}
