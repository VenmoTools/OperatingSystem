use crate::descriptor::Selectors;

#[naked]
pub unsafe fn go_to_user_mode(ip: usize, sp: usize) {
    use crate::descriptor::GDT;
    let selector: &Selectors = &GDT.1;
    // push ip sp
    llvm_asm!("
          push r10;\
          push r11;\
          push r12;\
          push r13;\
          push r14"
          : :"{r10}"(selector.user_data_selector.0), // Data segment
           "{r11}"(sp), // Stack pointer
           "{r12}"(1 << 9), // Flags - Set interrupt enable flag
           "{r13}"(selector.user_code_selector.0), // Code segment
           "{r14}"(ip) : : "intel", "volatile"
    );
    // Go to usermode
    llvm_asm!("mov ds, r14d
         mov es, r14d
         mov fs, r15d
         mov gs, r14d
         xor rax, rax
         xor rbx, rbx
         xor rcx, rcx
         xor rdx, rdx
         xor rsi, rsi
         xor rdi, rdi
         xor rbp, rbp
         xor r8, r8
         xor r9, r9
         xor r10, r10
         xor r11, r11
         xor r12, r12
         xor r13, r13
         xor r14, r14
         fninit
         pop rdi
         iretq"
         : : "{r14}"(selector.user_data_selector.0): : "intel", "volatile");
    unreachable!();
}