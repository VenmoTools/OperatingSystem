use core::fmt;
use core::ptr::read_volatile;
use core::ptr::write_volatile;

use crate::bits::BitOpt;
use crate::ia_32e::instructions::port::outw;
use crate::ia_32e::xapic::consts::{ASSERT, BCAST, CMOS_PORT, CMOS_RETURN, DELIVS, ENABLE, EOI, ERROR, ESR, ICRHI, ICRLO, ID, INIT, IRQ_ERROR, IRQ_SPURIOUS, IRQ_TIMER, LEVEL, LINT0, LINT1, MASKED, PCINT, PERIODIC, STARTUP, T_IRQ0, TDCR, TICR, TIMER, TPR, VER, X1};

use super::consts::SVR;

#[allow(non_camel_case_types)]
pub struct xApic {
    base: usize
}

impl xApic {

    pub fn new(base: usize) -> Self {
        Self { base }
    }

    pub fn cpu_init(&mut self) {
        unsafe {
            // Enable local APIC; set spurious interrupt vector.
            self.write(SVR, ENABLE | (T_IRQ0 + IRQ_SPURIOUS));

            // The timer repeatedly counts down at bus frequency
            // from lapic[TICR] and then issues an interrupt.
            // If xv6 cared more about precise timekeeping,
            // TICR would be calibrated using an external time source.
            self.write(TDCR, X1);
            self.write(TIMER, PERIODIC | (T_IRQ0 + IRQ_TIMER));
            self.write(TICR, 10000000);

            // Disable logical interrupt lines.
            self.write(LINT0, MASKED);
            self.write(LINT1, MASKED);

            // Disable performance counter overflow interrupts
            // on machines that provide that interrupt entry.
            if (self.read(VER) >> 16 & 0xFF) >= 4 {
                self.write(PCINT, MASKED);
            }

            // Map error interrupt to IRQ_ERROR.
            self.write(ERROR, T_IRQ0 + IRQ_ERROR);

            // Clear error status register (requires back-to-back writes).
            self.write(ESR, 0);
            self.write(ESR, 0);

            // Ack any outstanding interrupts.
            self.write(EOI, 0);

            // Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(ICRHI, 0);
            self.write(ICRLO, BCAST | INIT | LEVEL);
            while self.read(ICRLO) & DELIVS != 0 {}

            // Enable interrupts on the APIC (but not on the processor).
            self.write(TPR, 0);
        }

    }
    pub fn id(&self) -> u32 {
        unsafe { self.read(ID) >> 24 }
    }
    pub fn version(&self) -> u32 {
        unsafe { self.read(VER) }
    }
    pub fn icr(&self) -> u64 {
        unsafe { (self.read(ICRHI) as u64) << 32 | self.read(ICRLO) as u64 }
    }
    pub fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(ICRLO).get_bit(12) {}
            self.write(ICRHI, (value >> 32) as u32);
            self.write(ICRLO, value as u32);
            while self.read(ICRLO).get_bit(12) {}
        }
    }
    pub fn eoi(&mut self) {
        unsafe { self.write(EOI, 0); }
    }

    /// The entry point `addr` must be 4K aligned.
    /// This function will access memory: 0x467
    pub unsafe fn start_ap(&mut self, apic_id: u8, addr: u32) {
        assert_eq!(addr & 0xfff, 0, "The entry point address must be 4K aligned");

        // "The BSP must initialize CMOS shutdown code to 0AH
        // and the warm reset vector (DWORD based at 40:67) to point at
        // the AP startup code prior to the [universal startup algorithm]."
        outw(CMOS_PORT, 0xf);   // offset 0xF is shutdown code
        outw(CMOS_RETURN, 0xa);

        let wrv = (0x40 << 4 | 0x67) as *mut u16;  // Warm reset vector
        *wrv = 0;
        *wrv.add(1) = addr as u16 >> 4;

        // "Universal startup algorithm."
        // Send INIT (level-triggered) interrupt to reset other CPU.
        self.write(ICRHI, (apic_id as u32) << 24);
        self.write(ICRLO, INIT | LEVEL | ASSERT);
        microdelay(200);
        self.write(ICRLO, INIT | LEVEL);
        microdelay(10000);

        // Send startup IPI (twice!) to enter code.
        // Regular hardware is supposed to only accept a STARTUP
        // when it is in the halted state due to an INIT.  So the second
        // should be ignored, but it is part of the official Intel algorithm.
        for _ in 0..2 {
            self.write(ICRHI, (apic_id as u32) << 24);
            self.write(ICRLO, STARTUP | (addr >> 12) as u32);
            microdelay(200);
        }
    }
}

macro_rules! rdtsc {
    () => {
        {
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::__rdtscp;
            let mut _aux = 0;
            unsafe{
                __rdtscp(&mut _aux) as u64
            }
        }
    };
}

fn microdelay(us: u64) {
    let start = rdtsc!();
    let freq = 3_000_000_000u64; // 3GHz
    let end = start + freq / 1_000_000 * us;
    while rdtsc!() < end {}
}

impl fmt::Debug for xApic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
impl xApic {
    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.base + reg as usize) as *const u32)
    }
    unsafe fn write(&self, reg: u32, value: u32) {
        write_volatile((self.base + reg as usize) as *mut u32, value);
        let _ = self.read(0x20);
    }
}

