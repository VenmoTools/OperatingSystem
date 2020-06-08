use bitflags::bitflags;
use crate::ia_32e::cpu::Port;
use crate::ia_32e::instructions::port::{outb};

pub const TIMER_FREQUENCY: usize = 1193182;
pub const TIMER_MODE: u16 = 0x43;
pub const TIMER0: u16 = 0x40;
pub const TIMER1: u16 = 0x41;
pub const TIMER2: u16 = 0x42;
pub const TIMER3: u16 = 0x43;
pub const TIMER4: u16 = 0x44;
pub const TIMER5: u16 = 0x45;

pub const IRQ_FREQUENCY: usize = 100;

pub const fn timer_count(frequency: usize) -> usize {
    TIMER_FREQUENCY / frequency
}

// 0x40 - 0x42
bitflags! {
    /// 110110
    pub struct ContorlWord: u8{
        /// BCD 码
        const BCD = 1 << 0;
        const METHOD0 = 000 << 1;
        const METHOD1 = 001 << 1;
        const METHOD2 = 010 << 1;
        const METHOD3 = 011 << 1;
        const METHOD4 = 100 << 1;
        const METHOD5 = 101 << 1;
        /// 锁存数据仅供CPU读
        const LOCK_DATA = 00 << 4;
        /// 只读写低字节
        const LOW_READ_ONLY = 01 << 4;
        /// 只读写高字节
        const HIGH_READ_ONLY = 10 << 4;
        /// 先读写低字节后读写高字节
        const READ_LOW_THEN_HIGH = 11 << 4;
        /// 选择计数器
        const COUNTER0 = 00 << 6;
        /// 选择计数器 1
        const COUNTER1 = 01 << 6;
        /// 选择计数器 2
        const COUNTER2 = 10 << 6;
    }
}

struct Chips8253 {
    // frequency (Hz)
    frequency: u16,
    control: ContorlWord,
}

impl Chips8253 {
    pub fn new(frequency: u16, control: ContorlWord) -> Self {
        Self {
            frequency,
            control,
        }
    }

    // 中断计数方式
    pub unsafe fn interrupt_on_terminal_count(&self) {
        self.init_time_n(TIMER0)
    }
    // 硬件可重触发单稳方式
    pub unsafe fn hardware_retriggerable_one_shot(&self) {
        self.init_time_n(TIMER1)
    }
    // 比率发生器
    pub unsafe fn rate_generator(&self) {
        self.init_time_n(TIMER2)
    }
    // 方波发生器
    pub unsafe fn square_wave_generator(&self) {
        self.init_time_n(TIMER3)
    }
    // 软件触发选通
    pub unsafe fn software_triggered_strobe(&self) {
        self.init_time_n(TIMER4)
    }
    // 硬件触发选通
    pub unsafe fn hardware_triggered_strobe(&self) {
        self.init_time_n(TIMER5)
    }

    pub unsafe fn init_time_n(&self, n: u16) {
        debug_assert!(n >= 0 && n < 6, "8253 support timer0-timer5");
        outb(0x43, self.control.bits as u16);
        let mut port = Port::new(n);
        port.write(self.frequency as u8);
        port.write((self.frequency >> 8) as u8);
    }

    pub unsafe fn stop_time_n(&self, n: u16) {
        debug_assert!(n >= 0 && n < 6, "8253 support timer0-timer5");
        outb(self.control.bits, TIMER_MODE);
        let mut timer0 = Port::new(n);
        timer0.write(0_u8);
        timer0.write(0_u8);
    }

    pub unsafe fn read_pit_count(&self, n: u16) -> u16 {
        debug_assert!(n >= 0 && n < 6, "8253 support timer0-timer5");
        outb(0, TIMER_MODE);
        let mut port = Port::new(n);
        let low: u8 = port.read();
        let high = port.read();
        ((high as u16) << 8) | low as u16
    }

    pub unsafe fn set_pit_count(&self, count: u16) {
        let mut port = Port::new(TIMER0);
        port.write(count as u8);
        port.write((count >> 8) as u8);
    }
}

