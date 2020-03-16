
#[allow(dead_code)]
#[repr(packed)]
pub struct ScratchRegisters {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

impl ScratchRegisters {
    pub fn push() {
        unsafe {
            asm!(
                "push rax
                push rcx
                push rdx
                push rdi
                push rsi
                push r8
                push r9
                push r10
                push r11"
                : : : : "intel", "volatile"
            )
        }
    }

    pub fn pop(){
        unsafe{
            asm!(
                "pop r11
                pop r10
                pop r9
                pop r8
                pop rsi
                pop rdi
                pop rdx
                pop rcx
                pop rax"
                : : : : "intel", "volatile"
            )
        }
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct PreservedRegisters {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

impl PreservedRegisters{
    pub fn push(){
        unsafe{
            asm!(
                "push rbx
                push rbp
                push r12
                push r13
                push r14
                push r15"
                : : : : "intel", "volatile"
            )
        }
    }

    pub fn pop(){
        unsafe{
            asm!(
                "pop r15
                pop r14
                pop r13
                pop r12
                pop rbp
                pop rbx"
                : : : : "intel", "volatile"
            )
        }
    }

}

#[allow(dead_code)]
#[repr(packed)]
pub struct IretRegisters {
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize
}

impl IretRegisters{
    pub fn iret(){
        unsafe{
            asm!(
                "iretq"
                : : : : "intel", "volatile"
            )
        }
    }
}

#[allow(unused_macros)]
macro_rules! scratch_push {
    () => (asm!(
        "push rax
        push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11"
        : : : : "intel", "volatile"
    ));
}
#[allow(unused_macros)]
macro_rules! scratch_pop {
    () => (asm!(
        "pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi
        pop rdx
        pop rcx
        pop rax"
        : : : : "intel", "volatile"
    ));
}