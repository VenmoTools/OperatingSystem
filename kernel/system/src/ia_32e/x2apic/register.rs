/// this code base on https://github.com/kwzhao/x2apic-rs
use crate::ia_32e::cpu::apic::MSR;
use crate::ia_32e::cpu::msr::{IA32_APIC_BASE, IA32_X2APIC_APICID, IA32_X2APIC_VERSION, IA32_X2APIC_TPR, IA32_X2APIC_PPR, IA32_X2APIC_EOI, IA32_X2APIC_LDR, IA32_X2APIC_SIVR, IA32_X2APIC_ISR0, IA32_X2APIC_ISR1, IA32_X2APIC_ISR2, IA32_X2APIC_ISR3, IA32_X2APIC_ISR4, IA32_X2APIC_ISR5, IA32_X2APIC_ISR6, IA32_X2APIC_ISR7, IA32_X2APIC_TMR0, IA32_X2APIC_TMR1, IA32_X2APIC_TMR2, IA32_X2APIC_TMR3, IA32_X2APIC_TMR4, IA32_X2APIC_TMR5, IA32_X2APIC_TMR6, IA32_X2APIC_TMR7, IA32_X2APIC_IRR0, IA32_X2APIC_IRR1, IA32_X2APIC_IRR2, IA32_X2APIC_IRR3, IA32_X2APIC_IRR4, IA32_X2APIC_IRR5, IA32_X2APIC_IRR6, IA32_X2APIC_IRR7, IA32_X2APIC_ESR, IA32_X2APIC_ICR, IA32_X2APIC_LVT_TIMER, IA32_X2APIC_LVT_THERMAL, IA32_X2APIC_LVT_PMI, IA32_X2APIC_LVT_LINT0, IA32_X2APIC_LVT_LINT1, IA32_X2APIC_LVT_ERROR, IA32_X2APIC_INIT_COUNT, IA32_X2APIC_CUR_COUNT, IA32_X2APIC_DIV_CONF, IA32_X2APIC_SELF_IPI};
use core::ops::Range;
use paste;
use bit::BitIndex;


#[derive(Debug)]
pub struct LocalApicRegisters {
    base: MSR,
    id: MSR,
    version: MSR,
    tpr: MSR,
    ppr: MSR,
    eoi: MSR,
    ldr: MSR,
    sivr: MSR,
    isr0: MSR,
    isr1: MSR,
    isr2: MSR,
    isr3: MSR,
    isr4: MSR,
    isr5: MSR,
    isr6: MSR,
    isr7: MSR,
    tmr0: MSR,
    tmr1: MSR,
    tmr2: MSR,
    tmr3: MSR,
    tmr4: MSR,
    tmr5: MSR,
    tmr6: MSR,
    tmr7: MSR,
    irr0: MSR,
    irr1: MSR,
    irr2: MSR,
    irr3: MSR,
    irr4: MSR,
    irr5: MSR,
    irr6: MSR,
    irr7: MSR,
    error: MSR,
    icr: MSR,
    lvt_timer: MSR,
    lvt_thermal: MSR,
    lvt_perf: MSR,
    lvt_lint0: MSR,
    lvt_lint1: MSR,
    lvt_error: MSR,
    ticr: MSR,
    tccr: MSR,
    tdcr: MSR,
    self_ipi: MSR,
}

macro_rules! read {
    ($name:ident) => {
        paste::item! {
            pub unsafe fn $name(&self) -> u64 {
                self.$name.read()
            }

            pub unsafe fn [<$name _bit>](&self, bit: usize) -> bool {
                self.$name().bit(bit)
            }

            pub unsafe fn [<$name _bit_range>](
                &self,
                pos: Range<usize>,
            ) -> u64 {
                self.$name().bit_range(pos)
            }
        }
    };
}

macro_rules! write {
    ($name:ident) => {
        paste::item! {
            pub unsafe fn [<write_ $name>](&mut self, value: u64) {
                self.$name.write(value);
            }
        }
    };
}

macro_rules! read_write {
    ($name:ident) => {
        read!($name);
        write!($name);

        paste::item! {
            pub unsafe fn [<set_ $name _bit>](
                &mut self,
                bit: usize,
                val: bool,
            ) {
                let mut reg_val = self.$name();

                reg_val.set_bit(bit, val);

                self.[<write_ $name>](reg_val);
            }

            pub unsafe fn [<set_ $name _bit_range>](
                &mut self,
                pos: Range<usize>,
                val: u64,
            ) {
                let mut reg_val = self.$name();

                reg_val.set_bit_range(pos, val);

                self.[<write_ $name>](reg_val);
            }
        }
    };
}

impl LocalApicRegisters {
    pub fn new() -> Self {
        LocalApicRegisters {
            base: MSR::new(IA32_APIC_BASE),
            id: MSR::new(IA32_X2APIC_APICID),
            version: MSR::new(IA32_X2APIC_VERSION),
            tpr: MSR::new(IA32_X2APIC_TPR),
            ppr: MSR::new(IA32_X2APIC_PPR),
            eoi: MSR::new(IA32_X2APIC_EOI),
            ldr: MSR::new(IA32_X2APIC_LDR),
            sivr: MSR::new(IA32_X2APIC_SIVR),
            isr0: MSR::new(IA32_X2APIC_ISR0),
            isr1: MSR::new(IA32_X2APIC_ISR1),
            isr2: MSR::new(IA32_X2APIC_ISR2),
            isr3: MSR::new(IA32_X2APIC_ISR3),
            isr4: MSR::new(IA32_X2APIC_ISR4),
            isr5: MSR::new(IA32_X2APIC_ISR5),
            isr6: MSR::new(IA32_X2APIC_ISR6),
            isr7: MSR::new(IA32_X2APIC_ISR7),
            tmr0: MSR::new(IA32_X2APIC_TMR0),
            tmr1: MSR::new(IA32_X2APIC_TMR1),
            tmr2: MSR::new(IA32_X2APIC_TMR2),
            tmr3: MSR::new(IA32_X2APIC_TMR3),
            tmr4: MSR::new(IA32_X2APIC_TMR4),
            tmr5: MSR::new(IA32_X2APIC_TMR5),
            tmr6: MSR::new(IA32_X2APIC_TMR6),
            tmr7: MSR::new(IA32_X2APIC_TMR7),
            irr0: MSR::new(IA32_X2APIC_IRR0),
            irr1: MSR::new(IA32_X2APIC_IRR1),
            irr2: MSR::new(IA32_X2APIC_IRR2),
            irr3: MSR::new(IA32_X2APIC_IRR3),
            irr4: MSR::new(IA32_X2APIC_IRR4),
            irr5: MSR::new(IA32_X2APIC_IRR5),
            irr6: MSR::new(IA32_X2APIC_IRR6),
            irr7: MSR::new(IA32_X2APIC_IRR7),
            error: MSR::new(IA32_X2APIC_ESR),
            icr: MSR::new(IA32_X2APIC_ICR),
            lvt_timer: MSR::new(IA32_X2APIC_LVT_TIMER),
            lvt_thermal: MSR::new(IA32_X2APIC_LVT_THERMAL),
            lvt_perf: MSR::new(IA32_X2APIC_LVT_PMI),
            lvt_lint0: MSR::new(IA32_X2APIC_LVT_LINT0),
            lvt_lint1: MSR::new(IA32_X2APIC_LVT_LINT1),
            lvt_error: MSR::new(IA32_X2APIC_LVT_ERROR),
            ticr: MSR::new(IA32_X2APIC_INIT_COUNT),
            tccr: MSR::new(IA32_X2APIC_CUR_COUNT),
            tdcr: MSR::new(IA32_X2APIC_DIV_CONF),
            self_ipi: MSR::new(IA32_X2APIC_SELF_IPI),
        }
    }

    read_write!(base);
    read!(id);
    read!(version);
    read_write!(tpr);
    read!(ppr);
    write!(eoi);
    read_write!(ldr);
    read_write!(sivr);
    read!(isr0);
    read!(isr1);
    read!(isr2);
    read!(isr3);
    read!(isr4);
    read!(isr5);
    read!(isr6);
    read!(isr7);
    read!(tmr0);
    read!(tmr1);
    read!(tmr2);
    read!(tmr3);
    read!(tmr4);
    read!(tmr5);
    read!(tmr6);
    read!(tmr7);
    read!(irr0);
    read!(irr1);
    read!(irr2);
    read!(irr3);
    read!(irr4);
    read!(irr5);
    read!(irr6);
    read!(irr7);
    read!(error);
    read_write!(icr);
    read_write!(lvt_timer);
    read_write!(lvt_thermal);
    read_write!(lvt_perf);
    read_write!(lvt_lint0);
    read_write!(lvt_lint1);
    read_write!(lvt_error);
    read_write!(ticr);
    read!(tccr);
    read_write!(tdcr);
    write!(self_ipi);
}

/// Local APIC timer modes.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerMode {
    /// Timer only fires once.
    OneShot = 0b00,
    /// Timer fires periodically.
    Periodic = 0b01,
    /// Timer fires at an absolute time.
    TscDeadline = 0b10,
}

impl Into<u64> for TimerMode{
    fn into(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerDivide {
    /// Divide by 2.
    Div2 = 0b0000,
    /// Divide by 4.
    Div4 = 0b0001,
    /// Divide by 8.
    Div8 = 0b0010,
    /// Divide by 16.
    Div16 = 0b0011,
    /// Divide by 32.
    Div32 = 0b1000,
    /// Divide by 64.
    Div64 = 0b1001,
    /// Divide by 128.
    Div128 = 0b1010,
    /// Divide by 256.
    Div256 = 0b1011,
}


impl Into<u64> for TimerDivide{
    fn into(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum IpiDestMode {
    /// Physical destination mode.
    Physical = 0,
    /// Logical destination mode.
    Logical = 1,
}

impl Into<u64> for IpiDestMode{
    fn into(self) -> u64 {
        self as u64
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum IpiDeliveryMode {
    /// Delivers to the processors specified in the vector field.
    Fixed = 0b000,
    /// Same as fixed, except interrupt is delivered to the processor with the
    /// lowest priority.
    LowestPriority = 0b001,
    /// Delivers a system management interrupt to the target processors.
    SystemManagement = 0b010,
    /// Delivers a non-maskable interrupt to the target processors.
    NonMaskable = 0b100,
    /// Delivers an INIT interrupt to the target processor(s).
    Init = 0b101,
    /// Delivers a start-up IPI to the target processor(s).
    StartUp = 0b110,
}

impl Into<u64> for IpiDeliveryMode{
    fn into(self) -> u64 {
        self as u64
    }
}

/// Specifies the destination when calling `send_ipi_all`.
#[derive(Debug)]
#[repr(u8)]
pub enum IpiAllShorthand {
    /// Send inter-processor interrupt all processors.
    AllIncludingSelf = 0b10,
    /// Send inter-processor interrupt to all processor except this one.
    AllExcludingSelf = 0b11,
}

impl Into<u64> for IpiAllShorthand{
    fn into(self) -> u64 {
        self as u64
    }
}
