use crate::ia_32e::cpu::port::{UnsafePort, Port};


const CMD_END_OF_INTERRUPT: u8 = 0x20;

#[derive(Debug)]
struct Pic {
    offset: u8,
    command: UnsafePort<u8>,
    data: UnsafePort<u8>,
}

impl Pic {
    fn handle_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }
    unsafe fn end_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }
}

#[derive(Debug)]
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset_1: u8, offset_2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset_1,
                    command: UnsafePort::new(0x20),
                    data: UnsafePort::new(0x21),
                },
                Pic {
                    offset: offset_2,
                    command: UnsafePort::new(0xA0),
                    data: UnsafePort::new(0xA1),
                }
            ]
        }
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handle_interrupt(interrupt_id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handle_interrupt(interrupt_id) {
                self.pics[1].end_interrupt();
            }
            self.pics[0].end_interrupt();
        }
    }
}