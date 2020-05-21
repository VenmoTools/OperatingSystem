use system::bits::EferFlags;
use system::ia_32e::cpu::apic::{Efer, MSR};
use system::ia_32e::cpu::msr::{IA32_FMASK, IA32_KERNEL_GS_BASE, IA32_LSTAR, IA32_STAR};

use crate::descriptor::{GDT, TSS};

pub unsafe fn init() {
    let selector = &GDT.1;
    MSR::write_msr(IA32_STAR, selector.kernel_code_selector.0 as u64);
    MSR::write_msr(IA32_LSTAR, selector.kernel_code_selector.0 as u64);
    MSR::write_msr(IA32_FMASK, 0x0300); // Clear trap flag and interrupt enable
    MSR::write_msr(IA32_KERNEL_GS_BASE, &TSS as *const _ as u64);
    MSR::write_msr(IA32_KERNEL_GS_BASE, &TSS as *const _ as u64);
    Efer::write(EferFlags::SYSTEM_CALL_EXTENSIONS);
}