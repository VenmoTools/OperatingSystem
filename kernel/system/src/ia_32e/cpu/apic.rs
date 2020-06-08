use crate::bits::EferFlags;
use crate::ia_32e::instructions::register::{rdmsr, wrmsr};
use crate::ia_32e::VirtAddr;
use crate::bits::flags::LocalAPICFlags;


#[derive(Debug)]
pub struct MSR(u32);

impl MSR {

    pub fn new(n: u32) -> Self {
        MSR(n)
    }

    pub fn read(&self) -> u64 {
        rdmsr(self.0)
    }

    pub unsafe fn write(&mut self, data: u64) {
        wrmsr(self.0, data)
    }

    pub unsafe fn write_msr(msr:u32,value:u64){
        wrmsr(msr,value)
    }

    pub fn read_msr(msr:u32) -> u64{
        rdmsr(msr)
    }
}

/// The Extended Feature Enable Register.
#[derive(Debug)]
pub struct Efer;

/// FS.Base Model Specific Register.
#[derive(Debug)]
pub struct FsBase;

/// GS.Base Model Specific Register.
#[derive(Debug)]
pub struct GsBase;

/// KernelGsBase Model Specific Register.
#[derive(Debug)]
pub struct KernelGsBase;

impl Efer {
    /// The underlying model specific register.
    pub const MSR: MSR = MSR(0xC0000080);

    pub fn read() -> EferFlags {
        EferFlags::from_bits_truncate(Self::read_raw())
    }

    pub fn enable_long_mode() -> bool {
        Self::read().contains(EferFlags::LONG_MODE_ENABLE)
    }

    pub fn read_raw() -> u64 {
        Self::MSR.read()
    }

    pub unsafe fn write(flags: EferFlags) {
        let old_value = Self::read_raw();
        let reserved = old_value & !(EferFlags::all().bits());
        let new_value = reserved | flags.bits();
        Self::write_raw(new_value);
    }

    pub unsafe fn write_raw(value: u64) {
        Self::MSR.write(value)
    }
}

impl FsBase {
    /// The underlying model specific register.
    pub const MSR: MSR = MSR(0xC000_0100);

    /// Read the current FsBase register.
    pub fn read() -> VirtAddr {
        VirtAddr::new(Self::MSR.read())
    }

    /// Write a given virtual address to the FS.Base register.
    pub fn write(address: VirtAddr) {
        unsafe { Self::MSR.write(address.as_u64()) };
    }
}

impl GsBase {
    /// The underlying model specific register.
    pub const MSR: MSR = MSR(0xC000_0101);
    /// Read the current GsBase register.
    pub fn read() -> VirtAddr {
        VirtAddr::new(Self::MSR.read())
    }

    /// Write a given virtual address to the GS.Base register.
    pub fn write(address: VirtAddr) {
        unsafe { Self::MSR.write(address.as_u64()) };
    }
}

impl KernelGsBase {
    /// The underlying model specific register.
    pub const MSR: MSR = MSR(0xC000_0102);

    /// Read the current KernelGsBase register.
    pub fn read() -> VirtAddr {
        VirtAddr::new(Self::MSR.read())
    }

    /// Write a given virtual address to the KernelGsBase register.
    pub fn write(address: VirtAddr) {
        unsafe { Self::MSR.write(address.as_u64()) };
    }
}


//////////////////////////////////////////
///// Local APIC Register
//////////////////////////////////////////

/// 错误状态寄存器
#[derive(Debug)]
pub struct ESR(u32);

impl ESR {
    pub fn new(n: u32) -> Self {
        Self(n)
    }

    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) -> LocalAPICFlags {
        LocalAPICFlags::from_bits_truncate(self.0 as u8)
    }
}

/// Task Priority Register 任务优先权寄存器
#[derive(Debug)]
pub struct TPR;

impl TPR {
    pub fn read() -> u32 {
        rdmsr(0x808) as u32
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// Processor Priority Register 处理器优先权寄存器 只读
#[derive(Debug)]
pub struct PPR;

impl PPR {
    pub fn read() -> u32 {
        rdmsr(0x80a) as u32
    }
}

#[derive(Debug)]
pub struct CR8;

impl CR8 {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// Interrupt Request Register 中断请求处理器 256位
#[derive(Debug)]
pub struct IRR;

impl IRR {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// In-Service Register 正在服务寄存器 256位
#[derive(Debug)]
pub struct ISR;

impl ISR {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// Trigger Mode Register 触发模式寄存器 256位
#[derive(Debug)]
pub struct TMR;

impl TMR {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// End Of Interrupt 中断结束寄存器 32位
#[derive(Debug)]
pub struct EOI;

impl EOI {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

/// Spurious Interrupt Vector Register 伪中断向量寄存器 32位
#[derive(Debug)]
pub struct SVR;

impl SVR {
    pub fn read() -> u32 {
        0
    }

    pub unsafe fn write() {}

    pub fn flags(&self) {}
}

//////////////////////////////////////////
///// I/O APIC Register
//////////////////////////////////////////

/// IO APIC 间接索引寄存器
pub struct IOREGSEL(u32);

impl IOREGSEL {
    pub fn read() -> u32 { 0 }

    pub unsafe fn write() {}
}

/// IO APIC 数据操作寄存器
pub struct IOWIN(u32);

impl IOWIN {
    pub fn read() -> u32 { 0 }

    pub unsafe fn write() {}
}

/// IO APIC 中断结束寄存器 只写
pub struct IOEOI(u32);

impl IOEOI {
    pub unsafe fn write() {}
}

/// IO APIC ID寄存器
pub struct IOAPICID(u32);

impl IOAPICID {
    pub fn read() -> u32 { 0 }

    pub unsafe fn write() {}
}

/// IO APIC 版本寄存器 只读
pub struct IOAPICVER(u32);

impl IOAPICVER {
    pub fn read() -> u32 { 0 }
}

/// IO APIC IO中断定向投递寄存器组 共24个，索引0-23
pub struct IOREDTBL(u32);

impl IOREDTBL {
    pub fn read() -> u32 { 0 }
    pub unsafe fn write() {}
}
