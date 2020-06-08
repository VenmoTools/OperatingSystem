use crate::devices::bus::pic::{CfgSpaceReader, CfgSpaceWriter, PicAccess};
use crate::ia_32e::cpu::Port;
use super::{CONFIG_ADDRESS, CONFIG_DATA};


pub struct ConfigAddressPort {
    port: Port<u32>
}

impl ConfigAddressPort {
    pub fn new() -> Self {
        Self {
            port: unsafe { Port::new(CONFIG_ADDRESS) }
        }
    }

    pub unsafe fn read(&mut self, addr: &DeviceAddress) -> u32 {
        self.write(addr);
        self.port.read()
    }

    pub unsafe fn write(&mut self, addr: &DeviceAddress) {
        self.port.write(addr.address())
    }
}

pub struct ConfigDataPort {
    port: Port<u32>
}

impl ConfigDataPort {
    pub fn new() -> Self {
        Self {
            port: unsafe { Port::new(CONFIG_DATA) }
        }
    }

    pub unsafe fn write(&mut self, addr: &DeviceAddress, value: u32) {
        let mut cfg = ConfigAddressPort::new();
        cfg.write(addr);
        self.port.write(value)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DeviceAddress {
    bus: u8,
    slots: u8,
    func: u8,
    offset: u8,
}

impl DeviceAddress {
    pub fn new(bus: u8, slots: u8, func: u8, offset: u8) -> Self {
        Self {
            bus,
            slots,
            func,
            offset,
        }
    }
    pub fn address(&self) -> u32 {
        0x80000000 | (u32::from(self.bus) << 16) | (u32::from(self.slots) << 11) | (u32::from(self.func) << 8) | u32::from(self.offset)
    }
}

//////////////////////
///// Pic
/////////////////////

pub struct Pic;

impl Pic {
    pub fn new() -> Self{
        Self
    }
}


impl Pic {
    pub fn bus_iter(&self) -> PicIter {
        PicIter::new(self)
    }
}

unsafe impl PicAccess for Pic {
    unsafe fn read(&self, addr: &DeviceAddress) -> u32 {
        let mut cfg = ConfigAddressPort::new();
        cfg.read(addr)
    }

    unsafe fn write(&self, addr: &DeviceAddress, value: u32) {
        let mut data_port = ConfigDataPort::new();
        data_port.write(addr, value);
    }
}

pub struct PicIter<'pic> {
    pic: &'pic dyn PicAccess,
    num: u32,
}

impl<'pic> PicIter<'pic> {
    pub fn new(pic: &'pic dyn PicAccess) -> Self {
        Self {
            pic,
            num: 0,
        }
    }
}

impl<'pic> Iterator for PicIter<'pic> {
    type Item = PicBus<'pic>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 255 {
            let bus = PicBus {
                pic: self.pic,
                num: self.num as u8,
            };
            self.num += 1;
            Some(bus)
        } else {
            None
        }
    }
}

//////////////////////
///// Pic Bus
/////////////////////
pub struct PicBus<'pic> {
    pub pic: &'pic dyn PicAccess,
    pub num: u8,
}

unsafe impl<'pic> PicAccess for PicBus<'pic> {
    unsafe fn read(&self, addr: &DeviceAddress) -> u32 {
        self.pic.read(addr)
    }

    unsafe fn write(&self, addr: &DeviceAddress, value: u32) {
        self.pic.write(addr, value)
    }
}


impl<'pic> PicBus<'pic> {
    pub fn devices(&self) -> PicBusIter {
        PicBusIter::new(self)
    }

    pub fn address(&self, slots: u8, func_no: u8, offset: u8) -> DeviceAddress {
        DeviceAddress::new(self.num, slots, func_no, offset)
    }

    pub unsafe fn read(&self, slots: u8, func_no: u8, offset: u8) -> u32 {
        self.pic.read(&self.address(slots, func_no, offset))
    }
    pub unsafe fn write(&self, slots: u8, func_no: u8, offset: u8, value: u32) {
        self.pic.write(&self.address(slots, func_no, offset), value)
    }
}

pub struct PicBusIter<'bus> {
    pic: &'bus PicBus<'bus>,
    num: u32,
}

impl<'bus> PicBusIter<'bus> {
    pub fn new(pic: &'bus PicBus<'bus>) -> Self {
        Self {
            pic,
            num: 0,
        }
    }
}

impl<'bus> Iterator for PicBusIter<'bus> {
    type Item = PicDev<'bus>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 32 {
            let dev = PicDev {
                pic: self.pic,
                num: self.num as u8,
            };
            self.num += 1;
            Some(dev)
        } else {
            None
        }
    }
}


//////////////////////
///// Pic devices
/////////////////////
pub struct PicDev<'dev> {
    pub pic: &'dev PicBus<'dev>,
    pub num: u8,
}

impl<'dev> PicDev<'dev> {
    pub fn funcs(&self) -> PicDevIter {
        PicDevIter::new(self)
    }
    pub fn address(&self, func_no: u8, offset: u8) -> DeviceAddress {
        self.pic.address(self.num, func_no, offset)
    }

    pub unsafe fn read(&self, func_no: u8, offset: u8) -> u32 {
        self.pic.read(self.num, func_no, offset)
    }

    pub unsafe fn write(&self, func_no: u8, offset: u8, value: u32) {
        self.pic.write(self.num, func_no, offset, value)
    }
}

pub struct PicDevIter<'dev> {
    pic: &'dev PicDev<'dev>,
    num: u32,
}

impl<'dev> PicDevIter<'dev> {
    pub fn new(dev: &'dev PicDev<'dev>) -> Self {
        Self {
            pic: dev,
            num: 0,
        }
    }
}

impl<'dev> Iterator for PicDevIter<'dev> {
    type Item = PicFunc<'dev>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 8 {
            let func = PicFunc {
                dev: self.pic,
                num: self.num as u8,
            };
            self.num += 1;
            Some(func)
        } else {
            None
        }
    }
}

//////////////////////
///// Pic functions
/////////////////////
pub struct PicFunc<'dev> {
    pub dev: &'dev PicDev<'dev>,
    pub num: u8,
}

impl<'dev> PicFunc<'dev> {
    pub fn address(&self, offset: u8) -> DeviceAddress {
        self.dev.address(self.num, offset)
    }
}

impl<'dev> CfgSpaceReader for PicFunc<'dev> {
    unsafe fn read(&self, offset: u8) -> u32 {
        self.dev.read(self.num, offset)
    }
}

impl<'dev> CfgSpaceWriter for PicFunc<'dev> {
    unsafe fn write(&self, offset: u8, val: u32) {
        self.dev.write(self.num, offset, val)
    }
}
