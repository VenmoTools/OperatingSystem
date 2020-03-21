#[derive(Debug)]
pub struct ProcessRegister {
    /// FX valid?
    loadable: bool,
    /// FX location
    fx: usize,
    /// 进程使用的页表
    cr3: usize,
    /// RFLAGS寄存器
    rflags: usize,
    /// RBX寄存器
    rbx: usize,
    /// R12 寄存器
    r12: usize,
    /// R13 寄存器
    r13: usize,
    /// R14 寄存器
    r14: usize,
    /// R15 寄存器
    r15: usize,
    /// rbp寄存器
    rbp: usize,
    /// 栈指针寄存器
    rsp: usize,
}

impl ProcessRegister {
    pub const STACK_ELE_SIZE: usize = core::mem::size_of::<usize>();

    pub fn new() -> ProcessRegister {
        Self {
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

    pub fn get_page_table(&self) -> usize {
        self.cr3
    }

    pub fn set_stack_register(&mut self, address: usize) {
        self.rsp = address
    }

    pub fn set_page_table(&mut self, address: usize) {
        self.cr3 = address
    }

    pub fn set_fx(&mut self, address: usize) {
        self.fx = address
    }

    pub unsafe fn push_stack(&mut self, value: usize) {
        self.rsp -= Self::STACK_ELE_SIZE;
        *(self.rsp as *mut usize) = value;
    }
    pub unsafe fn pop_stack(&mut self) -> usize {
        let value = *(self.rsp as *const usize);
        self.rsp += Self::STACK_ELE_SIZE;
        value
    }
}