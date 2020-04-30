///! 封装关于cs tss等段操作指令
use crate::ia_32e::descriptor::SegmentSelector;

/// 加载cs段选择子
/// 在这里没有直接使用mov指令加载到cs寄存器，把新的选择子
/// 压到栈中，并且使用lretq重新加载cs寄存器，并在1:处继续
pub unsafe fn set_cs(selector: SegmentSelector) {
    #[inline(always)]
    unsafe fn inner(selector: SegmentSelector) {
        llvm_asm!(
            "pushq $0;\
            leaq 1f(%rip), %rax;\
            pushq %rax;\
            lretq;\
            1:"
            :
            : "ri"(u64::from(selector.0))
            : "rax" "memory"
        );
    }
    inner(selector);
}

/// 加载ss段选择子
pub unsafe fn load_ss(selector:SegmentSelector){
    llvm_asm!(
        "movw $0, %ss"
        :
        : "r"(selector.0)
        :"memory"
    );
}

/// 加载ds段选择子
pub unsafe fn load_ds(selector:SegmentSelector){
    llvm_asm!(
        "movw $0,%ds"
        :
        :"r"(selector.0)
        :"memory"
    );
}
/// 加载es段选择子
pub unsafe fn load_es(selector:SegmentSelector){
    llvm_asm!(
        "movw $0,%es"
        :
        :"r"(selector.0)
        :"memory"
    );
}
/// 加载fs段选择子
pub unsafe fn load_fs(selector:SegmentSelector){
    llvm_asm!(
        "movw $0, %fs"
        :
        :"r"(selector.0)
        :"memory"
    );
}
/// 加载gs段选择子
pub unsafe fn load_gs(selector:SegmentSelector){
    llvm_asm!(
        "movw $0, %gs"
        :
        :"r"(selector.0)
        :"memory"
    );
}
/// swapgs指令
pub unsafe fn swap_gs(){
    llvm_asm!(
        "swapgs"
        :
        :
        :"memory"
        :"volatile"
    );
}

pub unsafe fn load_gdt(){

}

/// 获取当前的代码段选择子
/// 获取失败的时候会返回0
pub fn cs() -> SegmentSelector{
    let mut segment:u16 = 0;

    unsafe{
        llvm_asm!(
            "mov %cs, $0"
            :"=r"(segment)
        );
    }
    SegmentSelector(segment)
}
