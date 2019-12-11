use crate::ia_32e::descriptor::DescriptorTablePointer;
use crate::ia_32e::descriptor::SegmentSelector;

#[inline]
pub unsafe fn ldgt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    asm!("ltr $0" :: "r" (sel.0));
}
