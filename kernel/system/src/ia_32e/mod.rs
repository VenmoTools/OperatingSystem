pub use addr::{align_down, align_up, NoCanonicalAddr, NoInvalidPhysAddr, PhysAddr, VirtAddr};
use core::fmt;

use crate::alloc::string::String;
use crate::ia_32e::x2apic::register::{IpiDestMode, TimerDivide, TimerMode};

///! ia-32模式下的描述符操作，分页操作，PIC以及常用的指令
mod addr;
pub mod descriptor;
pub mod instructions;
pub mod paging;
pub mod cpu;
pub mod x2apic;
pub mod acpi;
pub mod call_convention;
pub mod xapic;
pub mod controller;

/// 系统特权级
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegedLevel {
    /// 特权级0
    Ring0 = 0,
    /// 特权级1
    Ring1 = 1,
    /// 特权级2
    Ring2 = 2,
    /// 特权级3
    Ring3 = 3,
}

impl PrivilegedLevel {
    /// 根据给定额值判断当前特权级，如果不在范围则Panic
    pub fn from_u16(level: u16) -> PrivilegedLevel {
        match level {
            0 => PrivilegedLevel::Ring0,
            1 => PrivilegedLevel::Ring1,
            2 => PrivilegedLevel::Ring2,
            3 => PrivilegedLevel::Ring3,
            other => panic!("invalid privileged level `{}`", other),
        }
    }
}

/// 用于将u64以十六进制显示
#[repr(transparent)]
struct Hex(u64);

impl Hex {
    pub fn new(d: u64) -> Hex {
        Hex(d)
    }
}

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

/// 用于将u64以二进制显示
#[repr(transparent)]
struct Binary(u64);

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

#[doc(hidden)]
macro_rules! set_attr {

    ($name:ident,$attr:ident,$args:ty) => {
        pub fn $name(&mut self, arg: $args) -> &mut Self {
            self.$attr = Some(arg);
            self
        }
    };
}

#[derive(Copy, Clone, Default)]
pub struct ApicInfo {
    // for x2apic
    pub timer_vector: Option<usize>,
    pub error_vector: Option<usize>,
    pub spurious_vector: Option<usize>,
    pub timer_mode: Option<TimerMode>,
    pub timer_divide: Option<TimerDivide>,
    pub timer_initial: Option<u32>,
    pub ipi_destination_mode: Option<IpiDestMode>,
    // for io apic
    pub ioapic_offset: Option<u8>,
    // for 8259
}

impl ApicInfo {
    /// Returns a new local APIC builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets io apic offset
    /// This field is required.
    set_attr!(set_io_apic_offset,ioapic_offset,u8);

    /// Sets the interrupt index of timer interrupts.
    ///
    /// This field is required.
    set_attr!(set_timer_vector,timer_vector,usize);


    /// Sets the interrupt index for internal APIC errors.
    ///
    /// This field is required.
    set_attr!(set_error_vector,error_vector,usize);


    /// Sets the interrupt index for spurious interrupts.
    ///
    /// This field is required.
    set_attr!(set_spurious_vector,spurious_vector,usize);


    /// Sets the timer mode.
    ///
    /// Default: Periodic.
    set_attr!(set_timer_mode,timer_mode,TimerMode);


    /// Sets the timer divide configuration.
    ///
    /// Default: Div256.
    set_attr!(set_timer_divide,timer_divide,TimerDivide);


    /// Sets the timer initial count.
    ///
    /// Default: 10_000_000.
    set_attr!(set_timer_initial,timer_initial,u32);


    /// Sets the IPI destination mode.
    ///
    /// Default: Physical.
    set_attr!(set_ipi_destination_mode,ipi_destination_mode,IpiDestMode);


    /// Builds a new `LocalApic`.
    ///
    /// # Errors
    ///
    /// This function returns an error if any of the required fields are empty.
    pub fn build(self) -> Result<Self, String> {
        if cfg!(x2apic)
            && self.timer_vector.is_none()
            || self.error_vector.is_none()
            || self.spurious_vector.is_none() {
            return Err(String::from("x2apic: required field(s) empty"));
        }
        if cfg!(xapic) && self.ioapic_offset.is_none() {
            return Err(String::from("xapic: required field(s) empty"));
        }
        Ok(self)
    }
}

