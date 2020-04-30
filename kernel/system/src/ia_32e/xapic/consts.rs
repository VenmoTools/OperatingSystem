pub const LAPIC_ADDR: usize = 0xfee0_0000;

/**
FEE0 0000H Reserved
FEE0 0010H Reserved
FEE0 0020H Local APIC ID Register Read/Write.
FEE0 0030H Local APIC Version Register Read Only.
FEE0 0040H Reserved
FEE0 0050H Reserved
FEE0 0060H Reserved
FEE0 0070H Reserved
FEE0 0080H Task Priority Register (TPR) Read/Write.
FEE0 0090H Arbitration Priority Register1 (APR) Read Only.
FEE0 00A0H Processor Priority Register (PPR) Read Only.
FEE0 00B0H EOI Register Write Only.
FEE0 00C0H Remote Read Register1 (RRD) Read Only
FEE0 00D0H Logical Destination Register Read/Write.
FEE0 00E0H Destination Format Register Read/Write
FEE0 00F0H Spurious Interrupt Vector Register Read/Write (see Section 10.9.
FEE0 0100H In-Service Register (ISR); bits 31:0 Read Only.
FEE0 0110H In-Service Register (ISR); bits 63:32 Read Only.
FEE0 0120H In-Service Register (ISR); bits 95:64 Read Only.
FEE0 0130H In-Service Register (ISR); bits 127:96 Read Only.
FEE0 0140H In-Service Register (ISR); bits 159:128 Read Only.
FEE0 0150H In-Service Register (ISR); bits 191:160 Read Only.
FEE0 0160H In-Service Register (ISR); bits 223:192 Read Only.
FEE0 0170H In-Service Register (ISR); bits 255:224 Read Only.
FEE0 0180H Trigger Mode Register (TMR); bits 31:0 Read Only.
FEE0 0190H Trigger Mode Register (TMR); bits 63:32 Read Only.
FEE0 01A0H Trigger Mode Register (TMR); bits 95:64 Read Only
FEE0 01B0H Trigger Mode Register (TMR); bits 127:96 Read Only.
FEE0 01C0H Trigger Mode Register (TMR); bits 159:128 Read Only.
FEE0 01D0H Trigger Mode Register (TMR); bits 191:160 Read Only.
FEE0 01E0H Trigger Mode Register (TMR); bits 223:192 Read Only.
FEE0 01F0H Trigger Mode Register (TMR); bits 255:224 Read Only.
FEE0 0200H Interrupt Request Register (IRR); bits 31:0 Read Only.
FEE0 0210H Interrupt Request Register (IRR); bits 63:32 Read Only.
FEE0 0220H Interrupt Request Register (IRR); bits 95:64 Read Only.
FEE0 0230H Interrupt Request Register (IRR); bits 127:96 Read Only.
FEE0 0240H Interrupt Request Register (IRR); bits 159:128 Read Only.
FEE0 0250H Interrupt Request Register (IRR); bits 191:160 Read Only.
FEE0 0260H Interrupt Request Register (IRR); bits 223:192 Read Only.
FEE0 0270H Interrupt Request Register (IRR); bits 255:224 Read Only.
FEE0 0280H Error Status Register Read Only.
FEE0 0290H through
FEE0 02E0H Reserved
FEE0 02F0H LVT Corrected Machine Check Interrupt (CMCI) Register Read/Write.
FEE0 0300H Interrupt Command Register (ICR); bits 0-31 Read/Write.
FEE0 0310H Interrupt Command Register (ICR); bits 32-63 Read/Write.
FEE0 0320H LVT Timer Register Read/Write.
FEE0 0330H LVT Thermal Sensor Register2 Read/Write.
FEE0 0340H LVT Performance Monitoring Counters Register3 Read/Write.
FEE0 0350H LVT LINT0 Register Read/Write.
FEE0 0360H LVT LINT1 Register Read/Write.
FEE0 0370H LVT Error Register Read/Write.
FEE0 0380H Initial Count Register (for Timer) Read/Write.
FEE0 0390H Current Count Register (for Timer) Read Only.
FEE0 03A0H through
FEE0 03D0H Reserved
FEE0 03E0H Divide Configuration Register (for Timer) Read/Write.
FEE0 03F0H Reserved
*/

pub(crate) const CMOS_PORT: u16 = 0x70;
pub(crate) const CMOS_RETURN: u16 = 0x71;
pub(crate) const ID: u32 = 0x0020;
// ID
pub(crate) const VER: u32 = 0x0030;
// Version
pub(crate) const TPR: u32 = 0x0080;
// Task Priority
pub(crate) const EOI: u32 = 0x00B0;
// EOI
pub(crate) const SVR: u32 = 0x00F0;
// Spurious Interrupt Vector
pub(crate) const ENABLE: u32 = 0x00000100;
// Unit Enable
pub(crate) const ESR: u32 = 0x0280;
// Error Status
pub(crate) const ICRLO: u32 = 0x0300;
// Interrupt Command
pub(crate) const INIT: u32 = 0x00000500;
// INIT/RESET
pub(crate) const STARTUP: u32 = 0x00000600;
// Startup IPI
pub(crate) const DELIVS: u32 = 0x00001000;
// Delivery status
pub(crate) const ASSERT: u32 = 0x00004000;
// Assert interrupt (vs deassert)
pub(crate) const DEASSERT: u32 = 0x00000000;
pub(crate) const LEVEL: u32 = 0x00008000;
// Level triggered
pub(crate) const BCAST: u32 = 0x00080000;
// Send to all APICs, including self.
pub(crate) const BUSY: u32 = 0x00001000;
pub(crate) const FIXED: u32 = 0x00000000;
pub(crate) const ICRHI: u32 = 0x0310;
// Interrupt Command [63:32]
pub(crate) const TIMER: u32 = 0x0320;
// Local Vector Table 0 (TIMER)
pub(crate) const X1: u32 = 0x0000000B;
// divide counts by 1
pub(crate) const PERIODIC: u32 = 0x00020000;
// Periodic
pub(crate) const PCINT: u32 = 0x0340;
// Performance Counter LVT
pub(crate) const LINT0: u32 = 0x0350;
// Local Vector Table 1 (LINT0)
pub(crate) const LINT1: u32 = 0x0360;
// Local Vector Table 2 (LINT1)
pub(crate) const ERROR: u32 = 0x0370;
// Local Vector Table 3 (ERROR)
pub(crate) const MASKED: u32 = 0x00010000;
// Interrupt masked
pub(crate) const TICR: u32 = 0x0380;
// Timer Initial Count
pub(crate) const TCCR: u32 = 0x0390;
// Timer Current Count
pub(crate) const TDCR: u32 = 0x03E0;       // Timer Divide Configuration

pub(crate) const T_IRQ0: u32 = 32;
// IRQ 0 corresponds to int T_IRQ
pub(crate) const IRQ_TIMER: u32 = 0;
pub(crate) const IRQ_KBD: u32 = 1;
pub(crate) const IRQ_COM1: u32 = 4;
pub(crate) const IRQ_IDE: u32 = 14;
pub(crate) const IRQ_ERROR: u32 = 19;
pub(crate) const IRQ_SPURIOUS: u32 = 31;


