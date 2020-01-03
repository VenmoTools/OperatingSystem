///! 启用或关闭中断，以及中断的操作

/// 开启中断，已经使用unsafe包裹
#[inline]
pub fn enable() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

/// 屏蔽中断，已经使用unsafe包裹
#[inline]
pub fn disable() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

/// 触发调试中断  breakpoint exception
#[inline]
pub fn int3() {
    unsafe {
        asm!("int3" :::: "volatile");
    }
}

/// 对int n指令的封装
#[inline]
pub fn int_n(n: u32) {
    unsafe {
        asm!("int $0" :: "N" (n) :: "volatile");
    }
}
/// 暂停CPU，直到下一个中断到达。
#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}