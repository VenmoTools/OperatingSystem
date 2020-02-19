use crate::bitflags::bitflags;
use crate::bits::{CR0Flags, CR3Flags, CR4Flags};
use crate::ia_32e::{PhysAddr, VirtAddr};
use crate::ia_32e::instructions::register::read_cr4;
use crate::ia_32e::paging::frame::Frame;

use super::super::instructions::register::{read_cr0, read_cr2, read_cr3, write_cr0, write_cr2, write_cr3};

#[derive(Debug)]
pub struct CR0;

impl CR0 {
    pub fn read() -> CR0Flags {
        unsafe {
            CR0Flags::from_bits_truncate(read_cr0())
        }
    }

    pub fn read_raw() -> u64 {
        unsafe {
            read_cr0()
        }
    }

    pub unsafe fn write(flags: CR0Flags) {
        let old = Self::read_raw();
        let reserved = old & !(CR0Flags::all().bits());
        let new = reserved | flags.bits();
        write_cr0(new)
    }

    pub unsafe fn write_raw(data: u64) {
        write_cr0(data)
    }
}

pub struct CR2;

impl CR2 {
    /// 从当前的CR2寄存器中读取线性地址
    pub fn read() -> VirtAddr {
        unsafe {
            VirtAddr::new(read_cr2())
        }
    }
}

pub struct CR3;

impl CR3 {
    /// 从CR3寄存器中读取读取当前P4页表的地址
    pub fn read() -> (Frame, CR3Flags) {
        let data = unsafe { read_cr3() };
        let flags = CR3Flags::from_bits_truncate(data);
        let addr = PhysAddr::new(data & 0x00F_FFFF_FFFF_F000);
        let frame = Frame::include_address(addr);
        (frame, flags)
    }

    /// 将新的P4页表地址写入CR3寄存器中
    pub unsafe fn write(frame: Frame, flags: CR3Flags) {
        let addr = frame.start_address();
        let data = addr.as_u64() | flags.bits();
        write_cr3(data)
    }
}

pub struct CR4;

impl CR4 {
    /// 读取当前CR4寄存器的Flag
    pub fn read() -> CR4Flags {
        CR4Flags::from_bits_truncate(Self::read_raw())
    }

    /// 读取当前寄存器的值
    pub fn read_raw() -> u64 {
        unsafe {
            read_cr4()
        }
    }

    /// 向CR4寄存器写入原始u64数据
    pub unsafe fn write_raw(data: u64) {
        write_cr2(data)
    }

    /// 向CR4寄存器写入 Flags数据
    pub unsafe fn write(flags: CR4Flags) {
        let old = Self::read_raw();
        let reserved = old & !(CR4Flags::all().bits());
        let new = reserved | flags.bits();
        Self::write_raw(new)
    }
}