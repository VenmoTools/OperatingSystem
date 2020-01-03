pub mod gdt;
pub mod idt;
pub mod tss;


use crate::ia_32e::{PrivilegedLevel, Hex};
use crate::bits::BitOpt;
use bitflags::bitflags;
use core::fmt;
use crate::ia_32e::descriptor::tss::TaskStateSegment;

/// 16位段选择子，用于索引GDT
///
/// | 15-    3   |2 - 1|  0  |
/// +------------+-----+-----+
/// |   Index    | TI  | RPL |
/// +------------+-----+-----+
///
/// 名称 	功能描述
/// Index 	用于索引目标段描述符
/// TI 	    目标段描述符所在的描述符表类型
/// RPL 	请求特权级
///
/// 详情查看 Intel 3a, Section 3.4.2 "Segment Selectors"
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// 使用给定的索引和特权级创建新的段选择子
    pub fn new(index: u16, rpl: PrivilegedLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }
    /// 返回当前的描述符索引
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }
    /// 返回当前描述符的特权级
    pub fn rpl(&self) -> PrivilegedLevel {
        PrivilegedLevel::from_u16(self.0.get_bits(0..2))
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("rpl", &self.rpl());
        s.finish()
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

/// 64位描述符
#[derive(Clone)]
pub enum Descriptor {
    /// 用户代码段描述符
    UserSegment(u64),
    /// 系统段描述符
    SystemSegment(u64, u64),
}

impl fmt::Debug for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut x = f.debug_struct("Descriptor");

        match &self {
            Descriptor::UserSegment(seg) => {
                x.field("user_segment", &Hex(*seg));
            }
            Descriptor::SystemSegment(seg_l,seg_h) => {
                x.field("system_segment_low", &Hex(*seg_l));
                x.field("system_segment_high", &Hex(*seg_h));
            }
        }
        x.finish()
    }
}


impl Descriptor {
    /// 内核代码段描述符
    /// | 63-56     |55|54 |53|52 |51-48   |47|46-45|44|43 |42|41|40|39-16       |15 - 0    |
    /// +-----------+--+---+--+---+--------+--+-----+--+---+--+--+--+------------+----------+
    /// |BaseAddr(H)|G |D/B|L |AVL|limit(H)|P |DPL  |S |C/D|C |R |A | BaseAddr(L)| limit(L) |
    /// +-----------+--+---+--+---+--------+--+-----+--+---+--+--+--+------------+----------+
    pub fn kernel_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::EXECUTABLE | Flags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    /// 用户数据段描述符Ring3
    /// |   63-56   |55|54 |53|52 | 51-48  |47|46-45 |44|43 |42|41|40| 39-16      |15-0      |
    /// +-----------+--+---+--+---+--------+--+------+--+---+--+--+--+------------+----------+
    /// |BaseAddr(H)|G |D/B|L |AVL|limit(H)|P |DPL   |S |C/D|E |W |A | BaseAddr(L)| limit(L) |
    /// +-----------+--+---+--+---+--------+--+------+--+---+--+--+--+------------+----------+
    pub fn user_data_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::WRITABLE | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    /// 用户代码段描述符Ring3
    pub fn user_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT
            | Flags::PRESENT
            | Flags::EXECUTABLE
            | Flags::LONG_MODE
            | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    /// 根据给定的`TaskStateSegment`结构创建TSS描述符
    /// TSS描述符
    ///
    /// |127  -   109|   108    |    107 - 104      | 103 - 96        | 95   -   64  |
    /// +------------+----------+-------------------+-----------------+--------------+
    /// | reserved   | reserved |    reserved       |   reserved      |   Base Addr  |
    /// +------------+----------+-------------------+-----------------+--------------+
    ///
    /// | 63 - 56 |55|54|53|52 |51-48|47|46-45|44|43|42|41|40| 39 - 32|31  - 16|15 -0|
    /// +---------+--+--+--+---+-----+--+-----+--+--+--+--+--+--------+--------+-----+
    /// |Base Addr|0 |0 |0 |AVL|Limit|P | DPL |0 |1 |0 |B |1 |BaseAddr|BaseAddr|Limit|
    /// +---------+--+--+--+---+-----+--+-----+--+--+--+--+--+--------+--------+-----+
    ///
    pub fn tss_segment(ts: &'static TaskStateSegment) -> Descriptor {
        use core::mem::size_of;

        let ptr = ts as *const _ as u64;
        let mut low = DescriptorFlags::PRESENT.bits();

        // 段基址(低)
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // 段限长
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // 段属性 1001表示64位TSS段描述符
        low.set_bits(40..44, 0b1001);

        let mut high: u64 = 0;
        // 段基址(高)
        high.set_bits(0..32, ptr.get_bits(32..64));
        Descriptor::SystemSegment(low, high)
    }
}

/// 用于将GDT，IDT等描述符保存为指针形式
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// 描述符段限长
    pub limit: u16,
    /// 描述符的内存裸指针
    pub base: u64,
}

