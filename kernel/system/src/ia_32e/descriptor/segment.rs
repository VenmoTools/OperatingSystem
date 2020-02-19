use core::fmt;

use crate::bits::BitOpt;
use crate::ia_32e::PrivilegedLevel;

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
