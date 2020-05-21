///! 启用或关闭中断，以及中断的操作
#[inline]
pub fn system_pause() {
    unsafe { llvm_asm!("pause" : : : : "intel", "volatile"); }
}

/// 开启中断，已经使用unsafe包裹
#[inline]
pub fn enable_interrupt() {
    unsafe {
        llvm_asm!("sti" :::: "volatile");
    }
}

/// 屏蔽中断，已经使用unsafe包裹
#[inline]
pub fn disable_interrupt() {
    unsafe {
        llvm_asm!("cli" :::: "volatile");
    }
}

/// 触发调试中断  breakpoint exception
#[inline]
pub fn int3() {
    unsafe {
        llvm_asm!("int3" :::: "volatile");
    }
}

/// 对int n指令的封装
#[inline]
pub fn int_n(n: u32) {
    unsafe {
        llvm_asm!("int $0" :: "N" (n) :: "volatile");
    }
}
/// 暂停CPU，直到下一个中断到达。
#[inline]
pub fn hlt() {
    unsafe {
        llvm_asm!("hlt" :::: "volatile");
    }
}

/// Atomically enable interrupts and put the CPU to sleep
#[inline]
pub fn enable_interrupt_and_hlt() {
    unsafe {
        llvm_asm!("sti;hlt"::::"volatile")
    }
}

#[inline]
pub fn enable_interrupt_and_nop() {
    unsafe {
        llvm_asm!("sti;nop"::::"volatile")
    }
}

/// 返回是否启用中断。
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
        disable_interrupt();
    }

    // do `f` while interrupts are disabled
    let ret = f();

    // re-enable interrupts if they were previously enabled
    if saved_intpt_flag {
        enable_interrupt();
    }

    // return the result of `f` to the caller
    ret
}