use lazy_static::lazy_static;
use system::bits::PageFaultErrorCode;
use system::ia_32e::apic::LocalAPIC;
use system::ia_32e::descriptor::{InterruptDescriptorTable, InterruptStackFrame};
use system::ia_32e::instructions::page_table::flush_all;

use crate::{loop_hlt, println};

static mut LOCAL_APIC: LocalAPIC = LocalAPIC {
    address: 0,
    x2apic: false,
};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum IpiKind {
    WakeUp = 0x40,
    Tlb = 0x41,
    Switch = 0x42,
    Pit = 0x43,
}

impl From<IpiKind> for usize {
    fn from(kind: IpiKind) -> Self {
        match kind {
            IpiKind::WakeUp => 0x40,
            IpiKind::Tlb => 0x41,
            IpiKind::Switch => 0x42,
            IpiKind::Pit => 0x43,
        }
    }
}

pub enum SystemCall {
    Base = 0x80
}

pub fn init_idt() {
    IDT.load();
}

//---------------------------------中断描述符表--------------------------------------

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
        idt.divide_by_zero.set_handler_fn(divide_by_zero);
        idt.invalid_tss.set_handler_fn(invalid_tss);
        idt.security_exception.set_handler_fn(security_exception);
        idt.segment_not_present.set_handler_fn(segment_not_present);
        idt.alignment_check.set_handler_fn(alignment_check);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded);
        idt.device_not_available.set_handler_fn(device_not_available);
        idt.general_protection_fault.set_handler_fn(general_protection_fault);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.machine_check.set_handler_fn(machine_check);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt);
        idt.virtualization.set_handler_fn(virtualization);
        idt.x87_floating_point.set_handler_fn(x87_floating_point);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
        idt.simd_floating_point.set_handler_fn(simd_floating_point);
        idt.overflow.set_handler_fn(overflow);
        idt.debug.set_handler_fn(debug);
        // 在IDT中为双重错误处理程序设置堆栈索引
        // unsafe {
        //     idt.double_fault.set_handler_fn(double_fault).set_stack_index(gdt::DOUBLE_FAULT_LIST_INDEX);
        // }
        idt[IpiKind::WakeUp.into()].set_handler_fn(ipi_wakeup);
        idt[IpiKind::Switch.into()].set_handler_fn(ipi_switch);
        idt[IpiKind::Tlb.into()].set_handler_fn(ipi_tlb);
        idt[IpiKind::Pit.into()].set_handler_fn(ipi_pit);
        // idt[SystemCall::Base].set_handler_fn();
        // idt[SystemCall::Base].set_flags(IdtFlags::PRESENT | IdtFlags::RING_3 | IdtFlags::INTERRUPT);
        idt
    };
}

//---------------------------------中断处理--------------------------------------
extern "x86-interrupt" fn ipi_wakeup(_stackframe: &mut InterruptStackFrame) {
    unsafe { LOCAL_APIC.eoi() }
}

extern "x86-interrupt" fn ipi_switch(_stackframe: &mut InterruptStackFrame) {
    unsafe { LOCAL_APIC.eoi() }
    //todo: Switch
}

extern "x86-interrupt" fn ipi_pit(_stackframe: &mut InterruptStackFrame) {
    unsafe { LOCAL_APIC.eoi() }
    //todo: pit
}

extern "x86-interrupt" fn ipi_tlb(_stackframe: &mut InterruptStackFrame) {
    unsafe {
        LOCAL_APIC.eoi();
        flush_all();
    }
}

/// 时钟中断
/// 该频率是常量HZ，该值一般是在100 ~ 1000之间。
/// 该中断的作用是为了定时更新系统日期和时间，使系统时间不断地得到跳转。
/// 另外该中断的中断处理函数除了更新系统时间外，还需要更新本地CPU统计数。
/// 指的是调用scheduler_tick递减进程的时间片，若进程的时间片递减到0，进程则被调度出去而放弃CPU使用权
extern "x86-interrupt" fn timer_interrupt(_stackframe: &mut InterruptStackFrame) {
    //todo:
}

/// 键盘中断
extern "x86-interrupt" fn keyboard_interrupt(_stackframe: &mut InterruptStackFrame) {
    //todo:
}

//---------------------------------中断处理--------------------------------------

//---------------------------------异常处理--------------------------------------
// 当一个异常发生后，CPU大概会做一下操作
// 1. 将某个寄存器的值压入栈中，其中包含指令寄存器和RFLAGS寄存器
// 2. 从IDT中读取响应的条目，例如当发生了段错误后CPU会读取第13号异常
// 3. 检查对应条目是否存在，如果没有则再次抛出异常（Double fault）
// 4. 如果是中断门则关闭硬件中断
// 5. 从GDT中指定特定的选择子加载到CS寄存器中
// 6. 跳转到对应的处理函数

// Preserved 和 Scratch 寄存器
// 调用将寄存器分为两个部分，Preserved寄存器和Scratch寄存器
//
// 在Preserved寄存器中的值在函数调用的过程中必须保持不变，被调用方在恢复原始值时才可以改变这些寄存器
// 因此被称为“被调用者保存”
// 常见模式为在函数开始时将这些寄存器保存在堆栈中，并在函数返回前恢复
//
// Scratch寄存器可以允许调用不加限制的更改其中的值，如果调用方想在函数调用时保留暂存寄存器的值，则它需要在函数调用之前备份和还原它 Scratch寄存器时调用者保存的
// 常见模式为在函数调用前将这些寄存器保存在堆栈中，函数结束后再将其恢复
//
// 在x86_64位系统中
//     Preserved寄存器（被调用者保存）：rbp, rbx, rsp, r12, r13, r14, r15
//     Scratch 寄存器 （调用者保存） ：rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11

/// 调试中断
extern "x86-interrupt" fn breakpoint(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:BREAKPOINT\n{:#?}", stackframe);
}

/// 缺页错误
/// 1、当MMU中确实没有创建虚拟页物理页映射关系，并且在该虚拟地址之后再没有当前进程的线性区vma的时候，可以肯定这是一个编码错误，这将杀掉该进程；
/// 2、当MMU中确实没有创建虚拟页物理页映射关系，并且在该虚拟地址之后存在当前进程的线性区vma的时候，这很可能是缺页异常，并且可能是栈溢出导致的缺页异常；
/// 3、当使用malloc/mmap等希望访问物理空间的库函数/系统调用后，由于linux并未真正给新创建的vma映射物理页，此时若先进行写操作，将如上面的2的情况产生缺页异常，若先进行读操作虽也会产生缺页异常，将被映射给默认的零页(zero_pfn)，等再进行写操作时，仍会产生缺页异常，这次必须分配物理页了，进入写时复制的流程；
/// 4、当使用fork等系统调用创建子进程时，子进程不论有无自己的vma，“它的”vma都有对于物理页的映射，但它们共同映射的这些物理页属性为只读，即linux并未给子进程真正分配物理页，当父子进程任何一方要写相应物理页时，导致缺页异常的写时复制
extern "x86-interrupt" fn page_fault(stackframe: &mut InterruptStackFrame, code: PageFaultErrorCode) {
    use system::ia_32e::cpu::control::CR2;
    println!("EXCEPTION:PageFault");
    println!("Access Address:{:?}", CR2::read());
    println!("Error Code:{:?}", code);
    println!("{:#?}", stackframe);
    loop_hlt();
}

/// 在处理先前（第一个）异常处理程序期间发生第二个异常时，可能会发生双重错误异常
/// 只有异常的非常特殊的组合才会导致双重错误
/// +--------------------------+------------------------+
/// | First Exception          |Second Exception        |
/// +--------------------------+------------------------+
/// |Divide-by-zero,           |Invalid TSS,            |
/// |Invalid TSS,              |Segment Not Present,    |
/// |Segment Not Present,      |Stack-Segment Fault,    |
/// |Stack-Segment Fault,      |General Protection Fault|
/// |General Protection Fault  |                        |
/// +--------------------------+------------------------+
/// |                          |Page Fault,             |
/// |                          |Invalid TSS,            |
/// |Page Fault                |Segment Not Present,    |
/// |                          |Stack-Segment Fault,    |
/// |                          |General Protection Fault|
/// +--------------------------+------------------------+
/// 当系统正在处理一个异常时，如果又检测到一个异常，处理器试图向系统通知一个双重故障，而不是通知第二个异常。
/// 双重故障属于中止类异常，所以在转入双重故障处理程序时，被保存的CS和EIP可能不指向引起双重故障的指令，
/// 而且指令的重新启动不支持双重故障。双重故障提供的出错码为0。
///
/// 当正处理一个段故障异常时，有可能又产生一个页故障。在这种情况下，通知给系统的是一个页故障异常而不是双重故障异常。但是，如果正处理一个段故障或页故障时，又一个段故障被检测到；或者如果正处理一个页故障时，又一个页故障被检测到，那么就引起双重故障。
/// 当正处理一个双重故障时，又一个段或页故障被检测到，那么处理器暂停执行指令，并进入关机方式。关机方式类似于处理器指令一条HLT指令后的状态：处理器空转，并维持到处理器接收到一个NMI中断请求或者被重新启动为止。在关机方式下，处理器不响应INTR中断请求。
/// 双重故障通常指示系统表出现严重的问题，例如段描述符表、页表或中断描述符表出现问题。
/// 双重故障处理程序在重建系统表后，可能不得不重新启动操作系统
extern "x86-interrupt" fn double_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:DOUBLE FAULT\n{:#?}", stackframe);
}

/// 除零错误
/// 当执行DIV指令或IDIV指令时，如果除数等于0或者商太大，以至于存放商的操作数容纳不下，
/// 那么就产生这一故障。除法出错故障不提供出错码。
extern "x86-interrupt" fn divide_by_zero(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:divide_by_zero\n{:#?}", stackframe);
}

/// NMI不可屏蔽中断
/// NMI中断可由 CPU 内部产生，也可由外部 NMI 针脚产生
/// 外部不可屏蔽中断请求经由专门的CPU针脚NMI，通知CPU发生了灾难性事件，如电源掉电、总线奇偶位出错等。内部不可屏蔽中断请求是CPU内部自发产生的，如存储器读写出错、溢出中断、除法出错中断等。
/// NMI线上中断请求是不可屏蔽的（既无法禁止的）、而且立即被CPU锁存
extern "x86-interrupt" fn non_maskable_interrupt(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:non_maskable_interrupt\n{:#?}", stackframe);
}

/// 调试
/// 调试异常有故障类型，也有陷阱类型。
/// 调试程序可以访问调试寄存器DR6，以确定调试异常的原因和类型。调试异常不提供出错码
extern "x86-interrupt" fn debug(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:debug\n{:#?}", stackframe);
}

/// 溢出异常
/// INTO指令提供条件陷阱。如果OF标志为1，那么INTO指令产生陷阱；否则不产生陷阱，继续执行INTO后面的指令。
/// 在进入溢出处理程序时，被保存的CS和EIP指向INTO指令的下一条指令。溢出陷阱不提供出错码
extern "x86-interrupt" fn overflow(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:overflow\n{:#?}", stackframe);
}

/// 边界检查错误
/// 如果BOUND指令发现被测试的值超过了指令中给定的范围，那么就发生边界检查故障。边界检查故障不提供出错码
extern "x86-interrupt" fn bound_range_exceeded(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:bound_range_exceeded\n{:#?}", stackframe);
}

/// 无效的操作码
/// (1)操作码字段的内容不是一个合法的指令代码
/// (2)要求使用存储器操作数的场合，使用了寄存器操作数
/// (3)不能被加锁的指令前使用了LOCK前缀。非法操作码故障不提供操作码
extern "x86-interrupt" fn invalid_opcode(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:invalid_opcode\n{:#?}", stackframe);
}

/// 设备不可用
/// 设备不可用故障支持80387数字协处理器。
/// 在没有80387协处理器硬件的系统中，可用该异常的处理程序代替协处理器的软件模拟器。
/// 在发生任务切换时，使得只有在新任务使用浮点指令时，
/// 才进行80387寄存器状态的切换。设备不可用故障不提供出错码。
/// 该故障在下列情况下产生：
/// (1)在执行浮点指令时，控制寄存器CR0中的EM位或TS位为1；
/// (2)在执行WAIT指令时，控制寄存器CR0中TS位及EM位都为1。
/// 需要注意的是，本异常的处理程序必须是一个过程而不能是任务，
/// 否则当处理程序发布一条IRET指令时，80386就设置TS位。
/// 然后协处理器再次执行这个发生故障的指令，发现TS是置位的，
/// 因此就再次发生异常7，结果是无休止的循环。处理程序能通过陷阱门被调用，因为执行期间可以允许中断
extern "x86-interrupt" fn device_not_available(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:device_not_available\n{:#?}", stackframe);
}

/// x87协处理器异常
/// 协处理器出错故障指示协处理器发生了未被屏蔽的数字错误，如上溢或下溢。
/// 在引起故障的浮点指令之后的下一条浮点指令或WAIT指令，
/// 把协处理器出错作为一个故障通知给系统。协处理器出错故障不提供出错码
extern "x86-interrupt" fn x87_floating_point(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:x87_floating_point\n{:#?}", stackframe);
}

/// 机器检查 MCE machine check exceptions
/// 是在硬件不能纠正内部错误的时候触发
/// 当硬件能够纠正内部错误的时候称为silent machine check。
/// 错误来源
///   PCI-E设备信号质量/时钟
//    CPU芯片损坏/设计BUG
//    CPU Cache损坏或其它故障
//    CPU可能的缺陷
//    如CPU生产制造过程中带来的缺陷
//    内存坏/接触不良
//    BIOS配置不当
//    OS/MCE中断程序Bug
//    环境因素，如温度/湿度
extern "x86-interrupt" fn machine_check(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:machine_check\n{:#?}", stackframe);
}

/// SIMD浮点异常
/// 表明操作系统支持unmasked SIMD浮点Exception的执行；
/// SIMD floating-point异常仅仅会由SSE/SSE3/SSE2/SSE4 SIMD浮点指令产生
extern "x86-interrupt" fn simd_floating_point(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:simd_floating_point\n{:#?}", stackframe);
}

/// 虚拟化异常
extern "x86-interrupt" fn virtualization(stackframe: &mut InterruptStackFrame) {
    println!("EXCEPTION:virtualization\n{:#?}", stackframe);
}

/// 无效TSS
/// 当正从任务状态段TSS装入选择子时，如果发生了除了段不存在故障以外的段异常时，就发生无效TSS故障。
/// 在进入故障处理程序时，保存的CS及EIP指向发生故障的指令；
/// 或者该故障作为任务切换的一部分发生时，指向任务的第一条指令。
/// 无效TSS故障提供了一个出错码，出错码的格式如下所示。
/// |BIT15—BIT3|BIT2|BIT1|BIT0|
/// |Selector  |TI 	|IDT |EXT |
/// 其中Selector部分是指向引起故障的TSS的选择子。16位的出错代码的主要成分是选择子，指向引起故障的TSS的选择子。高13位是选择子的索引部分，TI位是描述符表指示位。
/// 一些引起无效TSS故障的原因如下：
///    TSS描述符中的段限长小于103；
///    无效的LDT描述符，或者LDT未出现；
///    堆栈段不是一个可写段；
///    堆栈段选择子索引的描述符超出描述符表界限；
///    堆栈段DPL与新的CPL不匹配；
///    堆栈段选择子的RPL不等于CPL；
///    代码段选择子索引的描述符超出描述符表界限；
///    代码段选择子不指向代码段；
///    非一致代码段的DPL不等于新的CPL；
///    一致代码段的DPL大于新的CPL；
///    对应DS、ES、FS或GS的选择子指向一个不可读段(如系统段)；
///    对应DS、ES、FS或GS的选择子索引的描述符超出描述符表的界限
extern "x86-interrupt" fn invalid_tss(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:invalid_tss\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

/// 段不存在
/// 处理器在把描述符装入非SS段寄存器的高速缓冲时，如果发现描述符其它方面有效，
/// 而P位为0(表示对应段不存在)，那么在引用此描述符时就发生段不存在故障。
/// 有关SS段的情形纳入堆栈段故障。
/// 在进入故障处理程序时，保存的CS及EIP执行发生故障的指令；
/// 或者该故障作为任务切换的一部分发生时，指向任务的第一条指令。
/// 段不存在故障提供了一个包含引起该故障的段选择子的出错代码。
extern "x86-interrupt" fn segment_not_present(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:segment_not_present\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

/// 堆栈段错误
/// 当处理器检测到用SS寄存器进行寻址的段有关的某种问题时，就发生堆栈段故障。
/// 在进入故障处理程序时，保存的CS及EIP指向发生故障的指令；
/// 或者该故障作为任务切换的一部分发生时，指向任务的第一条指令。堆栈段故障提供一个出错码
/// 当出现下列三种情况时，将引起堆栈段故障
/// 在堆栈操作时，偏移超出段界限所规定的范围。这种情况下的出错码是0。例如PUSH操作时，堆栈溢出
/// 在由特权级变换所引起的对内层堆栈的操作时，偏移超出段界限所规定的范围。这种情况下的出错码包含有内层堆栈的选择子
/// 装入到SS寄存器(高速缓冲寄存器)的描述符中的存在位为0。这种情况下的出错码包含有对应的选择子
extern "x86-interrupt" fn stack_segment_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:stack_segment_fault\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

/// 常规保护异常
/// 除了明确列出的段异常外，其它的段异常都被视为通用保护故障。
/// 在进入故障处理程序时，保存的CS及EIP指向发生故障的指令；
/// 或者该故障作为任务切换的一部分发生时，指向任务的第一条指令。
/// 常规保护异常可分为如下两类
/// (1)违反保护方式，但程序无须中止的异常 这类故障提供的出错码为0。这种异常在应用程序执行特权指令或I/O访问时发生，支持虚拟8086程序的系统或支持虚拟I/O访问的系统需要模拟这些指令，并在模拟完成产生故障的指令后，重新执行被中断的程序
/// (2)违反保护方式，并导致程序终止的异常。这类故障提供的出错码可能为0，也可能不为0(能确定选择子时)。
/// 引起这类故障的一些原因如下
///     向某个只读数据段或代码段写；
///     从某个只能执行的代码段读出；
///     将某个系统段描述符装入到数据段寄存器DS、ES、FS、GS或SS；
///     将控制转移到一个不可执行的段；
///     在通过段寄存器CS、DS、ES、FS或GS访问内存时，偏移超出段界限；
///     当访问某个描述符表时，超过描述符表段界限；
///     把PG位为1但PE位为0的控制信息装入到CR0寄存器；
///     切换到一个正忙的任务。
///     对上述两类通用保护故障的辨别，可通过检查引起故障的指令和出错码进行。如果出错码非0，那么肯定是第二类通用保护故障。如果出错码是0，那么需要进一步检查引起故障的指令，以确定它是否是系统支持的可以模拟的指令
extern "x86-interrupt" fn general_protection_fault(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:general_protection_fault\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

/// 对齐检查异常
extern "x86-interrupt" fn alignment_check(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:alignment_check\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}

/// 安全检查异常
extern "x86-interrupt" fn security_exception(stackframe: &mut InterruptStackFrame, _err: u64) {
    println!("EXCEPTION:security_exception\n{:#?}\nCode:{:#?}\n", stackframe, _err);
}
//---------------------------------异常处理--------------------------------------

