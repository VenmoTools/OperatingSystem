use crate::ia_32e::VirtAddr;

/// RSPn： Canonical型栈指针(特权级0-2)
/// ISTn: Canonical型中断栈表(共8组)
/// I/O位图基地址： I/O许可位图
/// 在发生特权级转换时，把新的SS段寄存器设置为NULL段选择子是为了完成远跳转(far call或INT n或中断或异常)，
/// 旧SS段寄存器和RSP将被保存在新栈中
#[derive(Debug,Clone, Copy)]
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