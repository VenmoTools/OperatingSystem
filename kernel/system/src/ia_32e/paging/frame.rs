use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::ia_32e::PhysAddr;
use crate::result::{Error, Result};
use crate::result::error::mem::MemErrorKind;

use super::page::{Page4KB, PageSize};

/// 物理内存帧
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Frame<S: PageSize = Page4KB> {
    start_addr: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Frame<S> {
    /// 返回给定物理地址`addr`的起始地址所在的物理帧
    ///
    /// # Error
    /// 当给定地址没有进行对齐是返回`NotAligned`错误
    pub fn from_start_addr(addr: PhysAddr) -> Result<Self> {
        if !addr.is_aligned(S::P_SIZE) {
            return Err(Error::new_memory(MemErrorKind::NotAligned, format!("the address `{:#X}` is not align by {}", addr.as_u64(), S::P_SIZE)));
        }
        Ok(Self::include_address(addr))
    }
    /// 返回包含给定的物理地址的物理帧
    pub fn include_address(addr: PhysAddr) -> Self {
        Frame {
            start_addr: addr.align_down(S::P_SIZE),
            size: PhantomData,
        }
    }

    /// 返回帧的起始地址
    pub fn start_address(&self) -> PhysAddr {
        self.start_addr
    }

    /// 返回当前物理帧大小 4KB 2MB, 1GB
    pub fn size(&self) -> u64 {
        S::P_SIZE
    }

    /// 遍历给定的起始帧到终止帧范围，包含终止帧
    pub fn frame_range_include(start: Self, end: Self) -> FrameRangeInclude<S> {
        FrameRangeInclude {
            start,
            end,
        }
    }
    /// 遍历给定的起始帧到终止帧范围，包含终止帧
    pub fn frame_range(start: Self, end: Self) -> FrameRange<S> {
        FrameRange {
            start,
            end,
        }
    }
}

/// 物理内存范围
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FrameRangeInclude<S: PageSize = Page4KB> {
    /// 起始.
    pub start: Frame<S>,
    /// 终止包含.
    pub end: Frame<S>,
}

impl<S: PageSize> FrameRangeInclude<S> {
    pub fn is_empty(&self) -> bool {
        !(self.start <= self.end)
    }
}

impl<S: PageSize> Iterator for FrameRangeInclude<S> {
    type Item = Frame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for FrameRangeInclude<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FrameRangeInclude")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// 物理内存范围
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FrameRange<S: PageSize = Page4KB> {
    /// 起始.
    pub start: Frame<S>,
    /// 终止不包含.
    pub end: Frame<S>,
}

impl<S: PageSize> FrameRange<S> {
    pub fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

impl<S: PageSize> Iterator for FrameRange<S> {
    type Item = Frame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for FrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}


impl<S: PageSize> fmt::Debug for Frame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Physical Frame[{}]({:#X})",
            S::DISPLAY_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for Frame<S> {
    type Output = Self;
    // 进行加法操作时需要按照给定给定对齐方式进行
    fn add(self, rhs: u64) -> Self::Output {
        Frame::include_address(self.start_address() + rhs * u64::from(S::P_SIZE))
    }
}

impl<S: PageSize> AddAssign<u64> for Frame<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Frame<S> {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Frame::include_address(self.start_address() - rhs * u64::from(S::P_SIZE))
    }
}

impl<S: PageSize> SubAssign<u64> for Frame<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl<S: PageSize> Sub<Frame> for Frame<S> {
    type Output = u64;
    // 计算两个帧的差值结果为2个帧的相隔的帧数
    fn sub(self, rhs: Frame<Page4KB>) -> Self::Output {
        (self.start_addr - rhs.start_addr) / S::P_SIZE
    }
}

