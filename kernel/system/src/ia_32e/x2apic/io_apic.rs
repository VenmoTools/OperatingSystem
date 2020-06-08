/// this code base on https://github.com/kwzhao/x2apic-rs

use crate::ia_32e::x2apic::ioapic_register::IoApicRegisters;
use crate::bits::flags::IrqFlags;
use crate::ia_32e::x2apic::consts::{IRQ_MODE_MASK, IRQ_MASK_BIT, IOAPIC_TABLE_BASE, IOAPIC_ID, IOAPIC_VERSION, IOAPIC_ARBITRATION};


/// IOAPIC interrupt modes.
#[derive(Debug)]
#[repr(u32)]
pub enum IrqMode {
    /// Asserts the INTR signal on all allowed processors.
    Fixed = 0x0000_0000,
    /// Asserts the INTR signal on the lowest priority processor allowed.
    LowestPriority = 0x0000_0100,
    /// System management interrupt.
    /// Requires edge-triggering.
    SystemManagement = 0x0000_0200,
    /// Asserts the NMI signal on all allowed processors.
    /// Requires edge-triggering.
    NonMaskable = 0x0000_0400,
    /// Asserts the INIT signal on all allowed processors.
    /// Requires edge-triggering.
    Init = 0x0000_0500,
    /// Asserts the INTR signal as a signal that originated in an
    /// externally-connected interrupt controller.
    /// Requires edge-triggering.
    External = 0x0000_0700,
}

// Gets the lower segment selector for `irq`
pub fn lo(irq: u8) -> u32 {
    IOAPIC_TABLE_BASE + (2 * u32::from(irq))
}

// Gets the upper segment selector for `irq`
pub fn hi(irq: u8) -> u32 {
    lo(irq) + 1
}


impl IrqMode {
    pub(super) fn as_u32(self) -> u32 {
        self as u32
    }
}

/// The IOAPIC structure.
#[derive(Debug)]
pub struct IoApic {
    regs: IoApicRegisters,
}

impl IoApic {
    /// Returns an IOAPIC with the given MMIO address `base_addr`.
    ///
    /// **The given MMIO address must already be mapped.**
    pub unsafe fn new(base_addr: u64) -> Self {
        IoApic {
            regs: IoApicRegisters::new(base_addr),
        }
    }

    /// Initialize the IOAPIC's redirection table entries with the given
    /// interrupt offset.
    ///
    /// Each entry `i` is redirected to `i + offset`.
    pub unsafe fn init(&mut self, offset: u8) {
        let end = self.max_table_entry() + 1;

        for i in 0..end {
            self.regs.set(lo(i), u32::from(i + offset));
            self.regs.write(hi(i), 0);
        }
    }

    /// Returns the IOAPIC IOAPIC_ID.
    pub unsafe fn id(&mut self) -> u8 {
        ((self.regs.read(IOAPIC_ID) >> 24) & 0xf) as u8
    }

    /// Sets the IOAPIC IOAPIC_ID to `id`.
    pub unsafe fn set_id(&mut self, id: u8) {
        self.regs.write(IOAPIC_ID, u32::from(id) << 24);
    }

    /// Returns the IOAPIC version.
    pub unsafe fn version(&mut self) -> u8 {
        (self.regs.read(IOAPIC_VERSION) & 0xff) as u8
    }

    /// Returns the entry number (starting at zero) of the highest entry in the
    /// redirection table.
    pub unsafe fn max_table_entry(&mut self) -> u8 {
        ((self.regs.read(IOAPIC_VERSION) >> 16) & 0xff) as u8
    }

    /// Returns the IOAPIC arbitration IOAPIC_ID.
    pub unsafe fn arbitration_id(&mut self) -> u8 {
        ((self.regs.read(IOAPIC_ARBITRATION) >> 24) & 0xf) as u8
    }

    /// Sets the IOAPIC arbitration IOAPIC_ID to `id`.
    pub unsafe fn set_arbitration_id(&mut self, id: u8) {
        self.regs.write(IOAPIC_ARBITRATION, u32::from(id) << 24);
    }

    /// Enable interrupt number `irq` on the CPUs specified by `dest`.
    pub unsafe fn enable_irq(&mut self, irq: u8, dest: u32, mode: IrqMode, options: IrqFlags, ) {
        let lo = lo(irq);
        let hi = hi(irq);

        self.regs.set(hi, dest << 24);

        self.regs.clear(lo, IRQ_MODE_MASK | IrqFlags::all().bits());
        self.regs.set(lo, mode.as_u32() | options.bits());

        self.regs.clear(lo, IRQ_MASK_BIT);
    }

    /// Disable interrupt number `irq`.
    pub unsafe fn disable_irq(&mut self, irq: u8) {
        self.regs.set(lo(irq), IRQ_MASK_BIT);
    }
}
