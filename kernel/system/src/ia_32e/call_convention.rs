use crate::println;

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
    pub fn dump(&self) {
        println!("RAX:   {:>016X}", { self.rax });
        println!("RCX:   {:>016X}", { self.rcx });
        println!("RDX:   {:>016X}", { self.rdx });
        println!("RDI:   {:>016X}", { self.rdi });
        println!("RSI:   {:>016X}", { self.rsi });
        println!("R8:    {:>016X}", { self.r8 });
        println!("R9:    {:>016X}", { self.r9 });
        println!("R10:   {:>016X}", { self.r10 });
        println!("R11:   {:>016X}", { self.r11 });
    }
}

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
    pub fn dump(&self) {
        println!("RBX:   {:>016X}", { self.rbx });
        println!("RBP:   {:>016X}", { self.rbp });
        println!("R12:   {:>016X}", { self.r12 });
        println!("R13:   {:>016X}", { self.r13 });
        println!("R14:   {:>016X}", { self.r14 });
        println!("R15:   {:>016X}", { self.r15 });
    }
}

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
    pub fn iret() {
        unsafe {
            asm!(
                "iretq"
                : : : : "intel", "volatile"
            )
        }
    }
}
#[allow(unused_macros)]
macro_rules! iret {
    () => {
        asm!(
            "iretq"
            : : : : "intel", "volatile"
        )
    };
}
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
#[allow(unused_macros)]
macro_rules! pop_fs {
    () => {
    asm!(
        "pop fs"
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

#[macro_export]
macro_rules! interrupt {
    ($name:ident,$func:block) => {
        #[naked]
        pub unsafe extern fn $name(){
            #[inline(never)]
            unsafe fn inner(){
                $func
            }
            // 保存scratch寄存器中数据
            scratch_push!();
            // 保存fs寄存器
            fs_push!();

            inner();

            fs_pop!();
            scratch_pop!();
            iret!();
        }
    };

}
#[macro_export]
macro_rules! interrupt_frame {
    ($name:ident,$func:block) => {
        #[naked]
        pub unsafe extern fn $name(){
            #[inline(never)]
            unsafe fn inner(stack: &mut $crate::ia_32e::call_convention::InterruptStack){
                $func
            }

            push_scratch!();
            push_preserved!();
            push_fs!();

            let rsp = 0_usize;
            asm!("" : "={rsp}"(rsp) : : : "intel", "volatile");
            inner(&mut *(rsp as *mut $crate::ia_32e::call_convention::InterruptStack));

            pop_fs!();
            pop_preserved!();
            pop_scratch!();
            iret!();
        }
    };
}
