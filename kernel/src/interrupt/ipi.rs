use system::ia_32e::instructions::page_table::flush_all;
use system::interrupt;

use crate::process::scheduler::switch;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum IpiKind {
    WakeUp = 0x40,
    Tlb = 0x41,
    Switch = 0x42,
    Pit = 0x43,
}

impl From<IpiKind> for usize {
    fn from(kind: IpiKind) -> Self {
        match kind {
            IpiKind::WakeUp => 0x40,
            IpiKind::Tlb => 0x41,
            IpiKind::Switch => 0x42,
            IpiKind::Pit => 0x43,
        }
    }
}


interrupt!(ipi_wakeup,{
});

interrupt!(ipi_switch,{
    switch();
});

interrupt!(ipi_pit,{
});

interrupt!(ipi_tlb,{
    flush_all();
});