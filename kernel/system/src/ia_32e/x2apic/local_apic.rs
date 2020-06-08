/// this code base on https://github.com/kwzhao/x2apic-rs

use crate::ia_32e::x2apic::register::{TimerMode, TimerDivide, IpiDestMode, LocalApicRegisters, IpiDeliveryMode, IpiAllShorthand};
use crate::ia_32e::x2apic::consts::{ICR_DEST_SHORTHAND, ICR_DESTINATION, TDCR_DIVIDE_VALUE, LVT_TIMER_MODE, LVT_TIMER_MASK, VERSION_MAX_LVT_ENTRY, VERSION_EOI_BCAST_SUPPRESSION, VERSION_NR, BASE_APIC_ENABLE, ICR_VECTOR, ICR_DELIVERY_MODE, ICR_DESTINATION_MODE, ICR_LEVEL, BASE_X2APIC_ENABLE, SIVR_APIC_SOFTWARE_ENABLE, LVT_TIMER_VECTOR, LVT_ERROR_VECTOR, SIVR_VECTOR, BASE_BSP};
use crate::bits::flags::LocalAPICFlags;
use crate::bits::BitOpt;
use crate::ia_32e::ApicInfo;

/// The local APIC structure.
#[derive(Debug)]
pub struct LocalApic {
    timer_vector: usize,
    error_vector: usize,
    spurious_vector: usize,
    timer_mode: TimerMode,
    timer_divide: TimerDivide,
    timer_initial: u32,
    ipi_destination_mode: IpiDestMode,
    regs: LocalApicRegisters,
}

impl From<ApicInfo> for LocalApic{
    fn from(info: ApicInfo) -> Self {
        LocalApic {
            timer_vector: info.timer_vector.expect("missing timer vector"),
            error_vector: info.error_vector.expect("missing error vector"),
            spurious_vector: info.spurious_vector.expect("missing spurious vector"),
            timer_mode: info.timer_mode.unwrap_or(TimerMode::Periodic),
            timer_divide: info.timer_divide.unwrap_or(TimerDivide::Div256),
            timer_initial: info.timer_initial.unwrap_or(10_000_000),
            ipi_destination_mode: info
                .ipi_destination_mode
                .unwrap_or(IpiDestMode::Physical),
            regs: LocalApicRegisters::new(),
        }
    }
}

impl LocalApic {
    /// Enables the local APIC.
    ///
    /// Turns on the APIC timer and disables the `LINT0` and `LINT1` local
    /// interrupts.
    pub unsafe fn enable(&mut self) {
        self.x2apic_mode_enable();

        self.remap_lvt_entries();

        self.configure_timer();
        self.enable_timer();

        self.disable_local_interrupt_pins();

        self.software_enable();
    }

    /// Disables the APIC
    pub unsafe fn disable(&mut self) {
        self.regs.set_base_bit(BASE_APIC_ENABLE, false);
    }

    /// Signals end-of-interrupt to the local APIC.
    pub unsafe fn end_of_interrupt(&mut self) {
        self.regs.write_eoi(0);
    }

    /// Is this processor the BSP?
    pub unsafe fn is_bsp(&self) -> bool {
        self.regs.base_bit(BASE_BSP)
    }

    /// Returns the local APIC ID.
    pub unsafe fn id(&self) -> u32 {
        self.regs.id() as u32
    }

    /// Returns the version number of the local APIC.
    pub unsafe fn version(&self) -> u8 {
        self.regs.version_bit_range(VERSION_NR) as u8
    }

    /// Returns the maximum local vector table entry.
    pub unsafe fn max_lvt_entry(&self) -> u8 {
        self.regs.version_bit_range(VERSION_MAX_LVT_ENTRY) as u8
    }

    /// Does this processor support EOI-broadcast suppression?
    pub unsafe fn has_eoi_bcast_suppression(&self) -> bool {
        self.regs.version_bit(VERSION_EOI_BCAST_SUPPRESSION)
    }

    /// Returns error flags from the error status register.
    pub unsafe fn error_flags(&self) -> LocalAPICFlags {
        LocalAPICFlags::from_bits_truncate(self.regs.error() as u8)
    }

    /// Enable the APIC timer.
    pub unsafe fn enable_timer(&mut self) {
        self.regs.set_lvt_timer_bit(LVT_TIMER_MASK, false);
    }

    /// Disable the APIC timer.
    pub unsafe fn disable_timer(&mut self) {
        self.regs.set_lvt_timer_bit(LVT_TIMER_MASK, true);
    }

    /// Sets the timer mode.
    pub unsafe fn set_timer_mode(&mut self, mode: TimerMode) {
        self.timer_mode = mode;
        self.regs.set_lvt_timer_bit_range(LVT_TIMER_MODE, mode.into());
    }

    /// Sets the timer divide configuration.
    pub unsafe fn set_timer_divide(&mut self, divide: TimerDivide) {
        self.timer_divide = divide;
        self.regs
            .set_tdcr_bit_range(TDCR_DIVIDE_VALUE, divide.into());
    }

    /// Sets the timer initial count.
    pub unsafe fn set_timer_initial(&mut self, initial: u32) {
        self.timer_initial = initial;
        self.regs.write_ticr(u64::from(initial));
    }

    /// Sets the logical x2APIC ID.
    ///
    /// This is used when the APIC is in logical mode.
    pub unsafe fn set_logical_id(&mut self, dest: u32) {
        self.regs.write_ldr(u64::from(dest));
    }

    /// Sends an IPI to the processor(s) in `dest`.
    pub unsafe fn send_ipi(&mut self, vector: u8, dest: u32) {
        let mut icr_val = self.format_icr(vector, IpiDeliveryMode::Fixed);

        icr_val.set_bits(ICR_DESTINATION, u64::from(dest));
        self.regs.write_icr(icr_val);
    }

    /// Sends an IPI to every processor, either including or excluding the
    /// current one.
    pub unsafe fn send_ipi_all(&mut self, vector: u8, who: IpiAllShorthand) {
        let mut icr_val = self.format_icr(vector, IpiDeliveryMode::Fixed);

        icr_val.set_bits(ICR_DEST_SHORTHAND, who.into());
        self.regs.write_icr(icr_val);
    }

    /// Send a lowest-priority IPI to the processor(s) in `dest`.
    pub unsafe fn send_lowest_priority_ipi(&mut self, vector: u8, dest: u32) {
        let mut icr_val =
            self.format_icr(vector, IpiDeliveryMode::LowestPriority);

        icr_val.set_bits(ICR_DESTINATION, u64::from(dest));
        self.regs.write_icr(icr_val);
    }

    /// Send a lowest-priority IPI to all processors, either including or
    /// excluding the current one.
    pub unsafe fn send_lowest_priority_ipi_all(
        &mut self,
        vector: u8,
        who: IpiAllShorthand,
    ) {
        let mut icr_val =
            self.format_icr(vector, IpiDeliveryMode::LowestPriority);

        icr_val.set_bits(ICR_DEST_SHORTHAND, who.into());
        self.regs.write_icr(icr_val);
    }

    /// Sends a system management IPI to `dest`.
    pub unsafe fn send_smi(&mut self, dest: u32) {
        let mut icr_val = self.format_icr(0, IpiDeliveryMode::SystemManagement);

        icr_val.set_bits(ICR_DESTINATION, u64::from(dest));
        self.regs.write_icr(icr_val);
    }

    /// Sends a system management IPI to all processors, either including or
    /// excluding the current one.
    pub unsafe fn send_smi_all(&mut self, who: IpiAllShorthand) {
        let mut icr_val = self.format_icr(0, IpiDeliveryMode::SystemManagement);

        icr_val.set_bits(ICR_DEST_SHORTHAND, who.into());
        self.regs.write_icr(icr_val);
    }

    /// Sends a non-maskable interrupt to the processor(s) in `dest`.
    pub unsafe fn send_nmi(&mut self, dest: u32) {
        let mut icr_val = self.format_icr(0, IpiDeliveryMode::NonMaskable);

        icr_val.set_bits(ICR_DESTINATION, u64::from(dest));
        self.regs.write_icr(icr_val);
    }

    /// Sends a non-maskable interrupt to all processors, either including or
    /// excluding the current one.
    pub unsafe fn send_nmi_all(&mut self, who: IpiAllShorthand) {
        let mut icr_val = self.format_icr(0, IpiDeliveryMode::NonMaskable);

        icr_val.set_bits(ICR_DEST_SHORTHAND, who.into());
        self.regs.write_icr(icr_val);
    }

    /// Sends a start-up IPI to the processors in `dest`.
    pub unsafe fn send_sipi(&mut self, vector: u8, dest: u32) {
        let mut icr_val = self.format_icr(vector, IpiDeliveryMode::StartUp);

        icr_val.set_bits(ICR_DESTINATION, u64::from(dest));
        self.regs.write_icr(icr_val);
    }

    /// Sends a start-up IPI to all other processors.
    pub unsafe fn send_sipi_all(&mut self, vector: u8) {
        let mut icr_val = self.format_icr(vector, IpiDeliveryMode::StartUp);

        icr_val.set_bits(
            ICR_DEST_SHORTHAND,
            IpiAllShorthand::AllExcludingSelf.into(),
        );
        self.regs.write_icr(icr_val);
    }

    /// Issues an IPI to itself on vector `irq`.
    pub unsafe fn send_ipi_self(&mut self, vector: u8) {
        self.regs.write_self_ipi(u64::from(vector));
    }

    fn format_icr(&self, vector: u8, mode: IpiDeliveryMode) -> u64 {
        let mut icr_val = 0;

        icr_val.set_bits(ICR_VECTOR, u64::from(vector));
        icr_val.set_bits(ICR_DELIVERY_MODE, mode.into());
        icr_val.set_bit(
            ICR_DESTINATION_MODE,
            self.ipi_destination_mode == IpiDestMode::Logical,
        );
        icr_val.set_bit(ICR_LEVEL, true);
        icr_val
    }

    // Misc configuration

    unsafe fn x2apic_mode_enable(&mut self) {
        self.regs.set_base_bit(BASE_X2APIC_ENABLE, true);
    }

    unsafe fn software_enable(&mut self) {
        self.regs.set_sivr_bit(SIVR_APIC_SOFTWARE_ENABLE, true);
    }

    unsafe fn remap_lvt_entries(&mut self) {
        self.regs.set_lvt_timer_bit_range(
            LVT_TIMER_VECTOR,
            self.timer_vector as u64,
        );
        self.regs.set_lvt_error_bit_range(
            LVT_ERROR_VECTOR,
            self.error_vector as u64,
        );
        self.regs
            .set_sivr_bit_range(SIVR_VECTOR, self.spurious_vector as u64);
    }

    unsafe fn configure_timer(&mut self) {
        self.regs
            .set_lvt_timer_bit_range(LVT_TIMER_MODE, self.timer_mode.into());
        self.regs
            .set_tdcr_bit_range(TDCR_DIVIDE_VALUE, self.timer_divide.into());
        self.regs.write_ticr(u64::from(self.timer_initial));
    }

    unsafe fn disable_local_interrupt_pins(&mut self) {
        self.regs.write_lvt_lint0(0);
        self.regs.write_lvt_lint1(0);
    }
}