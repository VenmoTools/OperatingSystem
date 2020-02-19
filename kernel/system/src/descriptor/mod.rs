pub use gdt::init_gdt;
pub use idt::{init_idt, init_pics};

mod gdt;
mod idt;

