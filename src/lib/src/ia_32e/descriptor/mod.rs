use bitflags::bitflags;
use core::fmt;

use crate::bits::BitOpt;
use crate::ia_32e::descriptor::tss::TaskStateSegment;
use crate::ia_32e::PrivilegedLevel;

pub mod gdt;
pub mod idt;
pub mod tss;


#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub fn new(index: u16, rpl: PrivilegedLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    pub fn index(&self) -> u16 {
        self.0 >> 3
    }

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

bitflags! {
    /// GDT 描述符标志位
   pub struct DescriptorFlags: u64 {
        /// 数据段描述符，可写
        const WRITABLE          = 1 << 41;
        ///
        /// Marks a code segment as “conforming”. This influences the privilege checks that
        /// occur on control transfers.
        const CONFORMING        = 1 << 42;
        /// 代码段描述符
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Must be set for long mode code segments.
        const LONG_MODE         = 1 << 53;

        /// The DPL for this descriptor is Ring 3
        const DPL_RING_3        = 3 << 45;
    }

}

#[derive(Debug, Clone)]
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::EXECUTABLE | Flags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn user_data_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::WRITABLE | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn user_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;
        let flags = Flags::USER_SEGMENT
            | Flags::PRESENT
            | Flags::EXECUTABLE
            | Flags::LONG_MODE
            | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn tss_segment(ts: &'static TaskStateSegment) -> Descriptor {
        use self::DescriptorFlags as Flags;
        use core::mem::size_of;

        let ptr = ts as *const _ as u64;

        let mut low = Flags::PRESENT.bits();

        low.set_bits(16..40, ptr);
        low.set_bits(56..64, ptr);
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));
        Descriptor::SystemSegment(low, high)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// 描述符表大小
    pub limit: u16,
    /// 描述符表的内存裸指针
    pub base: u64,
}

