// ------------------- byte 8 bits -------------------------------------
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
    result
}

pub unsafe fn outb(value: u8, port: u16) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
}
// ------------------- 2byte 16bits -------------------------------------
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("inw %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
    result
}

pub unsafe fn outw(value: u16, port: u16) {
    asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
}

// ------------------- 4byte 32bits -------------------------------------
pub unsafe fn inl(port: u16) -> u32 {
    let result: u32;
    asm!("inl %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
    result
}

pub unsafe fn outl(value: u32, port: u16) {
    asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
}