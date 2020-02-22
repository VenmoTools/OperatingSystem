use crate::ia_32e::PhysAddr;
use crate::ia_32e::paging::page::{PageSize, Page4KB, Page2MB, Page1GB};
use crate::ia_32e::paging::allocator::UnusedFrame;
use crate::ia_32e::paging::frame::Frame;

pub type Result<T> = core::result::Result<T, Error>;

pub enum TranslationResult {
    Frame4KB {
        frame: Frame<Page4KB>,
        offset: u64,
    },
    Frame2MB {
        frame: Frame<Page2MB>,
        offset: u64,
    },
    Frame1GB {
        frame: Frame<Page1GB>,
        offset: u64,
    },
    PageNotMapped,
    InvalidFrameAddress(PhysAddr),
}


#[derive(Debug)]
pub enum Error {
    CreatePageTableError,
    PageTableWalkError,
}

#[derive(Debug)]
pub enum CreatePageTableError {
    /// 映射的页过大
    MappedToHugePage,
    /// 物理帧分配错误
    FrameAllocateFailed,
}

#[derive(Debug)]
pub enum PageTableWalkError {
    /// 页面没有映射
    NotMapped,
    /// 映射过大的页面
    MappedToHugePage,
}

#[derive(Debug)]
pub enum MapToError<S: PageSize> {
    FrameAllocateFailed,
    ParentEntryHugePage,
    PageAlreadyMapped(UnusedFrame<S>),
}

#[derive(Debug)]
pub enum FlagUpdateError {
    PageNotMapped,
    ParentEntryHugePage,
}

#[derive(Debug)]
pub enum TranslateError {
    PageNotMapped,
    ParentEntryHugePage,
    InvalidFrameAddress(PhysAddr),
}

#[derive(Debug)]
pub enum UnmapError {
    PageNotMapped,
    ParentEntryHugePage,
    InvalidFrameAddress(PhysAddr),
}

#[derive(Debug)]
pub enum FrameError {
    /// 帧过大
    HugeFrame,
    /// 帧不存在， `PRESENT`没有被置位
    FrameNotPresent,
}


impl From<FrameError> for PageTableWalkError {
    fn from(e: FrameError) -> Self {
        match e {
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
        }
    }
}

impl From<CreatePageTableError> for MapToError<Page4KB> {
    fn from(err: CreatePageTableError) -> Self {
        match err {
            CreatePageTableError::MappedToHugePage => MapToError::ParentEntryHugePage,
            CreatePageTableError::FrameAllocateFailed => MapToError::FrameAllocateFailed,
        }
    }
}

impl From<CreatePageTableError> for MapToError<Page2MB> {
    fn from(err: CreatePageTableError) -> Self {
        match err {
            CreatePageTableError::MappedToHugePage => MapToError::ParentEntryHugePage,
            CreatePageTableError::FrameAllocateFailed => MapToError::FrameAllocateFailed,
        }
    }
}

impl From<CreatePageTableError> for MapToError<Page1GB> {
    fn from(err: CreatePageTableError) -> Self {
        match err {
            CreatePageTableError::MappedToHugePage => MapToError::ParentEntryHugePage,
            CreatePageTableError::FrameAllocateFailed => MapToError::FrameAllocateFailed,
        }
    }
}

impl From<PageTableWalkError> for UnmapError {
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => UnmapError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => UnmapError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for FlagUpdateError {
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => FlagUpdateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => FlagUpdateError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for TranslateError {
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => TranslateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => TranslateError::PageNotMapped,
        }
    }
}

