use crate::bits::BitOpt;
use crate::alloc::vec::Vec;
use byteorder::LittleEndian;
use byteorder::ByteOrder;

use core::fmt;
use crate::devices::bus::pic::dev::DeviceAddress;

pub mod class;
pub mod dev;
pub mod header;
pub mod data;

pub const CONFIG_ADDRESS:u16 = 0xCF8;

pub const CONFIG_DATA:u16 = 0xCFC;


pub unsafe trait PicAccess{

    unsafe fn read(&self,addr: &DeviceAddress) -> u32;

    unsafe fn write(&self,addr: &DeviceAddress,value:u32);
}

pub trait CfgSpaceReader{

    /// len > 3 && len % 4 == 0
    unsafe fn read_range(&self,offset:u8,len:u16) -> Vec<u8>{
        let mut res = Vec::with_capacity(len as usize);
        let offset = offset as u16;
        let data = (offset..offset+len).step_by(4).fold(Vec::new(),|mut acc,offset|{
            let val = self.read(offset as u8);
            acc.push(val);
            acc
        });
        res.set_len(len as usize);
        LittleEndian::write_u32_into(data.as_slice(),&mut res);
        res
    }

    unsafe  fn read(&self,offset: u8) -> u32;

    unsafe fn read_u8(&self, offset: u8) -> u8 {
        let dword_offset = (offset / 4) * 4;
        let dword = self.read(dword_offset);

        let shift = (offset % 4) * 8;
        ((dword >> shift) & 0xFF) as u8
    }
}

pub trait CfgSpaceWriter{
    unsafe fn write(&self,offset:u8,val:u32);
}

// When you want to retrieve the actual base address of a BAR, be sure to mask the lower bits.
// For 16-Bit Memory Space BARs, you calculate (BAR[x] & 0xFFF0).
// For 32-Bit Memory Space BARs, you calculate (BAR[x] & 0xFFFFFFF0).
// For 64-Bit Memory Space BARs, you calculate ((BAR[x] & 0xFFFFFFF0) + ((BAR[x+1] & 0xFFFFFFFF) << 32))
// For I/O Space BARs, you calculate (BAR[x] & 0xFFFFFFFC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PciBaseAddress {
    None,
    Memory(u32),
    Port(u16)
}

#[derive(Debug,Copy, Clone,Eq, PartialEq)]
pub struct MemorySpaceBarLayout(u32);

impl MemorySpaceBarLayout {
    pub fn new(layout:u32) -> Self{
        Self(layout)
    }

    pub fn base_address(&self) -> u32{
        self.0.get_bits(4..32)
    }

    pub fn types(&self) -> u8{
        self.0.get_bits(1..3) as u8
    }

    pub fn prefetchable(&self) -> bool{
        self.0.get_bit(3)
    }
}


impl PciBaseAddress {
    pub fn is_none(&self) -> bool {
        match self {
            &PciBaseAddress::None => true,
            _ => false,
        }
    }
    pub fn is_some(&self) -> bool{
        !self.is_none()
    }
}


impl From<u32> for PciBaseAddress {
    fn from(bar: u32) -> Self {
        if bar & 0xFFFFFFFC == 0 {
            PciBaseAddress::None
        } else if bar & 1 == 0 { // in Memory Space BAR Layout  bit0 Always 0
            PciBaseAddress::Memory(bar & 0xFFFFFFF0)
        } else { // in I/O Space BAR Layout  bit0 Always 1
            PciBaseAddress::Port((bar & 0xFFFC) as u16)
        }
    }
}

impl fmt::Display for PciBaseAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PciBaseAddress::Memory(address) => write!(f, "{:>08X}", address),
            &PciBaseAddress::Port(address) => write!(f, "{:>04X}", address),
            &PciBaseAddress::None => write!(f, "None")
        }
    }
}


pub fn get_offset(offset:u8) -> u16{
    ( (offset as u16 & 2) * 8) & 0xffff
}