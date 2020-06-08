use bitflags::_core::sync::atomic::AtomicUsize;
use spin::Mutex;
use system::ia_32e::ApicInfo;
#[cfg(feature = "pic")]
use system::ia_32e::controller::PIC;
use system::ia_32e::controller::ProgrammableController;
#[cfg(feature = "xapic")]
use system::ia_32e::controller::X2APIC;
#[cfg(feature = "x2apic")]
use system::ia_32e::controller::XPAIC;
use system::ia_32e::cpu::ChainedPics;
use system::ia_32e::descriptor::InterruptDescriptorTable;
#[cfg(feature = "x2apic")]
use system::ia_32e::x2apic::local_apic::LocalApic;
#[cfg(feature = "xapic")]
use system::ia_32e::xapic::consts::LAPIC_ADDR;
#[cfg(feature = "xapic")]
use system::ia_32e::xapic::xApic;

use lazy_static::lazy_static;

use crate::interrupt::{exceptions, irq};
use crate::println;

pub const PIC_MAIN: u8 = 32;
pub const PIC_SLAVE: u8 = PIC_MAIN + 8;
pub const TICKS: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum InterruptIndex {
    // 时钟中断
    Timer = PIC_MAIN,
    // 键盘中断
    KeyBoard,
    Cascade,
    Com2,
    Com1,
    Lpt2,
    Floppy,
    Lpt1,
    Rtc,
    Pci1,
    Pci2,
    Pci3,
    Mouse,
    Fpu,
    Ata1,
    Ata2,
}

impl From<usize> for InterruptIndex {
    fn from(index: usize) -> Self {
        Self::from(index as u8)
    }
}

impl From<InterruptIndex> for usize {
    fn from(index: InterruptIndex) -> Self {
        Self::from(index as usize)
    }
}

impl From<InterruptIndex> for u8 {
    fn from(index: InterruptIndex) -> Self {
        Self::from(index as u8)
    }
}

impl From<u8> for InterruptIndex {
    fn from(index: u8) -> Self {
        match index {
            32 => InterruptIndex::Timer,
            33 => InterruptIndex::KeyBoard,
            34 => InterruptIndex::Cascade,
            35 => InterruptIndex::Com2,
            36 => InterruptIndex::Com1,
            37 => InterruptIndex::Lpt2,
            38 => InterruptIndex::Floppy,
            39 => InterruptIndex::Lpt1,
            40 => InterruptIndex::Rtc,
            41 => InterruptIndex::Pci1,
            42 => InterruptIndex::Pci2,
            43 => InterruptIndex::Pci3,
            44 => InterruptIndex::Mouse,
            45 => InterruptIndex::Fpu,
            46 => InterruptIndex::Ata1,
            47 => InterruptIndex::Ata2,
            _ => {
                panic!("the index is invalid!")
            }
        }
    }
}


#[cfg(feature = "pic")]
pub static CONTROLLER: Mutex<ProgrammableController<PIC>> = Mutex::new(ProgrammableController::empty());
#[cfg(feature = "xapic")]
pub static CONTROLLER: Mutex<ProgrammableController<XPAIC>> = Mutex::new(ProgrammableController::empty());
#[cfg(feature = "x2apic")]
pub static CONTROLLER: Mutex<ProgrammableController<X2APIC>> = Mutex::new(ProgrammableController::empty());

lazy_static! {
    static ref IDT: InterruptDescriptorTable = init();
}

#[cfg(feature = "pic")]
pub fn disable_8259a() {
    unsafe {
        CONTROLLER.lock().disable();
    }
}

#[cfg(feature = "pic")]
pub fn init_apic(info: ApicInfo) {
    let mut lock = CONTROLLER.lock();
    lock.set_pic(unsafe { ChainedPics::new(PIC_MAIN, PIC_SLAVE) });
    unsafe {
        lock.init(info);
    }
    println!("pic init done!");
}

#[cfg(feature = "xapic")]
pub fn init_apic(info: ApicInfo) {
    let mut lock = CONTROLLER.lock();
    lock.set_xapic(xApic::new(LAPIC_ADDR));
    unsafe {
        lock.init(info)
    }
    println!("xapic init done!");
}

#[cfg(feature = "x2apic")]
pub fn init_apic(info: ApicInfo) {
    let mut lock = CONTROLLER.lock();
    unsafe {
        lock.init(info)
    }
    println!("x2apic init done!");
}

pub fn init() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_by_zero.set_handler_fn(exceptions::divide_by_zero);
    idt.breakpoint.set_handler_fn(exceptions::breakpoint);
    idt.page_fault.set_handler_fn(exceptions::page_fault);
    idt.divide_by_zero.set_handler_fn(exceptions::divide_by_zero);
    idt.invalid_tss.set_handler_fn(exceptions::invalid_tss);
    idt.security_exception.set_handler_fn(exceptions::security_exception);
    idt.segment_not_present.set_handler_fn(exceptions::segment_not_present);
    idt.alignment_check.set_handler_fn(exceptions::alignment_check);
    idt.bound_range_exceeded.set_handler_fn(exceptions::bound_range_exceeded);
    idt.device_not_available.set_handler_fn(exceptions::device_not_available);
    idt.general_protection_fault.set_handler_fn(exceptions::general_protection_fault);
    idt.invalid_opcode.set_handler_fn(exceptions::invalid_opcode);
    idt.machine_check.set_handler_fn(exceptions::machine_check);
    idt.non_maskable_interrupt.set_handler_fn(exceptions::non_maskable_interrupt);
    idt.virtualization.set_handler_fn(exceptions::virtualization);
    idt.x87_floating_point.set_handler_fn(exceptions::x87_floating_point);
    idt.stack_segment_fault.set_handler_fn(exceptions::stack_segment_fault);
    idt.simd_floating_point.set_handler_fn(exceptions::simd_floating_point);
    idt.overflow.set_handler_fn(exceptions::overflow);
    idt.debug.set_handler_fn(exceptions::debug);
    // irq
    idt[InterruptIndex::Timer.into()].set_handler_fn(irq::timer);
    idt[InterruptIndex::KeyBoard.into()].set_handler_fn(irq::keyboard);
    // no support yet
    // idt[ipi::IpiKind::WakeUp.into()].set_handler_fn(ipi::ipi_wakeup);
    // idt[ipi::IpiKind::Switch.into()].set_handler_fn(ipi::ipi_switch);
    // idt[ipi::IpiKind::Tlb.into()].set_handler_fn(ipi::ipi_tlb);
    // idt[ipi::IpiKind::Pit.into()].set_handler_fn(ipi::ipi_pit);
    // idt[SystemCall::Base].set_handler_fn();
    // idt[SystemCall::Base].set_flags(IdtFlags::PRESENT | IdtFlags::RING_3 | IdtFlags::INTERRUPT);
    idt
}

pub fn init_idt() {
    IDT.load();
}


