pub use gdt::{GDT, init_gdt, init_tss, Selectors, TSS};
pub use idt::{CONTROLLER, disable_8259a, init_apic, init_idt, InterruptIndex, PIC_MAIN, PIC_SLAVE, TICKS};

mod gdt;
mod idt;

