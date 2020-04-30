use crate::ia_32e::VirtAddr;

#[inline]
pub unsafe fn flush(addr: VirtAddr) {
    llvm_asm!("invlpg ($0)": :"r"(addr.as_u64()) :"memory")
}

/// 重新赋值CR3 使更改后的页表生效 更改页表后原页表依旧缓存与TLB中，重新加载页目录迫使CR3寄存器刷新TLB
#[inline]
pub unsafe fn flush_all() {
    use crate::ia_32e::cpu::control::CR3;
    let (frame, flags) = CR3::read();
    CR3::write(frame, flags)
}