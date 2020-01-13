use crate::ia_32e::VirtAddr;
use bitflags::_core::marker::PhantomData;

/// 针对3种不同的页大小的抽象
pub trait PageSize: Copy + Eq + PartialEq + Ord {
    const P_SIZE: u64;
    const DISPLAY_STR: &'static str;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page4KB {}

impl PageSize for Page4KB {
    const P_SIZE: u64 = 4096;
    const DISPLAY_STR: &'static str = "page 4 kb";
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page1MB {}

impl PageSize for Page1MB {
    const P_SIZE: u64 = Page4KB::P_SIZE * 512;
    const DISPLAY_STR: &'static str = "page 1 MB";
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Page1GB {}

impl PageSize for Page1GB {
    const P_SIZE: u64 = Page1MB::P_SIZE * 512;
    const DISPLAY_STR: &'static str = "page 1 GB";
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct Page<S: PageSize = Page4KB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    pub const SIZE: u64 = S::P_SIZE;

    /// 返回给定地址的页表
    /// 如果地址没有正确对齐将会返回错误
    pub fn from_start_address(address: VirtAddr) -> Result<Self, ()> {
        if !address.is_aligned(S::P_SIZE) {
            return Err(());
        }
        Ok(Page::containing_address(address))
    }
    /// 返回包含给定地址的页表
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::P_SIZE),
            size: PhantomData,
        }
    }

    pub fn start_address(&self) -> VirtAddr{
        self.start_address
    }

    pub const fn size(&self) -> u64{
        S::P_SIZE
    }
}
