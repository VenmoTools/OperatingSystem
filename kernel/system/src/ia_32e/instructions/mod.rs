///! 封装了常用的x86指令
pub mod tables;
pub mod segmention;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod port;
pub mod register;
pub mod interrupt;
pub mod rflags;
pub mod page_table;
pub mod apic;


