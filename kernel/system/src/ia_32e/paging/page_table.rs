use core::fmt;
use core::ops::{Index, IndexMut};

use crate::bits::PageTableFlags;
use crate::ia_32e::PhysAddr;

use super::PageIndex;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64
}

impl PageTableEntry {
    /// 创建一个空的页表实体
    pub const fn new() -> Self {
        PageTableEntry { entry: 0 }
    }
    /// 判断页表实体是否被使用
    pub const fn is_unused(&self) -> bool {
        self.entry == 0
    }
    /// 将页表实体设置位未使用
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }
    /// 获取当前实体的bitmap
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }
    /// 获取当前实体所映射的物理地址
    pub fn addr(&self) -> PhysAddr{
        PhysAddr::new( self.entry & 0x000fffff_fffff000)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

pub const ENTRY_COUNT: usize = 512;

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// 创建一个空的页表
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); ENTRY_COUNT]
        }
    }
    /// 清空表中所有内容
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
    /// 获取只读迭代器
    pub fn iter(&self) -> impl Iterator<Item=&PageTableEntry> {
        self.entries.iter()
    }
    /// 获取可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

// ----------------- 为 Page实现索引功能 支持usize索引和PageIndex索引 -----------------
impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageIndex> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: PageIndex) -> &Self::Output {
        &self.entries[cast::usize(u16::from(index))]
    }
}

impl IndexMut<PageIndex> for PageTable {
    fn index_mut(&mut self, index: PageIndex) -> &mut Self::Output {
        &mut self.entries[cast::usize(u16::from(index))]
    }
}

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}