pub use gdt::init_gdt_and_tss;
pub use idt::init_idt;

mod gdt;
mod idt;

