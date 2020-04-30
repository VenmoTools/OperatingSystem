use crate::ia_32e::descriptor::DescriptorTablePointer;
///! 用于加载GDT IDT TSS的相关指令
use crate::ia_32e::descriptor::SegmentSelector;

/// 使用`lgdt`加载GDT描述符
#[inline]
pub unsafe fn ldgt(gdt: &DescriptorTablePointer) {
    llvm_asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// 使用`sgdt`取出GDTR寄存器的数据
#[inline]
pub fn sgdt() -> DescriptorTablePointer {
    let gdt = DescriptorTablePointer::empty();
    unsafe {
        llvm_asm!(
            "sgdt ($0)":"=r"(&gdt) : :"memory"
        )
    }
    gdt
}

/// 使用`sgdt`取出IDTR寄存器的数据
#[inline]
pub fn sidt() -> DescriptorTablePointer {
    let idt = DescriptorTablePointer::empty();
    unsafe {
        llvm_asm!(
            "sidt ($0)":"=r"(&idt)::"memory"
        )
    }
    idt
}


/// 使用`lidt`加载IDT描述符
#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    llvm_asm!("lidt ($0)" :: "r" (idt) : "memory");
}

/// 使用`ltr`加载TSS描述符
#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    llvm_asm!("ltr $0" :: "r" (sel.0));
}

#[inline]
pub unsafe fn load_tr(sel: SegmentSelector) {
    llvm_asm!("ltr $0" :: "r" (sel.0));
}