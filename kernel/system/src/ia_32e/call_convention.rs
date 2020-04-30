use crate::alloc::string::String;
use crate::ia_32e::VirtAddr;

#[allow(dead_code)]
#[repr(packed)]
pub struct ScratchRegisters {
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rsi: usize,
    rdi: usize,
    rdx: usize,
    rcx: usize,
    rax: usize,
}

impl ScratchRegisters {
    pub fn dump(&self) -> String {
        format!("[ScratchRegisters] RAX:{} RCX:{} RDX:{} RDI:{} RSI:{} R8:{} R9:{} R10:{} R11:{}\n", { self.rax }, { self.rcx }, { self.rdx }, { self.rdi }, { self.rsi }, { self.r8 }, { self.r9 }, { self.r10 }, { self.r11 })
    }
    pub fn rax(&self) -> VirtAddr {
        VirtAddr::new({ self.rax } as u64)
    }
    pub fn rcx(&self) -> VirtAddr {
        VirtAddr::new({ self.rcx } as u64)
    }
    pub fn rdx(&self) -> VirtAddr {
        VirtAddr::new({ self.rdx } as u64)
    }
    pub fn rdi(&self) -> VirtAddr {
        VirtAddr::new({ self.rdi } as u64)
    }
    pub fn rsi(&self) -> VirtAddr {
        VirtAddr::new({ self.rsi } as u64)
    }
    pub fn r8(&self) -> VirtAddr {
        VirtAddr::new({ self.r8 } as u64)
    }
    pub fn r9(&self) -> VirtAddr {
        VirtAddr::new({ self.r9 } as u64)
    }
    pub fn r10(&self) -> VirtAddr {
        VirtAddr::new({ self.r10 } as u64)
    }
    pub fn r11(&self) -> VirtAddr {
        VirtAddr::new({ self.r11 } as u64)
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! push_scratch {
    () => (llvm_asm!(
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
    () => (llvm_asm!(
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
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbp: usize,
    rbx: usize,
}

impl PreservedRegisters {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[PreservedRegisters] r15:{},r14:{},r13:{},r12:{},rbp:{},rbx:{}\n", { self.r15 }, { self.r14 }, { self.r13 }, { self.r12 }, { self.rbp }, { self.rbx })
    }
    pub fn r15(&self) -> VirtAddr {
        VirtAddr::new({ self.r15 } as u64)
    }
    pub fn r14(&self) -> VirtAddr {
        VirtAddr::new({ self.r14 } as u64)
    }
    pub fn r13(&self) -> VirtAddr {
        VirtAddr::new({ self.r13 } as u64)
    }
    pub fn r12(&self) -> VirtAddr {
        VirtAddr::new({ self.r12 } as u64)
    }
    pub fn rbp(&self) -> VirtAddr {
        VirtAddr::new({ self.rbp } as u64)
    }
    pub fn rbx(&self) -> VirtAddr {
        VirtAddr::new({ self.rbx } as u64)
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! push_preserved {
    () => {
        llvm_asm!(
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
        llvm_asm!(
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
    rip: usize,
    cs: usize,
    rflags: usize,
    rsp: usize,
    ss: usize,
}

impl IretRegisters {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[IretRegister]: rip:{},cs:{},rflags:{},rsp:{},ss:{}\n", { self.rip }, { self.cs }, { self.rflags }, { self.rsp }, { self.ss })
    }
    pub fn rip(&self) -> VirtAddr {
        VirtAddr::new({ self.rip } as u64)
    }
    pub fn cs(&self) -> VirtAddr {
        VirtAddr::new({ self.cs } as u64)
    }
    pub fn rflags(&self) -> VirtAddr {
        VirtAddr::new({ self.rflags } as u64)
    }
    pub fn rsp(&self) -> VirtAddr {
        VirtAddr::new({ self.rsp } as u64)
    }
    pub fn ss(&self) -> VirtAddr {
        VirtAddr::new({ self.ss } as u64)
    }
}

impl IretRegisters {
    pub fn iret() {
        unsafe {
            llvm_asm!(
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
        llvm_asm!(
            "iretq"
            : : : : "intel", "volatile"
        )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! push_fs {
    () => {
    llvm_asm!(
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
    llvm_asm!(
        "pop fs"
        ::::"intel","volatile"
    )
    };
}
#[macro_export]
#[allow(unused_macros)]
macro_rules! cld {
    () => {
    llvm_asm!(
        "cld"
        ::::"intel","volatile"
    )
    };
}


#[repr(packed)]
pub struct InterruptStack {
    fs: usize,
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub iret: IretRegisters,
}

impl InterruptStack {
    /// https://github.com/rust-lang/rust/issues/46043
    pub fn dump(&self) -> String {
        format!("[FS] {} \n{} {} {}", { self.fs }, self.preserved.dump(), self.scratch.dump(), self.iret.dump())
    }

    pub fn fs(&self) -> VirtAddr {
        VirtAddr::new({ self.fs } as u64)
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
            fs_push!();
            cld!();

            inner();

            fs_pop!();
            scratch_pop!();
            iret!();
        }
    };
}
#[macro_export]
macro_rules! get_rsp {
    () => {
        {
            let mut __rsp = 0_usize;
            llvm_asm!("mov $0,rsp" : "=r"(__rsp) : : : "intel", "volatile");
            __rsp
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

            let rsp = $crate::get_rsp!();
            inner(&mut *(rsp as *mut $crate::ia_32e::call_convention::InterruptStack));

            // $crate::pop_fs!();
            $crate::pop_preserved!();
            $crate::pop_scratch!();
            $crate::iret!();
        }
    };
}
