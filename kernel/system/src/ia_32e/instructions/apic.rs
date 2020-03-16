use crate::ia_32e::instructions::register::wrmsr2;

unsafe fn enable_local_apic() {
    asm!(
    "movq $0x1b ,%rax;\
     rdmsr;\
     bts $10,%rax;\
     bts $11,%rax;\
     wrmsr;"
    )
}

/// 屏蔽LVT中所有中断投递功能，该函数仅用于系统为给LVT配备处理程序时
unsafe fn mask_all_lvt() {
    use crate::ia_32e::cpu::msr::{
        IA32_X2APIC_IRR0,
        IA32_X2APIC_LVT_TIMER,
        IA32_X2APIC_LVT_THERMAL,
        IA32_X2APIC_LVT_PMI,
        IA32_X2APIC_LVT_LINT0,
        IA32_X2APIC_LVT_LINT1,
        IA32_X2APIC_LVT_ERROR,
    };
    // CMCI
    wrmsr2(IA32_X2APIC_IRR0, 0x10000, 0x00);
    // Timer
    wrmsr2(IA32_X2APIC_LVT_TIMER, 0x10000, 0x00);
    // Thermal Monitor
    wrmsr2(IA32_X2APIC_LVT_THERMAL, 0x10000, 0x00);
    // Performance Counter
    wrmsr2(IA32_X2APIC_LVT_PMI, 0x10000, 0x00);
    // LINT0
    wrmsr2(IA32_X2APIC_LVT_LINT0, 0x10000, 0x00);
    // LINT1
    wrmsr2(IA32_X2APIC_LVT_LINT1, 0x10000, 0x00);
    // Error
    wrmsr2(IA32_X2APIC_LVT_ERROR, 0x10000, 0x00);
}