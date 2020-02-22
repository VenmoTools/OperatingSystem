use bitflags::_core::marker::PhantomData;

use crate::ia_32e::VirtAddr;
use crate::ia_32e::paging::PageIndex;

/// 针对3种不同的页大小的抽象
pub trait PageSize: Copy + Eq + PartialEq + Ord {
    const P_SIZE: u64;
    const DISPLAY_STR: &'static str;
}

/// NotGiantPageSize只用于4KB和2MB页表
pub trait NotGiantPageSize: PageSize {}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page4KB {}

impl PageSize for Page4KB {
    const P_SIZE: u64 = 4096;
    const DISPLAY_STR: &'static str = "page 4 kb";
}

impl NotGiantPageSize for Page4KB {}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page2MB {}

impl PageSize for Page2MB {
    const P_SIZE: u64 = Page4KB::P_SIZE * 512;
    const DISPLAY_STR: &'static str = "page 1 MB";
}
impl NotGiantPageSize for Page2MB {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page1GB {}

impl PageSize for Page1GB {
    const P_SIZE: u64 = Page2MB::P_SIZE * 512;
    const DISPLAY_STR: &'static str = "page 1 GB";
}

/// 页表
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct Page<S: PageSize = Page4KB> {
    /// 起始地址
    start_address: VirtAddr,
    /// 页表大小
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// 当前页大小
    pub const SIZE: u64 = S::P_SIZE;

    /// 返回给定地址的页表
    /// 如果地址没有正确对齐将会返回错误
    pub fn from_start_address(address: VirtAddr) -> Result<Self, ()> {
        if !address.is_aligned(S::P_SIZE) {
            return Err(());
        }
        Ok(Page::include_address(address))
    }
    /// 返回包含给定地址的页表
    pub fn include_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::P_SIZE),
            size: PhantomData,
        }
    }

    /// 获取当前页的虚拟地址
    pub fn start_address(&self) -> VirtAddr {
        self.start_address
    }

    /// 获取当前页大小
    pub const fn size(&self) -> u64 {
        S::P_SIZE
    }

    /// 获取4级页表索引
    pub fn p4_index(&self) -> PageIndex {
        self.start_address.page4_index()
    }
    /// 获取3级页表索引
    pub fn p3_index(&self) -> PageIndex {
        self.start_address.page3_index()
    }
}

impl Page<Page4KB>{
    /// 返回当前页面的1级页表索引
    pub fn p1_index(&self) -> PageIndex{
        self.start_address().page1_index()
    }
    // 返回含有指定页索引内存页（4kb）
    pub fn from_page_table_indices(
        p4_index: PageIndex,
        p3_index: PageIndex,
        p2_index: PageIndex,
        p1_index: PageIndex,
    ) -> Self {
        use crate::bits::BitOpt;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        addr.set_bits(12..21, u64::from(p1_index));
        Page::include_address(VirtAddr::new(addr))
    }
}

impl Page<Page2MB>{
    // 返回含有指定页索引内存页（2mb）
    pub fn from_page_table_indices_2mib(
        p4_index: PageIndex,
        p3_index: PageIndex,
        p2_index: PageIndex,
    ) -> Self {
        use crate::bits::BitOpt;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        Page::include_address(VirtAddr::new(addr))
    }
}

impl Page<Page1GB>{
    // 返回含有指定页索引内存页（1GiB）
    pub fn from_page_table_indices_1gib(
        p4_index: PageIndex,
        p3_index: PageIndex,
    ) -> Self {
        use crate::bits::BitOpt;
        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        Page::include_address(VirtAddr::new(addr))
    }
}

impl<S: NotGiantPageSize> Page<S> {
    pub fn p2_index(&self) -> PageIndex {
        self.start_address.page2_index()
    }
}
