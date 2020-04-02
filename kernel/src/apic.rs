use alloc::string::ToString;

use raw_cpuid::CpuId;
use system::bits::LocalAPICFlags;
use system::ia_32e::cpu::apic::MSR;
use system::ia_32e::cpu::msr::{APIC_BASE, IA32_X2APIC_APICID, IA32_X2APIC_PPR, IA32_X2APIC_TPR, IA32_X2APIC_VERSION};
use system::result::{DevicesErrorKind, Error};

#[derive(Debug, Copy, Clone)]
pub struct LocalAPICInfo {
    version: u8,
    max_lvt: u8,
    svr: bool,
}

impl LocalAPICInfo {
    pub fn from(data: u64) -> Self {
        Self {
            version: (data & 0xFF) as u8,
            max_lvt: (data >> 16 & 0xFF + 1) as u8,
            svr: data >> 24 & 0x1 != 0,
        }
    }
}

pub fn read_tpr() -> u64 {
    MSR::read_msr(IA32_X2APIC_TPR)
}

pub fn read_ppr() -> u64 {
    MSR::read_msr(IA32_X2APIC_PPR)
}

pub fn enable_xapic() {
    let base = MSR::read_msr(APIC_BASE) as u32;
    unsafe { MSR::write_msr(base, 1 << 10 | 1 << 11) };
}

pub fn enable_svr8() {
    let base = MSR::read_msr(APIC_BASE) as u32;
    unsafe { MSR::write_msr(base, 1 << 8 | 1 << 12) };
}

pub fn get_local_apic_id() -> u64 {
    MSR::read_msr(IA32_X2APIC_APICID)
}

pub fn get_local_apic_version() -> LocalAPICInfo {
    LocalAPICInfo::from(MSR::read_msr(IA32_X2APIC_VERSION))
}

pub fn disable_apic() {
    let base = MSR::read_msr(APIC_BASE) as u32;
    unsafe { MSR::write_msr(base, 0 << 11) };
}

pub fn init_local_apic() -> Result<(), Error> {
    let id = &CpuId::new();
    if !id.get_feature_info().map(|f| f.has_apic()).unwrap() {
        return Err(Error::new_devices(DevicesErrorKind::NotSupport, Some("apic not support".to_string())));
    }
    enable_xapic();
    enable_svr8();
    Ok(())
}

pub struct LocalApic(u64);

impl LocalApic {
    pub fn apic_base(&self) -> u32 {
        (self.0 >> 12 & 0xFF_FFFF) as u32
    }

    pub fn is_enabled_bsp(&self) -> bool {
        LocalAPICFlags::from_bits_truncate(self.0 as u32).contains(LocalAPICFlags::BSP)
    }

    pub fn is_enabled(&self) -> bool {
        LocalAPICFlags::from_bits_truncate(self.0 as u32).contains(LocalAPICFlags::APIC_GLOBAL_ENABLE)
    }
}