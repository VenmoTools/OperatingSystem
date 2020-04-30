pub use gdt::{GDT, init_gdt, init_tss, Selectors, TSS};
pub use idt::{disable_8259a, init_apic, init_idt};

mod gdt;
mod idt;

