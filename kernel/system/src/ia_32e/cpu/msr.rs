use crate::bits::EferFlags;
use crate::ia_32e::instructions::register::{rdmsr, wrmsr};
use crate::ia_32e::VirtAddr;

#[derive(Debug)]
pub struct MSR(u32);

impl MSR {
    pub fn new(n: u32) -> Self {
        MSR(n)
    }

    pub unsafe fn read(&self) -> u64 {
        rdmsr(self.0)
    }

    pub unsafe fn write(&mut self, data: u64) {
        wrmsr(self.0, data)
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

    pub fn read_raw() -> u64 {
        unsafe {
            Self::MSR.read()
        }
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
        VirtAddr::new(unsafe { Self::MSR.read() })
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
        VirtAddr::new(unsafe { Self::MSR.read() })
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
        VirtAddr::new(unsafe { Self::MSR.read() })
    }

    /// Write a given virtual address to the KernelGsBase register.
    pub fn write(address: VirtAddr) {
        unsafe { Self::MSR.write(address.as_u64()) };
    }
}