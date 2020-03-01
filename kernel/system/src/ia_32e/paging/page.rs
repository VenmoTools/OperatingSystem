use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::ia_32e::paging::PageIndex;
use crate::ia_32e::VirtAddr;

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

/// 页
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    /// 返回可遍历指定起始页面和（不包含）终止页面结构
    pub fn range_page(start: Self, end: Self) -> PageRange<S> {
        PageRange {
            start,
            end,
        }
    }
    /// 返回可遍历指定起始页面和（包含）终止页面结构
    pub fn range_include(start: Self, end: Self) -> PageRangeInclude<S> {
        PageRangeInclude {
            start,
            end,
        }
    }
}

/// 遍历指定起始页面和终止页面（不包含）。
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PageRange<S: PageSize = Page4KB> {
    /// 起始范围（包含）
    pub start: Page<S>,
    /// 终止范围（不包含）
    pub end: Page<S>,
}

impl<S: PageSize> PageRange<S> {
    /// 在指定返回内是否含有页面
    pub fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

impl<S: PageSize> Iterator for PageRange<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let page = self.start.clone();
            self.start += 1;
            return Some(page);
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PageRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageRangeInclude")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}


/// 遍历指定起始页面和终止页面（包含）。
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PageRangeInclude<S: PageSize = Page4KB> {
    /// 起始范围（包含）
    pub start: Page<S>,
    /// 终止范围（包含）
    pub end: Page<S>,
}

impl<S: PageSize> PageRangeInclude<S> {
    /// 在指定返回内是否含有页面
    pub fn is_empty(&self) -> bool {
        !(self.start <= self.end)
    }
}

impl<S: PageSize> Iterator for PageRangeInclude<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start.clone();
            self.start += 1;
            return Some(page);
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PageRangeInclude<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageRangeInclude")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

impl Page<Page4KB> {
    /// 返回当前页面的1级页表索引
    pub fn p1_index(&self) -> PageIndex {
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

impl Page<Page2MB> {
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

impl Page<Page1GB> {
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

impl<S: PageSize> fmt::Debug for Page<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Page[{}]({:#x})",
            S::DISPLAY_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Page::include_address(self.start_address() + rhs * u64::from(S::P_SIZE))
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Page::include_address(self.start_address() - rhs * u64::from(S::P_SIZE))
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl<S: PageSize> Sub<Self> for Page<S> {
    type Output = u64;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.start_address - rhs.start_address) / S::P_SIZE
    }
}
