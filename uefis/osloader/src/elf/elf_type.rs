use core::fmt;

use crate::elf::GenElf;

// 魔数
pub const MAGIC_BITES: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46];
// Ident 版本
const CURRENT_IDENT_VERSION: u8 = 0x01;
// ELF 版本
const CURRENT_ELF_VERSION: u32 = 0x01;


/// 二进制应用接口类型
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ABI {
    SystemV,
    /// HP-UX 是一种多用户操作系统，允许多个用户同时使用该系统
    HpUx,
    /// NetBSD是一个免费的，具有高度移植性的 UNIX-like 操作系统，是现行可移植平台最多的操作系统
    NetBSD,
    Linux,
    /// GNU Hurd 是一系列基于GNU Mach的守护进程，这一套守护进程最终形成了GNU操作系统
    GunHurd,
    /// 是Sun Microsystems研发的计算机操作系统。它被认为是UNIX操作系统的衍生版本之一。 Solaris属于混合开源软件。
    Solaris,
    /// 是IBM基于AT&T Unix System V开发的一套类UNIX操作系统，运行在IBM专有的Power系列芯片设计的小型机硬件系统之上
    AIX,
    /// IRIX是由硅谷图形公司（Silicon Graphics Inc.，一般用简称：SGI，美国图形工作站生产厂商）以System V与BSD延伸程序为基础所发展成的UNIX操作系统
    IRIX,
    /// FreeBSD是一种类UNIX操作系统，是由经过BSD、386BSD和4.4BSD发展而来的Unix的一个重要分支
    FreeBSD,
    /// tru64 unix
    Tru64Unix,
    NovellModesto,
    /// OpenBSD是一个多平台的，基于4.4BSD的类UNIX操作系统，是BSD衍生出的三种免费操作系统（另外两种是NetBSD和FreeBSD）之一，被称为世界上最安全的操作系统
    OpenBSD,
    /// OpenVMS是基于VMS的多任务多处理器操作系统；当决定将VMS系统移植到Alpha计算机上时，将VMS系统的名称改为了OpenVMS
    OpenVMS,
    /// NonStop系统是专为在线交易处理而设计的可容错，可扩展的分布式计算机系统
    NonStopKernel,
    /// AROS Research操作系统是一种轻量级，高效且灵活的桌面操作系统
    AROS,
    /// FenixOS-专注于高可伸缩性和可靠性的研究操作系统
    FenixOS,
    /// CloudABI是用于类似UNIX的操作系统的运行时环境
    CloudABI,
    /// 未知
    Unknown(u8),
}

/// 二进制应用接口版本
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ABIVersion {
    Unspecified,
    Specified(u8),
}

/// 段类型
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum SectionType {
    /// 将节头标识为无效。此节头没有关联的节。节头的其他成员具有未定义的值
    SHT_NULL,
    /// 标识由程序定义的信息，这些信息的格式和含义完全由程序确定
    SHT_PROGBITS,
    /// 标识符号表。通常，SHT_SYMTAB 节会提供用于链接编辑的符号。
    /// 作为完整的符号表，该表可能包含许多对于动态链接不必要的符号
    /// 因此，目标文件还可以包含一个 SHT_DYNSYM 节，用以包含最低限度的一组动态链接符号，从而节省空间
    SHT_SYMTAB,
    /// 包含一个字符串表。一个对象文件包含多个字符串表，比如.strtab（包含符号的名字）和.shstrtab（包含section的名称）
    SHT_STRTAB,
    /// 标识包含显式加数的重定位项，如 32 位目标文件类的 Elf32_Rela 类型。目标文件可以有多个重定位节。
    SHT_RELA,
    /// 标识符号散列表。动态链接的目标文件必须包含符号散列表
    SHT_HASH,
    /// 标识动态链接的信息。当前，目标文件只能有一个动态节
    SHT_DYNAMIC,
    /// 标识以某种方法标记文件的信息
    SHT_NOTE,
    /// 标识在文件中不占用任何空间，但在其他方面与 SHT_PROGBITS 类似的节。
    /// 虽然此节不包含任何字节，但 sh_offset 成员包含概念性文件偏移
    SHT_NOBITS,
    /// 标识不包含显式加数的重定位项，如 32 位目标文件类的 Elf32_Rel 类型。目标文件可以有多个重定位节。
    SHT_REL,
    /// 标识具有未指定的语义的保留节。包含此类型的节的程序不符合 ABI。
    SHT_SHLIB,
    /// 非全局符号的动态符号表
    /// SHT_DYNSYM 还可以使用 SHT_SUNW_LDYNSYM 节进行扩充。
    /// 此附加节为运行时环境提供局部函数符号，但对于动态链接来说不是必需的
    /// 如果 SHT_SUNW_LDYNSYM 节和 SHT_DYNSYM 节同时存在，链接编辑器会将这两者的数据区域紧邻彼此放置。
    /// SHT_SUNW_LDYNSYM 节位于 SHT_DYNSYM 节的前面。这种放置方式可以使两个表看起来像是一个更大的连续符号表，其中包含 SHT_SYMTAB 中的缩减符号集合
    SHT_DYNSYM,
    // 标识包含指针数组的节，这些指针指向初始化函数。数组中的每个指针都被视为不返回任何值的无参数过程。,
    SHT_INIT_ARRAY,
    // 标识包含指针数组的节，这些指针指向析构函数。数组中的每个指针都被视为不返回任何值的无参数过程,
    SHT_FINI_ARRAY,
    // 标识包含指针数组的节，这些指针指向在其他所有初始化函数之前调用的函数。数组中的每个指针都被视为不返回任何值的无参数过程
    SHT_PREINIT_ARRAY,
    // 标识节组。节组标识一组相关的节，这些节必须作为一个单元由链接编辑器进行处理。SHT_GROUP 类型的节只能出现在可重定位目标文件中
    SHT_GROUP,
    // 标识包含扩展节索引的节，扩展节索引与符号表关联。如果符号表引用的任何节头索引包含转义值 SHN_XINDEX，则需要关联的 SHT_SYMTAB_SHNDX。
    SHT_SYMTAB_SHNDX,
    /// 此范围内包含的值（包括这两个值）保留用于特定于操作系统的语义
    SHT_LOOS,
    /// 此范围内包含的值（包括这两个值）保留用于特定于操作系统的语义
    SHT_HIOS,
    /// 标识特定于 x64 的数据，其中包含对应于栈展开的展开函数表的各项
    SHT_AMD64_UNWIND,
    /// 下界 为特定于处理器的语义保留 0x70000000
    SHT_LOPROC,
    /// 上界 为特定于处理器的语义保留 0x7fffffff
    SHT_HIPROC,
    /// 指定了为应用程序保留的索引的下界，这个范围内的索引可以被应用程序使用 0x80000000
    SHT_LOUSER,
    /// 指定了为应用程序保留的索引的上界，这个范围内的索引可以被应用程序使用 0xffffffff
    SHT_HIUSER,
    /// 未知
    Unknown(u32),
}

impl From<u32> for ElfHeaderFlags {
    fn from(n: u32) -> Self {
        match n {
            0x0 => ElfHeaderFlags::X86,
            0xffff00 => ElfHeaderFlags::EF_SPARC_EXT_MASK,
            0x000100 => ElfHeaderFlags::EF_SPARC_32PLUS,
            0x000200 => ElfHeaderFlags::EF_SPARC_SUN_US1,
            0x000400 => ElfHeaderFlags::EF_SPARC_HAL_R1,
            0x000800 => ElfHeaderFlags::EF_SPARC_SUN_US3,
            0x3 => ElfHeaderFlags::EF_SPARCV9_MM,
//            0x0 => ElfHeaderFlags::EF_SPARCV9_TSO,
            0x1 => ElfHeaderFlags::EF_SPARCV9_PSO,
            0x2 => ElfHeaderFlags::EF_SPARCV9_RMO,
            n => ElfHeaderFlags::Unknown(n),
        }
    }
}

/// 与文件关联的特定于处理器的标志。
/// 标志名称采用EF_machine_flag形式。
/// 对于 x86，此成员目前为零。
/// SPARC标志
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ElfHeaderFlags {
    X86,
    /// 供应商扩展掩码
    EF_SPARC_EXT_MASK,
    /// 通用V8+功能
    EF_SPARC_32PLUS,
    /// SunUltraSPARC1扩展
    EF_SPARC_SUN_US1,
    /// HALR1扩展
    EF_SPARC_HAL_R1,
    /// SunUltraSPARC3扩展
    EF_SPARC_SUN_US3,
    /// 内存型号掩码
    EF_SPARCV9_MM,
    /// 总体存储排序
//    EF_SPARCV9_TSO,
    /// 部分存储排序
    EF_SPARCV9_PSO,
    /// 非严格内存排序
    EF_SPARCV9_RMO,
    Unknown(u32),
}


/// ELF文件类型
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
pub enum ElfType {
    /// 无类型
    None,
    ///  可重定位文件，通常是文件名以.o结尾，目标文件
    Rel,
    /// 可执行文件
    Exec,
    /// 动态库文件
    Dyn,
    /// core文件
    Core,
    /// 表示已经定义了5种文件类型
    Loos,
    Hios,
    /// 0xff00
    Loproc,
    /// 0xffff
    Hiproc,
    Unknown(u16),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
pub enum ElfMachine {
    /// 未知
    Unknown,
    /// 可扩充处理器结构（SPARC，Scalable Processor Architecture）是RISC微处理器架构之一
    Sparc,
    ///  x86泛指一系列基于Intel 8086且向后兼容的中央处理器指令集架构。最早的8086处理器于1978年由Intel推出，为16位微处理器
    X86,
    /// MIPS架构是一种采取精简指令集（RISC）的处理器架构
    Mips,
    /// 是一种精简指令集（RISC）架构的中央处理器（CPU）
    PowerPc,
    S390,
    /// 是一个32位精简指令集（RISC）处理器架构
    Arm,
    ///  SuperH 是以 32 位元存取的精简指令集架构，多用在嵌入式系统
    SuperH,
    /// IA64，又称英特尔安腾架构（Intel Itanium architecture），使用在Itanium处理器家族上的64位指令集架构
    Ia64,
    /// x86-64（ 又称x64，即英文词64-bit extended，64位拓展 的简写）是x86架构的64位拓展，向后兼容于16位及32位的x86架构
    X86Ex,
    /// ARMv8架构 ARMv8架构包含两个执行状态：AArch64和AArch32
    AArch64,
    /// 是一个基于精简指令集（RISC）原则的开源指令集架构（ISA）
    RiscV,
    MUnknown(u16),
}

/// ELF所使用的类型
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ElfKind {
    /// elf 32位
    Elf32,
    /// elf 64位
    Elf64,
    Unknown(u8),
}

/// ELF所使用的字节序
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ElfEndian {
    /// 字节序 小端
    LittleEndian,
    /// 字节序 大端
    BigEndian,
    Unknown(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ProgramType {
    /// 未使用。没有定义成员值。使用此类型，程序头表可以包含忽略的项
    NULL,
    /// 指定由p_filesz和p_memsz描述的可装入段。
    /// 文件中的字节会映射到内存段的起始位置。
    /// 如果段的内存大小(p_memsz)大于文件大小(p_filesz)，则将多余字节的值定义为0。
    /// 这些字节跟在段的已初始化区域后面。文件大小不能大于内存大小。
    /// 程序头表中的可装入段的各项按升序显示，并基于p_vaddr成员进行排列。
    LOAD,
    /// 指定动态链接信息
    DYNAMIC,
    // 指定要作为解释程序调用的以空字符结尾的路径名的位置和大小。
    // 对于动态可执行文件，必须设置此类型。此类型可出现在共享目标文件中。
    // 此类型不能在一个文件中多次出现。此类型（如果存在）必须位于任何可装入段的各项的前面。有关详细信息，
    INTERP,
    /// 指定辅助信息的位置和大小。
    NOTE,
    /// 保留类型，但具有未指定的语义
    SHLIB,
    // 指定程序头表在文件及程序内存映像中的位置和大小。
    // 此段类型不能在一个文件中多次出现。此外，仅当程序头表是程序内存映像的一部分时，才可以出现此段。
    // 此类型（如果存在）必须位于任何可装入段的各项的前面
    PHDR,
    // 此范围内包含的值保留用于特定于操作系统的语义。 0x60000000
    LOOS,
    // 此范围内包含的值保留用于特定于操作系统的语义。 0x6FFFFFFF
    HIOS,
    // 此范围内包含的值（包括这两个值）保留用于特定于处理器的语义。 0x70000000
    LOPROC,
    // 此范围内包含的值（包括这两个值）保留用于特定于处理器的语义。 0x7FFFFFFF
    HIPROC,
    // 0x6474e551
    GnuStack,
    Unknown(u32),
}


/// ELF魔数
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Magic {
    /// ELF有效魔数
    Valid,
    /// ELF无效魔数
    Invalid,
}

/// 标识符版本
#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum IdentVersion {
    /// 只有一个版本
    Current,
    Unknown(u8),
}

/// ELF版本
#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum ElfVersion {
    /// 只有一个版本
    Current,
    Unknown(u32),
}


#[allow(unused_variables, non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProgramFlags {
    /* Segment is executable = (1 << 0)*/
    PF_X,
    /* Segment is writable = (1 << 1)*/
    PF_W,
    /* Segment is readable = (1 << 2) */
    PF_R,
    /* Segment is readable = (1 << 2) | (1 << 1)* */
    PF_RW,
    /* OS-specific = 0x0ff00000*/
    PF_MASKOS,
    /* Processor-specific = 0xf0000000*/
    PF_MASKPROC,
    Unknown(u32),
}

impl From<u32> for ProgramFlags {
    fn from(n: u32) -> Self {
        match n {
            0b00 => ProgramFlags::PF_X,
            0b10 => ProgramFlags::PF_W,
            0b100 => ProgramFlags::PF_R,
            0b110 => ProgramFlags::PF_RW,
            0x0ff00000 => ProgramFlags::PF_MASKOS,
            0xf0000000 => ProgramFlags::PF_MASKPROC,
            n => ProgramFlags::Unknown(n)
        }
    }
}


///////////////////////////////////////
//// From Trait Implement
//////////////////////////////////////

impl From<u32> for ElfVersion {
    fn from(n: u32) -> Self {
        match n {
            CURRENT_ELF_VERSION => ElfVersion::Current,
            n => ElfVersion::Unknown(n)
        }
    }
}

impl From<u8> for ABIVersion {
    fn from(n: u8) -> Self {
        match n {
            0x00 => ABIVersion::Unspecified,
            n => ABIVersion::Specified(n)
        }
    }
}

impl From<u8> for IdentVersion {
    fn from(n: u8) -> Self {
        match n {
            CURRENT_IDENT_VERSION => IdentVersion::Current,
            n => IdentVersion::Unknown(n)
        }
    }
}

impl From<u32> for ProgramType {
    fn from(n: u32) -> Self {
        match n {
            0x00000000 => ProgramType::NULL,
            0x00000001 => ProgramType::LOAD,
            0x00000002 => ProgramType::DYNAMIC,
            0x00000003 => ProgramType::INTERP,
            0x00000004 => ProgramType::NOTE,
            0x00000005 => ProgramType::SHLIB,
            0x00000006 => ProgramType::PHDR,
            0x60000000 => ProgramType::LOOS,
            0x6FFFFFFF => ProgramType::HIOS,
            0x70000000 => ProgramType::LOPROC,
            0x7FFFFFFF => ProgramType::HIPROC,
            0x6474e551 => ProgramType::GnuStack,
            n => ProgramType::Unknown(n)
        }
    }
}

impl From<u8> for ElfEndian {
    fn from(n: u8) -> Self {
        match n {
            1 => ElfEndian::LittleEndian,
            2 => ElfEndian::BigEndian,
            n => ElfEndian::Unknown(n)
        }
    }
}

impl From<u8> for ElfKind {
    fn from(n: u8) -> Self {
        match n {
            1 => ElfKind::Elf32,
            2 => ElfKind::Elf64,
            n => ElfKind::Unknown(n)
        }
    }
}

impl From<u16> for ElfMachine {
    fn from(n: u16) -> Self {
        match n {
            0x00 => ElfMachine::Unknown,
            0x02 => ElfMachine::Sparc,
            0x03 => ElfMachine::X86,
            0x08 => ElfMachine::Mips,
            0x14 => ElfMachine::PowerPc,
            0x16 => ElfMachine::S390,
            0x28 => ElfMachine::Arm,
            0x2A => ElfMachine::SuperH,
            0x32 => ElfMachine::Ia64,
            0x3E => ElfMachine::X86Ex,
            0xB7 => ElfMachine::AArch64,
            0xF3 => ElfMachine::RiscV,
            n => ElfMachine::MUnknown(n)
        }
    }
}

impl From<u16> for ElfType {
    fn from(n: u16) -> Self {
        match n {
            0x00 => ElfType::None,
            0x01 => ElfType::Rel,
            0x02 => ElfType::Exec,
            0x03 => ElfType::Dyn,
            0x04 => ElfType::Core,
            0xfe00 => ElfType::Loos,
            0xfeff => ElfType::Hios,
            0xff00 => ElfType::Loproc,
            0xffff => ElfType::Hiproc,
            n => ElfType::Unknown(n)
        }
    }
}

impl From<u32> for SectionType {
    fn from(n: u32) -> Self {
        match n {
            0x0 => SectionType::SHT_NULL,
            0x1 => SectionType::SHT_PROGBITS,
            0x2 => SectionType::SHT_SYMTAB,
            0x3 => SectionType::SHT_STRTAB,
            0x4 => SectionType::SHT_RELA,
            0x5 => SectionType::SHT_HASH,
            0x6 => SectionType::SHT_DYNAMIC,
            0x7 => SectionType::SHT_NOTE,
            0x8 => SectionType::SHT_NOBITS,
            0x9 => SectionType::SHT_REL,
            0x0A => SectionType::SHT_SHLIB,
            0x0B => SectionType::SHT_DYNSYM,
            0x0E => SectionType::SHT_INIT_ARRAY,
            0x0F => SectionType::SHT_FINI_ARRAY,
            0x10 => SectionType::SHT_PREINIT_ARRAY,
            0x11 => SectionType::SHT_GROUP,
            0x12 => SectionType::SHT_SYMTAB_SHNDX,
            0x60000000 => SectionType::SHT_LOOS,
            0x6fffffff => SectionType::SHT_HIOS,
            0x70000000 => SectionType::SHT_LOPROC,
            0x70000001 => SectionType::SHT_AMD64_UNWIND,
            0x7fffffff => SectionType::SHT_HIPROC,
            0x80000000 => SectionType::SHT_LOUSER,
            0xffffffff => SectionType::SHT_HIUSER,
            n => SectionType::Unknown(n),
        }
    }
}

impl From<u8> for ABI {
    fn from(n: u8) -> Self {
        match n {
            0x00 => ABI::SystemV,
            0x01 => ABI::HpUx,
            0x02 => ABI::NetBSD,
            0x03 => ABI::Linux,
            0x04 => ABI::GunHurd,
            0x06 => ABI::Solaris,
            0x07 => ABI::AIX,
            0x08 => ABI::IRIX,
            0x09 => ABI::FreeBSD,
            0x0A => ABI::Tru64Unix,
            0x0B => ABI::NovellModesto,
            0x0C => ABI::OpenBSD,
            0x0D => ABI::OpenVMS,
            0x0E => ABI::NonStopKernel,
            0x0F => ABI::AROS,
            0x10 => ABI::FenixOS,
            0x11 => ABI::CloudABI,
            n => ABI::Unknown(n)
        }
    }
}
