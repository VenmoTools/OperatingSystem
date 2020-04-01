pub use gdt::init_gdt_and_tss;
// pub use idt::init_idt;
pub use idt_not_call::init_idt;

mod gdt;
// mod idt;
mod idt_not_call;

