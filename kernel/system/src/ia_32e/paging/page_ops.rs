use super::ENTRY_COUNT;

/// 页表的9位索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageIndex(u16);

impl PageIndex {
    /// 根据给定的索引创建页表索引值
    /// 索引最大不能超过512个
    pub fn new(index: u16) -> Self {
        assert!(usize::from(index) < ENTRY_COUNT);
        Self(index)
    }
    pub fn empty() -> Self{
        Self(0)
    }
    /// 索引最大不能超过512个,使用取模运算可防止超出512
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }
}

impl From<PageIndex> for u16 {
    fn from(index: PageIndex) -> Self {
        index.0
    }
}

impl From<PageIndex> for u32 {
    fn from(index: PageIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageIndex> for u64 {
    fn from(index: PageIndex) -> Self {
        u64::from(index.0)
    }
}

/// 4KB页面 12位偏移
#[derive(Debug, Clone,Ord, PartialOrd, Eq, PartialEq)]
pub struct PageOffset(u16);

impl PageOffset {
    /// 根据给定u16类型创建偏移值
    /// 如果给定的偏移超过你4096将会Panic
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    /// 永远小于 >=4096.
    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u16 {
    fn from(index: PageOffset) -> Self {
        index.0
    }
}

impl From<PageOffset> for u32 {
    fn from(index: PageOffset) -> Self {
        u32::from(index.0)
    }
}

impl From<PageOffset> for u64 {
    fn from(index: PageOffset) -> Self {
        u64::from(index.0)
    }
}