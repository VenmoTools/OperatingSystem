use crate::ia_32e::addr::VirtAddr;
use core::fmt;
use crate::ia_32e::Hex;


#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    /// 64位 canonical型地址，栈指针(RSP)，特权级Ring0-Ring2
    pub privilege_stack_table: [VirtAddr; 3],
    reserved_2: u64,
    /// 64位 canonical型地址 中断栈表 (IST)
    pub interrupt_stack_table: [VirtAddr; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// 16位IO位图
    pub io_map_base: u16,
}

impl fmt::Debug for TaskStateSegment{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut de = f.debug_struct("TaskStateSegment");
        de.field("privilege_stack_table",&self.privilege_stack_table);
        de.field("interrupt_stack_table",&self.interrupt_stack_table);
        de.field("io_map_base",&self.io_map_base);
        de.finish()
    }
}

impl TaskStateSegment {
    /// 创建一个0,io位图，0特权级的TSS
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [VirtAddr::zero(); 3],
            interrupt_stack_table: [VirtAddr::zero(); 7],
            io_map_base: 0,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}