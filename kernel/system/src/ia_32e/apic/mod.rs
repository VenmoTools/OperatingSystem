// code base on redox local_apic
use crate::ia_32e::cpu::apic::MSR;
use crate::ia_32e::cpu::msr::{IA32_APIC_BASE, IA32_X2APIC_SIVR, IA32_X2APIC_APICID, IA32_X2APIC_VERSION, IA32_X2APIC_ICR, IA32_X2APIC_EOI};
use crate::ia_32e::instructions::register::{rdmsr, wrmsr};
use raw_cpuid::CpuId;
use core::intrinsics::{volatile_load, volatile_store};

pub mod lvt;


#[derive(Debug)]
pub struct LocalAPIC {
    pub address: usize,
    pub x2apic: bool,
}

impl LocalAPIC {
    pub unsafe fn init() -> Self {
        let msr_addr = rdmsr(IA32_APIC_BASE);
        let has_x2 = CpuId::new().get_feature_info().unwrap().has_x2apic();
        Self {
            address: msr_addr as usize,
            x2apic: has_x2,
        }
    }

    pub unsafe fn init_ap(&self) {
        if self.x2apic {
            let mut msr = MSR::new(IA32_APIC_BASE);
            let value = msr.read();
            msr.write(value | 1 << 10);
            MSR::write_msr(IA32_X2APIC_SIVR, 0x100)
        } else {
            MSR::write_msr(0xF0, 0x100)
        }
    }

    pub unsafe fn read(&self, reg: u32) -> u32 {
        volatile_load((self.address + reg as usize) as *const u32)
    }

    pub unsafe fn write(&mut self, reg: u32, value: u32) {
        volatile_store((self.address + reg as usize) as *mut u32, value)
    }

    pub fn id(&self) -> u32 {
        if self.x2apic {
            rdmsr(IA32_X2APIC_APICID) as u32
        } else {
            unsafe { self.read(0x20) }
        }
    }

    pub fn version(&self) -> LocalAPICInfo {
        LocalAPICInfo::from(if self.x2apic {
            rdmsr(IA32_X2APIC_VERSION) as u32
        } else {
            unsafe { self.read(0x30) }
        })
    }

    pub fn set_icr(&mut self, value: u64) {
        if self.x2apic {
            unsafe { wrmsr(IA32_X2APIC_ICR, value); }
        } else {
            unsafe {
                while self.read(0x300) & 1 << 12 == 1 << 12 {}
                self.write(0x310, (value >> 32) as u32);
                self.write(0x300, value as u32);
                while self.read(0x300) & 1 << 12 == 1 << 12 {}
            }
        }
    }

    pub fn ipi(&mut self, apic_id: usize) {
        let mut icr = 0x4040;
        if self.x2apic {
            icr |= (apic_id as u64) << 32;
        } else {
            icr |= (apic_id as u64) << 56;
        }
        self.set_icr(icr);
    }

    pub unsafe fn eoi(&mut self) {
        if self.x2apic {
            wrmsr(IA32_X2APIC_EOI, 0);
        } else {
            self.write(0xB0, 0);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LocalAPICInfo {
    /// 版本ID
    vendor: u8,
    /// 保留位
    reserved_1: u8,
    /// LVT表项数目 此数值+1代表处理器支持的LVT表项数
    lvt_entry: u8,
    /// reserved_2中包含禁止广播EOI消息标志位因此取值只能为1或0
    reserved_2: u8,
}

impl From<u32> for LocalAPICInfo {
    fn from(data: u32) -> Self {
        let vendor = data as u8;
        let r_1 = (data << 16 >> 16 >> 8) as u8;
        let lvt = (data << 8 >> 8 >> 16) as u8;
        let r_2 = (data >> 24) as u8;
        Self {
            vendor,
            reserved_1: r_1,
            lvt_entry: lvt,
            reserved_2: r_2,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LocalAPICVendor {
    _82489DX,
    IntegratedAPIC,
}

