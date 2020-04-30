/// this code base on https://github.com/kwzhao/x2apic-rs

use core::ops::Range;

pub const BASE_APIC_ENABLE: usize = 11;
pub const BASE_X2APIC_ENABLE: usize = 10;
pub const BASE_BSP: usize = 8;

pub const VERSION_NR: Range<usize> = 0..8;
pub const VERSION_MAX_LVT_ENTRY: Range<usize> = 16..24;
pub const VERSION_EOI_BCAST_SUPPRESSION: usize = 24;

pub const SIVR_EOI_BCAST_SUPPRESSION: usize = 12;
pub const SIVR_FOCUS_PROCESSOR_CHECKING: usize = 9;
pub const SIVR_APIC_SOFTWARE_ENABLE: usize = 8;
pub const SIVR_VECTOR: Range<usize> = 0..8;

pub const ICR_DESTINATION: Range<usize> = 32..64;
pub const ICR_DEST_SHORTHAND: Range<usize> = 18..20;
pub const ICR_TRIGGER_MODE: usize = 15;
pub const ICR_LEVEL: usize = 14;
pub const ICR_DESTINATION_MODE: usize = 11;
pub const ICR_DELIVERY_MODE: Range<usize> = 8..11;
pub const ICR_VECTOR: Range<usize> = 0..8;

pub const LVT_TIMER_MODE: Range<usize> = 17..19;
pub const LVT_TIMER_MASK: usize = 16;
pub const LVT_TIMER_VECTOR: Range<usize> = 0..8;

pub const LVT_ERROR_VECTOR: Range<usize> = 0..8;

pub const TDCR_DIVIDE_VALUE: Range<usize> = 0..4;

pub const IRQ_MASK_BIT: u32 = 0x0001_0000;
pub const IRQ_MODE_MASK: u32 = 0x0000_0700;


// Register selectors
pub const IOAPIC_ID: u32 = 0x00;
pub const IOAPIC_VERSION: u32 = 0x01;
pub const IOAPIC_ARBITRATION: u32 = 0x02;
pub const IOAPIC_TABLE_BASE: u32 = 0x10;