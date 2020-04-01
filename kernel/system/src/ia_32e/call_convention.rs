use crate::alloc::string::String;

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
    pub fn dump(&self) -> String {
        format!("[ScratchRegisters] RAX:{} RCX:{} RDX:{} RDI:{} RSI:{} R8:{} R9:{} R10:{} R11:{}\n", {self.rax}, {self.rcx}, {self.rdx}, {self.rdi}, {self.rsi}, {self.r8},{ self.r9},{ self.r10},{ self.r11})
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! push_scratch {
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
#[macro_export]
#[allow(unused_macros)]
macro_rules! pop_scratch {
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

impl PreservedRegisters {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[PreservedRegisters] r15:{},r14:{},r13:{},r12:{},rbp:{},rbx:{}\n", {self.r15}, {self.r14}, {self.r13},{ self.r12}, {self.rbp}, {self.rbx})
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! push_preserved {
    () => {
        asm!(
                "push rbx
                push rbp
                push r12
                push r13
                push r14
                push r15"
                : : : : "intel", "volatile"
            )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! pop_preserved {
    () => {
        asm!(
            "pop r15
            pop r14
            pop r13
            pop r12
            pop rbp
            pop rbx"
            : : : : "intel", "volatile"
        )
    };
}

#[allow(dead_code)]
#[repr(packed)]
pub struct IretRegisters {
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}

impl IretRegisters {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[IretRegister]: rip:{},cs:{},rflags:{},rsp:{},ss:{}\n", {self.rip}, {self.cs}, {self.rflags}, {self.rsp}, {self.ss})
    }
}

impl IretRegisters {
    pub fn iret() {
        unsafe {
            asm!(
    "iretq"
    : : : : "intel", "volatile"
    )
        }
    }
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! iret {
    () => {
        asm!(
            "iretq"
            : : : : "intel", "volatile"
        )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! push_fs {
    () => {
    asm!(
        "push fs;\
        mov rax,0x18;\
        mov fs,ax;"
        ::::"intel","volatile"
    )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! pop_fs {
    () => {
    asm!(
        "pop fs"
        ::::"intel","volatile"
    )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! cld {
    () => {
    asm!(
        "cld"
        ::::"intel","volatile"
    )
    };
}


#[repr(packed)]
pub struct InterruptStack {
    pub fs: usize,
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub iret: IretRegisters,
}

impl InterruptStack {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[FS] {} \n{} {} {}", {self.fs}, self.preserved.dump(), self.scratch.dump(), self.iret.dump())
    }
}

#[macro_export]
macro_rules! interrupt {
    ($name:ident,$func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name(){
            #[inline(never)]
            unsafe fn inner(){
                $func
            }
            // 保存scratch寄存器中数据
            scratch_push!();
            // 保存fs寄存器
            // fs_push!();
            cld!();
            inner();

            // fs_pop!();
            scratch_pop!();
            iret!();
        }
    };

}
#[macro_export]
macro_rules! interrupt_frame {
    ($name:ident,$stack:ident,$func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name(){
            #[inline(never)]
            unsafe fn inner($stack: &mut $crate::ia_32e::call_convention::InterruptStack){
                $func
            }

            $crate::push_scratch!();
            $crate::push_preserved!();
            // $crate::push_fs!();
             $crate::cld!();

            let mut rsp = 0_usize;
            asm!("mov $0,rsp" : "=r"(rsp) : : : "intel", "volatile");
            inner(&mut *(rsp as *mut $crate::ia_32e::call_convention::InterruptStack));

            // $crate::pop_fs!();
            $crate::pop_preserved!();
            $crate::pop_scratch!();
            $crate::iret!();
        }
    };
}
