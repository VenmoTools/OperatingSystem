///! 用于封装基本的寄存器操作指令
/// 向CR0寄存器写入64位数据
#[inline]
pub unsafe fn write_cr0(value: u64) {
    llvm_asm!("mov $0, %cr0" ::"r"(value):"memory")
}

/// 向CR0寄存器读取64位数据
#[inline]
pub unsafe fn read_cr0() -> u64 {
    let mut value: u64 = 0;
    llvm_asm!("mov %cr0, $0" :"=r"(value));
    value
}

/// 向CR2寄存器写入64位数据
#[inline]
pub unsafe fn write_cr2(value: u64) {
    llvm_asm!("mov $0, %cr2" ::"r"(value):"memory")
}

/// 向CR2寄存器读取64位数据
#[inline]
pub unsafe fn read_cr2() -> u64 {
    let mut value: u64 = 0;
    llvm_asm!("mov %cr2, $0" :"=r"(value));
    value
}

/// 向CR3寄存器写入64位数据
#[inline]
pub unsafe fn write_cr3(value: u64) {
    llvm_asm!("mov $0, %cr3" ::"r"(value):"memory")
}

/// 向CR3寄存器读取64位数据
#[inline]
pub unsafe fn read_cr3() -> u64 {
    let mut value: u64 = 0;
    llvm_asm!("mov %cr3, $0" :"=r"(value));
    value
}

/// 从CR4寄存器读取64位数据
#[inline]
pub unsafe fn read_cr4() -> u64 {
    let mut value = 0_u64;
    llvm_asm!("mov %cr4,$0" : "=r"(value));
    value
}

/// 向CR4寄存器写入64位数据
#[inline]
pub unsafe fn write_cr4(data: u64) {
    llvm_asm!("mov $0,%cr4"::"r"(data):"memory")
}


/// 从当前RFLAGS 寄存器中读取64位数据
#[inline]
pub unsafe fn read_rflags() -> u64 {
    let mut r: u64 = 0;
    llvm_asm!("pushfq; popq $0" : "=r"(r) :: "memory");
    r
}

/// 将`val`值写入rflags寄存器
#[inline]
pub unsafe fn write_raw(val: u64) {
    llvm_asm!("pushq $0; popfq" :: "r"(val) : "memory" "flags");
}

/// 获取当前的RIP指针的值
#[inline(always)]
pub fn read_rip() -> u64 {
    let mut rip: u64 = 0;
    unsafe {
        llvm_asm!(
            "lea (%rip), $0"
            : "=r"(rip) ::: "volatile"
        );
    }
    rip
}

/// 从msr寄存器中读取64位数据
#[inline]
pub fn rdmsr(msr: u32) -> u64 {
    let (mut high, mut low) = (0_u32, 0_32);
    unsafe{
        llvm_asm!("rdmsr"
        : "={eax}"(low),"={edx}"(high)
        : "{ecx}"(msr)
        : "memory"
        : "volatile"
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// 从msr寄存器中写入64位数据
#[inline]
pub unsafe fn wrmsr(msr: u32, data: u64) {
    let low = data as u32;
    let high = (data >> 32) as u32;
    llvm_asm!("wrmsr"
        :
        : "{ecx}"(msr),"{eax}"(low),"{edx}"(high)
        : "memory"
        : "volatile"
        )
}

#[inline]
pub unsafe fn wrmsr2(msr: u32, data1: u32,data2:u32) {
    llvm_asm!("wrmsr"
        :
        : "{ecx}"(msr),"{eax}"(data1),"{edx}"(data2)
        : "memory"
        : "volatile"
        )
}