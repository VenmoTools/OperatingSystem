use system::ia_32e::call_convention::InterruptStack;
use system::result::Result;

use crate::process::process_mut;

pub fn syscall(rax: usize, rbx: usize, rcx: usize, rdx: usize, es: usize, rflgas: usize, rbp: usize, stack: &mut InterruptStack) {
    #[inline(always)]
    fn handler(rax: usize, rbx: usize, rcx: usize, rdx: usize, _es: usize, rflgas: usize, _rbp: usize, _stack: &mut InterruptStack) -> Result<usize> {
        println!("rax:{}", rax);
        if rax == 0x2000_0000 {
            test_syscall(rbx, rcx, rdx, rflgas)
        }
        Ok(0)
    }

    {
        let process = process_mut();
        if let Some(cur) = process.current() {
            let mut cur_process = cur.write();
            cur_process.syscall = Some((rax, rbx, rcx, rdx, rflgas, es))
        }
    }

    let _res = handler(rax, rbx, rcx, rdx, es, rflgas, rbp, stack);

    {
        let process = process_mut();
        if let Some(cur) = process.current() {
            let mut cur_process = cur.write();
            cur_process.syscall = None
        }
    }
}

pub fn test_syscall(rbx: usize, rcx: usize, rdx: usize, rflgas: usize) {
    println!("rbx: {}, rcx: {},rdx: {},rflags:{}", rbx, rcx, rdx, rflgas)
}