use bitflags::bitflags;
use crate::ia_32e::cpu::Port;
use crate::ia_32e::instructions::port::outb;

// 0x40 - 0x42
bitflags! {
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

struct Chips8253{

}

impl Chips8253 {
    unsafe fn init(value: u16){
        let flags = ContorlWord::COUNTER0 | ContorlWord::READ_LOW_THEN_HIGH | ContorlWord::METHOD2;
        outb(0x43, flags.bits as u16);
        let mut port = Port::new(0x40);
        port.write(value as u8);
        port.write((value >> 8 )as u8);
    }
}

