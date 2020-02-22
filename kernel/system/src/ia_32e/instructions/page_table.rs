use crate::ia_32e::VirtAddr;

#[inline]
pub unsafe fn flush(addr: VirtAddr) {
    asm!("invlpg ($0)": :"r"(addr.as_u64()) :"memory")
}

#[inline]
pub unsafe fn flush_all() {
    use crate::ia_32e::cpu::control::CR3;
    let (frame, flags) = CR3::read();
    CR3::write(frame, flags)
}