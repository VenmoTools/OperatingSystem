use core::fmt;

use crate::bits::{BitOpt, DescriptorFlags};

use super::super::Hex;
use super::tss::TaskStateSegment;

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
            Descriptor::SystemSegment(seg_l, seg_h) => {
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