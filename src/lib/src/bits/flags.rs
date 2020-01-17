///! 存放所有的位操作

use crate::bitflags::bitflags;

bitflags! {
    /// 页异常错误码
    #[repr(transparent)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

bitflags! {
    /// PTE flag
    pub struct PageTableFlags: u64 {
        /// 页存在标志位，如果置1表示存在否则表示不存在
        const PRESENT =         1 << 0;
        /// 物理页可写标志位
        /// 如果1级页表没有设置该标志位，那么对应的物理页是只读
        /// 如果其他高等级页表没有设置该位，那么表示表示这个该页所映射的整个范围都是只读的
        const WRITABLE =        1 << 1;
        /// 表示该页是否能在用户模式访问 置1时用户模式，置0为内核模式
        const USER_ACCESSIBLE = 1 << 2;
        /// 页级写穿标志位， 如果置1表示写穿`write-through`用于缓存 置0表示 回写`write-back`
        const WRITE_THROUGH =   1 << 3;
        /// 禁止页级缓存标志位 置1时表示页不能缓存，否则表示页可以缓存
        const NO_CACHE =        1 << 4;
        /// 访问标示位， 置0时表示CPU未访问，置1时表示CPU已访问
        const ACCESSED =        1 << 5;
        /// 脏页标志位。 置1时为脏页，置0时为干净页
        const DIRTY =           1 << 6;
        /// 页面属性标志位，只能用于2级或3级页表(如果支持PAT则置为1否则必须值0)
        const HUGE_PAGE =       1 << 7;
        /// 全局属性标志位， 如果置1表示全局页面，置0表示局部页面，
        /// 更新CR3控制寄存器时不会刷新TLB内的全局页表项
        const GLOBAL =          1 << 8;
        /// 9-11无映射，可自用
        const BIT_9 =           1 << 9;
        const BIT_10 =          1 << 10;
        const BIT_11 =          1 << 11;
        /// 52-58无映射，可自用
        const BIT_52 =          1 << 52;
        const BIT_53 =          1 << 53;
        const BIT_54 =          1 << 54;
        const BIT_55 =          1 << 55;
        const BIT_56 =          1 << 56;
        const BIT_57 =          1 << 57;
        const BIT_58 =          1 << 58;
        const BIT_59 =          1 << 59;
        /// Protection key如果CR4.PKE=1表示页不保护键，可以忽略
        const Protection_60 =          1 << 60;
        const Protection_61 =          1 << 61;
        const Protection_62 =          1 << 62;
        /// 如果IA32_EFER.NXE = 1，则禁用执行
        /// （如果为1，则不允许从此条目控制的1 GB页面中提取指令；请参见4.6节）
        /// 否则，保留（必须为0）
        /// 仅当在EFER寄存器中启用了不执行页面保护功能时才可以使用
        const NO_EXECUTE =      1 << 63;
    }
}

// GDT 描述符标志位
// 代码段描述符标志位
// 位43  42 	41 	40 	描述符类型 	说明
//  1 	  0 	0 	0 	代码 	    仅执行
//  1 	  0 	0 	1 	代码 	    仅执行，已访问
//  1 	  0 	1 	0 	代码 	    执行/可读
//  1 	  0 	1 	1 	代码 	    执行/可读，已访问
//  1 	  1 	0 	0 	代码 	    一致性段，仅执行
//  1 	  1 	0 	1 	代码 	    一致性段，仅执行，已访问
//  1 	  1 	1 	0 	代码 	    一致性段，执行/可读
//  1 	  1 	1 	1 	代码 	    一致性段，执行/可读，已访问
// 代码段描述符
// 43  42  41  40 	说明
// 0 	0   0   0   16B描述符的高8B
// 0 	0   1   0   LDT段描述符
// 1 	0 	0 	1 	64位TSS段描述符
// 1 	0 	1 	1 	64位TSS段描述符
// 1 	1 	1 	0 	64位中断门描述符
// 1 	1 	1 	1 	64位陷进门描述符
bitflags! {
   pub struct DescriptorFlags: u64 {
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const PRESENT           = 1 << 47;
        const LONG_MODE         = 1 << 53;
        const DPL_RING_3        = 3 << 45;
    }
}