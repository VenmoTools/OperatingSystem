use core::ops::{Deref, DerefMut};

use super::frame::Frame;
use super::page::{Page4KB, PageSize};

/// 物理帧分配器
pub unsafe trait FrameAllocator<S: PageSize> {
    /// 分配一个物理帧
    fn alloc(&mut self) -> Option<UnusedFrame<S>>;
    /// 释放一个物理帧
    fn dealloc(&mut self, frame: UnusedFrame<S>);
}

#[derive(Debug)]
pub struct UnusedFrame<S: PageSize = Page4KB>(Frame<S>);

impl<S: PageSize> UnusedFrame<S> {
    pub unsafe fn new(frame: Frame<S>) -> Self {
        UnusedFrame(frame)
    }
    pub fn frame(&self) -> Frame<S> {
        self.0
    }
}

impl<S: PageSize> Deref for UnusedFrame<S> {
    type Target = Frame<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnusedFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}