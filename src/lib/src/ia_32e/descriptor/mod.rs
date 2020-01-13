///! x86系统使用的所有描述符 包裹GDT IDT TSS等
mod gdt;
mod idt;
mod tss;
mod segment;
mod descriptors;

pub use segment::SegmentSelector;
pub use descriptors::{Descriptor, DescriptorTablePointer};
pub use gdt::GlobalDescriptorTable;
pub use idt::{InterruptDescriptorTable, InterruptStackFrame, EntryOptions, HandlerFuncWithErrCode, InterruptStackFrameValue};
pub use tss::TaskStateSegment;
