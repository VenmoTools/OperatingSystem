use spin::Mutex;
use system::ia_32e::apic::LocalAPIC;
use system::ia_32e::cpu::ChainedPics;
use system::ia_32e::descriptor::InterruptDescriptorTable;
// use system::bits::flags::IdtFlags;
use system::ia_32e::instructions::page_table::flush_all;
use system::interrupt_frame;

use lazy_static::lazy_static;

use crate::println;

pub const PIC_MAIN: u8 = 32;
pub const PIC_SLAVE: u8 = PIC_MAIN + 8;
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_MAIN, PIC_SLAVE) });


lazy_static! {
    static ref LOCAL_APIC: Mutex<LocalAPIC> =  Mutex::new(unsafe{LocalAPIC::init()});
}
lazy_static! {
    static ref IDT: InterruptDescriptorTable = init();
}

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

pub enum SystemCall {
    Base = 0x80
}


pub fn init() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_by_zero.set_handler_fn(divide_by_zero);
    idt.breakpoint.set_handler_fn(breakpoint);
    idt.page_fault.set_handler_fn(page_fault);
    idt.divide_by_zero.set_handler_fn(divide_by_zero);
    idt.invalid_tss.set_handler_fn(invalid_tss);
    idt.security_exception.set_handler_fn(security_exception);
    idt.segment_not_present.set_handler_fn(segment_not_present);
    idt.alignment_check.set_handler_fn(alignment_check);
    idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded);
    idt.device_not_available.set_handler_fn(device_not_available);
    idt.general_protection_fault.set_handler_fn(general_protection_fault);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.machine_check.set_handler_fn(machine_check);
    idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt);
    idt.virtualization.set_handler_fn(virtualization);
    idt.x87_floating_point.set_handler_fn(x87_floating_point);
    idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
    idt.simd_floating_point.set_handler_fn(simd_floating_point);
    idt.overflow.set_handler_fn(overflow);
    idt.debug.set_handler_fn(debug);
    idt[IpiKind::WakeUp.into()].set_handler_fn(ipi_wakeup);
    idt[IpiKind::Switch.into()].set_handler_fn(ipi_switch);
    idt[IpiKind::Tlb.into()].set_handler_fn(ipi_tlb);
    idt[IpiKind::Pit.into()].set_handler_fn(ipi_pit);
    // idt[SystemCall::Base].set_handler_fn();
    // idt[SystemCall::Base].set_flags(IdtFlags::PRESENT | IdtFlags::RING_3 | IdtFlags::INTERRUPT);
    idt
}

interrupt_frame!(ipi_wakeup,_stack,{
    LOCAL_APIC.lock().eoi()
});

interrupt_frame!(ipi_switch,_stack,{
     LOCAL_APIC.lock().eoi()
});

interrupt_frame!(ipi_pit,_stack,{
    LOCAL_APIC.lock().eoi()
});

interrupt_frame!(ipi_tlb,_stack,{
    LOCAL_APIC.lock().eoi();
    flush_all();
});

pub fn init_idt() {
    IDT.load();
}

interrupt_frame!(divide_by_zero,stack,{
    println!("Divide by zero: {:?}",stack.dump());
});

interrupt_frame!(debug,stack,{
    println!("Debug trap {:?}",stack.dump());
});

interrupt_frame!(non_maskable,stack,{
    println!("Non-maskable interrupt: {:?}",stack.dump());
});

interrupt_frame!(breakpoint,stack,{
    println!("Breakpoint trap: {:?}",stack.dump());
});

interrupt_frame!(invalid_opcode, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(page_fault, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(double_fault, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(invalid_tss,stack,{
    println!("invalid_tss: {:?}",stack.dump());
});
interrupt_frame!(security_exception,stack,{
    println!("security_exception: {:?}",stack.dump());
});
interrupt_frame!(segment_not_present,stack,{
    println!("segment_not_present: {:?}",stack.dump());
});
interrupt_frame!(alignment_check,stack,{
    println!("alignment_check: {:?}",stack.dump());
});
interrupt_frame!(bound_range_exceeded,stack,{
    println!("bound_range_exceeded: {:?}",stack.dump());
});
interrupt_frame!(device_not_available,stack,{
    println!("device_not_available: {:?}",stack.dump());
});
interrupt_frame!(general_protection_fault,stack,{
    println!("general_protection_fault: {:?}",stack.dump());
});
interrupt_frame!(machine_check,stack,{
    println!("machine_check: {:?}",stack.dump());
});
interrupt_frame!(non_maskable_interrupt,stack,{
    println!("non_maskable_interrupt: {:?}",stack.dump());
});
interrupt_frame!(virtualization,stack,{
    println!("virtualization: {:?}",stack.dump());
});
interrupt_frame!(x87_floating_point,stack,{
    println!("x87_floating_point: {:?}",stack.dump());
});
interrupt_frame!(stack_segment_fault,stack,{
    println!("stack_segment_fault: {:?}",stack.dump());
});
interrupt_frame!(simd_floating_point,stack,{
    println!("simd_floating_point: {:?}",stack.dump());
});
interrupt_frame!(overflow,stack,{
    println!("overflow: {:?}",stack.dump());
});