use super::{Port, UnsafePort};

const EOI: u8 = 0x20;
const ICW4: u8 = 0x01;
const ICW1: u8 = 0x11;
const ICW3_M: u8 = 0x04;
const ICW3_S: u8 = 0x02;
const MASKED: u8 = 0xff;

/// 单PIC 8259A的结构
#[derive(Debug)]
struct Pic {
    offset: u8,
    command: UnsafePort<u8>,
    data: UnsafePort<u8>,
}

impl Pic {
    /// 判断中断向量是否可接受的范围中
    /// IRQ0-IRQ7 共8个
    /// IRQ8 -IRQ15 共8个
    fn handle_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }
    /// 向对应端口写入EOI命令完成中断
    unsafe fn end_interrupt(&mut self) {
        self.command.write(EOI);
    }
}

/// PIC两级级联结构
#[derive(Debug)]
pub struct ChainedPics {
    main: Pic,
    slave: Pic,
}

impl ChainedPics {
    /// 根据指定的偏移创建ChainedPics
    pub const unsafe fn new(offset_1: u8, offset_2: u8) -> ChainedPics {
        ChainedPics {
            main: Pic {
                offset: offset_1,
                command: UnsafePort::new(0x20),
                data: UnsafePort::new(0x21),
            },
            slave: Pic {
                offset: offset_2,
                command: UnsafePort::new(0xA0),
                data: UnsafePort::new(0xA1),
            },
        }
    }

    /// 用于完成主8259A和从8259A芯片初始化操作
    /// | M/S   | ICWx | I/OPort| value|
    /// | ----- | ---- | ------ | ---- |
    /// | Main  | ICW1 | 0x20   | 0x11 |
    /// |       | ICW2 | 0x21   | 0x20 |
    /// |       | ICW3 | 0x21   | 0x04 |
    /// |       | ICW4 | 0x21   | 0x01 |
    /// | Slave | ICW1 | 0xA0   | 0x11 |
    /// |       | ICW2 | 0xA1   | 0x28 |
    /// |       | iCW3 | 0xA1   | 0x02 |
    /// |       | ICW4 | 0xA1   | 0x01 |
    pub unsafe fn initialize(&mut self) {
        // 我们需要在写入PIC之间增加延迟，尤其是在较旧的主板上。
        // 但是我们不一定有任何类型的计时器，因为它们大多数需要中断。
        // 通过将垃圾数据写入端口0x80，
        // 各种较旧版本的Linux和其他PC操作系统已通过在端口0x80上写入垃圾数据来解决此问题
        // 或者我们可以使用几个nop指令来完成
        let mut wait_port: Port<u32> = Port::new(0x80);
        // wait是一个闭包 我们只需要往0x80端口写一些数据即可
        let mut wait = || { wait_port.write(0) };

        // 写入之前保存掩码
        let saved_mask1 = self.main.data.read();
        let saved_mask2 = self.slave.data.read();
        // 主片写入ICW1
        self.main.command.write(ICW1);
        wait();
        // 从片写入ICW1
        self.slave.command.write(ICW1);
        wait();

        // 主片写入ICW2
        self.main.data.write(self.main.offset);
        wait();
        // 从片写入ICW2
        self.slave.data.write(self.slave.offset);
        wait();

        // 主片写入ICW3
        self.main.data.write(ICW3_M);
        wait();
        // 从片写入ICW3
        self.slave.data.write(ICW3_S);
        wait();

        // 主片写入ICW4
        self.main.data.write(ICW4);
        wait();
        // 从片写入ICW4
        self.slave.data.write(ICW4);
        wait();

        // 恢复掩码
        self.main.data.write(saved_mask1);
        self.slave.data.write(saved_mask2);
    }

    /// 屏蔽8259a中断控制器
    pub unsafe fn disable_8259a(&mut self) {
        self.main.data.write(MASKED);
        self.slave.data.write(MASKED);
    }

    /// 判断当前的中断号是否可以被主片或从片处理
    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.main.handle_interrupt(interrupt_id) || self.slave.handle_interrupt(interrupt_id)
    }
    /// 8259A采用非自动中断结束方式，
    /// 那么CPU必须在中断处理程序结尾向8259A芯片发送EOI(End Of Interrupt,结束中断)命令
    /// 来复位ISR对应位，如果中断请求来自级联8259A芯片，则必须向两个芯片都发送EOI命令
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.slave.handle_interrupt(interrupt_id) {
                self.slave.end_interrupt();
            }
            self.main.end_interrupt();
        }
    }
}