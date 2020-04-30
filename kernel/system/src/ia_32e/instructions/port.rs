// ------------------- byte 8 bits -------------------------------------
/// 端口读操作（8位）
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    llvm_asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
    result
}
/// 端口写操作（8位）
pub unsafe fn outb(value: u8, port: u16) {
    llvm_asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
}
// ------------------- 2byte 16bits -------------------------------------
/// 端口读操作（16位）
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    llvm_asm!("inw %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
    result
}
/// 端口写操作（16位）
pub unsafe fn outw(value: u16, port: u16) {
    llvm_asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
}

// ------------------- 4byte 32bits -------------------------------------
/// 端口读操作（32位）
pub unsafe fn inl(port: u16) -> u32 {
    let result: u32;
    llvm_asm!("inl %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
    result
}
/// 端口写操作（32位）
pub unsafe fn outl(value: u32, port: u16) {
    llvm_asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
}