use crate::bits::RFlags;

/// 读取原始RFLAGS寄存器值
pub fn read() -> u64 {
    let mut r = 0_u64;
    unsafe {
        asm!("pushfq; popq $0" : "=r"(r) ::"memory")
    };
    r
}

/// 读取RFLAGS寄存器转为RFLAGS
pub fn read_flags() -> RFlags {
    RFlags::from_bits_truncate(read())
}

/// 写入REFLAGS寄存器原始数据
pub fn write(val: u64) {
    unsafe { asm!("pushq $0; popfq" :: "r"(val) : "memory" "flags") };
}
/// 写入REFLAGS寄存器
pub fn write_flags(flags: RFlags) {
    let old_value = read();
    let reserved = old_value & !(RFlags::all().bits());
    let new_value = reserved | flags.bits();
    write(new_value);
}