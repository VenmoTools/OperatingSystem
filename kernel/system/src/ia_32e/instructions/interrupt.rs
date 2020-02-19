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

/// Returns whether interrupts are enabled.
pub fn are_enabled() -> bool {
    use crate::bits::RFlags;
    use super::rflags::read_flags;
    read_flags().contains(RFlags::INTERRUPT_FLAG)
}

/// Run a closure with disabled interrupts.
///
/// Run the given closure, disabling interrupts before running it (if they aren't already disabled).
/// Afterwards, interrupts are enabling again if they were enabled before.
///
/// If you have other `enable` and `disable` calls _within_ the closure, things may not work as expected.
///
/// # Examples
///
/// ```ignore
/// // interrupts are enabled
/// without_interrupts(|| {
///     // interrupts are disabled
///     without_interrupts(|| {
///         // interrupts are disabled
///     });
///     // interrupts are still disabled
/// });
/// // interrupts are enabled again
/// ```
pub fn without_interrupts<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
{
    // true if the interrupt flag is set (i.e. interrupts are enabled)
    let saved_intpt_flag = are_enabled();

    // if interrupts are enabled, disable them for now
    if saved_intpt_flag {
        disable();
    }

    // do `f` while interrupts are disabled
    let ret = f();

    // re-enable interrupts if they were previously enabled
    if saved_intpt_flag {
        enable();
    }

    // return the result of `f` to the caller
    ret
}