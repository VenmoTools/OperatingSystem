use volatile::Volatile;

/// 进程的状态
#[derive(Copy, Clone)]
pub enum State {
    Crate,
    Ready,
    Running,
    Blocking,
    Stop,
}

/// 进程标志
#[repr(u64)]
pub enum Flags {
    KernelThread,
    Thread,
    Process,
}

/// 用于保存进程信息
struct ProcessInfo {
    /// 内核栈基址
    rsp0: i64,
    /// 内核代码指针
    rip: i64,
    /// 内核当前栈指针
    rsp: i64,
    /// FS段寄存器
    fs: i64,
    /// GS段寄存器
    gs: i64,
    /// CR2控制寄存器
    cr2: i64,
    /// 产生异常的异常号
    trap_no: i64,
    /// 异常的错误码
    err_code: i64,
}

/// 进程主体
#[repr(C)]
pub struct Process {
    /// 进程状态
    state: Volatile<State>,
    /// 进程运行状态信息
    info: ProcessInfo,
    /// 进程标志信息，内核线程，进程，线程
    flags: Flags,
    /// 进程号
    pid: i64,
    /// 进程可用时间片
    counter: i64,
    /// 进程优先级
    priority: i64,
    /// 进程地址空间范围
    /// 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF 应用层
    /// 0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF 内核层
    address_limit: i64,

}