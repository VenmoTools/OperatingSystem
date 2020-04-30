use system::{get_rsp, pop_scratch, push_scratch};

#[derive(Clone, Debug)]
pub struct ProcessRegister {
    /// FX valid?
    loadable: bool,
    /// FX location
    fx: usize,
    /// Page table pointer
    cr3: usize,
    /// RFLAGS register
    rflags: usize,
    /// RBX register
    rbx: usize,
    /// R12 register
    r12: usize,
    /// R13 register
    r13: usize,
    /// R14 register
    r14: usize,
    /// R15 register
    r15: usize,
    /// Base pointer
    rbp: usize,
    /// Stack pointer
    rsp: usize,
}

impl ProcessRegister {
    pub fn new() -> ProcessRegister {
        ProcessRegister {
            loadable: false,
            fx: 0,
            cr3: 0,
            rflags: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: 0,
            rsp: 0,
        }
    }
    /// get context page table
    pub fn get_page_table(&mut self) -> usize { self.cr3 }
    /// set context fx register
    pub fn set_fx(&mut self, address: usize) {
        self.fx = address;
    }
    /// set context page table
    pub fn set_page_table(&mut self, address: usize) {
        self.cr3 = address;
    }

    /// set context stack
    pub fn set_stack(&mut self, address: usize) {
        self.rsp = address;
    }

    /// push signal handler function to context stack
    /// # Safety
    /// the function is unsafe cause this function will call other unsafe function
    pub unsafe fn signal_stack(&mut self, handler: extern fn(usize), sig: u8) {
        self.push_stack(sig as usize);
        self.push_stack(handler as usize);
        self.push_stack(signal_handler_wrapper as usize);
    }
    /// push data into context stack
    /// # Safety:
    /// the function is unsafe cause need push data to user specified stack pointer
    /// user should check rsp is valid
    pub unsafe fn push_stack(&mut self, value: usize) {
        self.rsp -= core::mem::size_of::<usize>();
        *(self.rsp as *mut usize) = value;
    }
    /// pop data from context stack
    /// # Safety:
    /// the function is unsafe cause need pop data to user specified stack pointer
    /// user should check rsp is valid
    pub unsafe fn pop_stack(&mut self) -> usize {
        let value = *(self.rsp as *const usize);
        self.rsp += core::mem::size_of::<usize>();
        value
    }

    /// switch to next context
    #[inline(never)]
    #[naked]
    #[cold]
    pub unsafe fn switch_to(&mut self, next: &mut ProcessRegister) {
        // save fx register
        llvm_asm!("fxsave [$0]"         :               : "r"(self.fx)      : "memory" :"intel", "volatile");
        //todo:
        if self.loadable {
            llvm_asm!("fxrstor64 [$0]"  :               :"r"(next.fx)       : "memory" :"intel", "volatile");
        } else {
            llvm_asm!("fninit [$0]"     :               :                   : "memory" :"intel", "volatile");
        }
        // switch cr3
        llvm_asm!("mov $0, cr3"    : "=r"(self.cr3)     :                   : "memory" : "intel", "volatile");
        if self.cr3 != next.cr3 {
            llvm_asm!("mov cr3,$0" :                    :"r"(next.cr3)      : "memory" : "intel", "volatile");
        }
        // switch rflags
        llvm_asm!("pushfq; pop $0" :"=r"(self.rflags)   :                   : "memory" : "intel", "volatile");
        llvm_asm!("push $0; popfq" :                    :"r"(next.rflags)   : "memory" : "intel", "volatile");
        // switch rbx
        llvm_asm!("mov $0,rbx"     :"=r"(self.rbx)      :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov rbx,$0"     :                    :"r"(next.rbx)      : "memory" : "intel", "volatile");
        // switch r12
        llvm_asm!("mov $0, r12"     :"=r"(self.r12)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov r12, $0"     :                   :"r"(next.r12)      : "memory" : "intel", "volatile");
        // switch r13
        llvm_asm!("mov $0, r13"     :"=r"(self.r13)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov r13, $0"     :                   :"r"(next.r13)      : "memory" : "intel", "volatile");
        // switch r14
        llvm_asm!("mov $0, r14"     :"=r"(self.r14)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov r14, $0"     :                   :"r"(next.r14)      : "memory" : "intel", "volatile");
        // switch r15
        llvm_asm!("mov $0, r15"     :"=r"(self.r15)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov r15, $0"     :                   :"r"(next.r15)      : "memory" : "intel", "volatile");
        // switch rsp
        llvm_asm!("mov $0, rsp"     :"=r"(self.rsp)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov rsp, $0"     :                   :"r"(next.rsp)      : "memory" : "intel", "volatile");
        // switch rbp
        llvm_asm!("mov $0, rbp"     :"=r"(self.rbp)     :                   : "memory" : "intel", "volatile");
        llvm_asm!("mov rbp, $0"     :                   :"r"(next.rbp)      : "memory" : "intel", "volatile");
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct SignalHandlerStack {
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rsi: usize,
    rdi: usize,
    rdx: usize,
    rcx: usize,
    rax: usize,
    handler: extern fn(usize),
    sig: usize,
    rip: usize,
}


#[naked]
unsafe extern fn signal_handler_wrapper() {
    #[inline(never)]
    unsafe fn inner(stack: &SignalHandlerStack) {
        (stack.handler)(stack.sig);
    }
    push_scratch!();
    let rsp = get_rsp!();
    inner(&*(rsp as *const SignalHandlerStack));
    pop_scratch!();
}

