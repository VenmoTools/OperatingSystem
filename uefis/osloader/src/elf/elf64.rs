use core::fmt;

use crate::elf::{ABIVersion, ElfHeaderFlags, ElfVersion, GenElf, GenElfHeader, GenProgramHeader, GenSectionHeader};
use crate::elf::elf_type::{ABI, ElfEndian, ElfKind, ElfMachine, ElfType, ProgramFlags, ProgramType, SectionType};
use crate::elf::flags::SectionHeaderFlags;

pub struct Elf64<'a>(&'a [u8]);

impl<'a> Elf64<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
}

impl<'a> GenElf for Elf64<'a> {
    type Word = u64;
    type ElfHeaderType = ElfHeader64;
    type ProgramHeaderType = ProgramHeader64;
    type SectionHeaderType = SectionHeader64;

    fn as_bytes(&self) -> &[u8] { self.0 }
}

impl<'a> fmt::Debug for Elf64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Elf64 File")
            .field("Memory Location", &self.0.as_ptr())
            .finish()
    }
}

#[repr(C)]
pub struct ElfHeader64 {
    pub magic: [u8; 4],
    pub class: u8,
    pub endianness: u8,
    pub header_version: u8,
    pub abi: u8,
    pub abi_version: u8,
    pub unused: [u8; 7],
    pub elftype: u16,
    pub machine: u16,
    pub elf_version: u32,
    pub entry: <Self as GenElfHeader>::Word,
    pub phoff: <Self as GenElfHeader>::Word,
    pub shoff: <Self as GenElfHeader>::Word,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

impl GenElfHeader for ElfHeader64 {
    type Word = u64;

    fn class(&self) -> ElfKind {
        self.class.into()
    }

    fn endianness(&self) -> ElfEndian {
        self.endianness.into()
    }

    fn header_version(&self) -> u8 {
        self.header_version
    }

    fn abi(&self) -> ABI {
        self.abi.into()
    }

    fn abi_version(&self) -> ABIVersion {
        self.abi_version.into()
    }

    fn elftype(&self) -> ElfType {
        self.elftype.into()
    }

    fn machine(&self) -> ElfMachine {
        self.machine.into()
    }

    fn elf_version(&self) -> ElfVersion {
        self.elf_version.into()
    }

    fn entry_point(&self) -> Self::Word {
        self.entry
    }

    fn program_header_offset(&self) -> Self::Word {
        self.phoff
    }

    fn section_header_offset(&self) -> Self::Word {
        self.shoff
    }

    fn flags(&self) -> ElfHeaderFlags {
        self.flags.into()
    }

    fn elf_header_size(&self) -> u16 {
        self.ehsize
    }

    fn program_header_entry_size(&self) -> u16 {
        self.phentsize
    }

    fn program_header_entry_num(&self) -> u16 {
        self.phnum
    }

    fn section_header_entry_size(&self) -> u16 {
        self.shentsize
    }

    fn section_header_entry_num(&self) -> u16 {
        self.shnum
    }

    fn shstr_index(&self) -> u16 {
        self.shstrndx
    }
}

impl fmt::Debug for ElfHeader64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Elf64")
            .field("Class", &self.class())
            .field("Endianness", &self.endianness())
            .field("ELF Header Version", &self.header_version())
            .field("ABI", &self.abi())
            .field("ABI Version", &self.abi_version())
            .field("File Type", &self.elftype())
            .field("Target Machine", &self.machine())
            .field("ELF Version", &self.elf_version())
            .field("Entry Point", &self.entry_point())
            .field("Program Header Offset", &self.program_header_offset())
            .field("Section Header Offset", &self.section_header_offset())
            .field("Flags", &self.flags())
            .field("ELF Header Size", &self.elf_header_size())
            .field("Program Header Size", &self.program_header_entry_size())
            .field("Program Header Number", &self.program_header_entry_num())
            .field("Section Header Size", &self.section_header_entry_size())
            .field("Section Header Number", &self.section_header_entry_num())
            .field(".shstr Section Index", &self.shstr_index())
            .finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ProgramHeader64 {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: <Self as GenProgramHeader>::Word,
    pub p_vaddr: <Self as GenProgramHeader>::Word,
    pub p_paddr: <Self as GenProgramHeader>::Word,
    pub p_filesz: <Self as GenProgramHeader>::Word,
    pub p_memsz: <Self as GenProgramHeader>::Word,
    pub p_align: <Self as GenProgramHeader>::Word,
}

impl GenProgramHeader for ProgramHeader64 {
    type Word = u64;

    fn ph_type(&self) -> ProgramType {
        self.p_type.into()
    }

    fn flags(&self) -> ProgramFlags {
        self.p_flags.into()
    }

    fn offset(&self) -> Self::Word {
        self.p_offset
    }

    fn vaddr(&self) -> Self::Word {
        self.p_vaddr
    }

    fn paddr(&self) -> Self::Word {
        self.p_paddr
    }

    fn filesz(&self) -> Self::Word {
        self.p_filesz
    }

    fn memsz(&self) -> Self::Word {
        self.p_memsz
    }

    fn align(&self) -> Self::Word {
        self.p_align
    }
}

///
/// sh_type                     sh_link                    sh_info
/// SHT_DYNAMIC                 关联的字符串表的节头索引。     0
/// SHT_HASH                    关联的符号表的节头索引。      0
/// SHT_REL | SHT_RELA          关联的符号表的节头索引。      如果 sh_flags 成员包含 SHF_INFO_LINK 标志，则为应用重定位的节的节头索引，否则为 0
/// SHT_SYMTAB | SHT_DYNSYM     关联的字符串表的节头索引。    比上一个局部符号 STB_LOCAL 的符号表索引大一
/// SHT_GROUP                   关联的符号表的节头索引。      关联的符号表中项的符号表索引。指定的符号表项的名称用于提供节组的签名。
/// SHT_SYMTAB_SHNDX            关联的符号表的节头索引。       0
/// https://docs.oracle.com/cd/E38902_01/html/E38861/chapter6-94076.html#scrolltoc
#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader64 {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: <Self as GenSectionHeader>::Word,
    pub sh_addr: <Self as GenSectionHeader>::Word,
    pub sh_offset: <Self as GenSectionHeader>::Word,
    pub sh_size: <Self as GenSectionHeader>::Word,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: <Self as GenSectionHeader>::Word,
    pub sh_entsize: <Self as GenSectionHeader>::Word,
}

impl GenSectionHeader for SectionHeader64 {
    type Word = u64;

    fn name_off(&self) -> u32 {
        self.sh_name
    }

    fn sh_type(&self) -> SectionType {
        self.sh_type.into()
    }

    fn flags(&self) -> SectionHeaderFlags {
        SectionHeaderFlags::from_bits_truncate(self.sh_flags)
    }

    fn addr(&self) -> Self::Word {
        self.sh_addr
    }

    fn offset(&self) -> Self::Word {
        self.sh_offset
    }

    fn size(&self) -> Self::Word {
        self.sh_size
    }

    fn link(&self) -> u32 {
        self.sh_link
    }

    fn info(&self) -> u32 {
        self.sh_info
    }

    fn addralign(&self) -> Self::Word {
        self.sh_addralign
    }

    fn entsize(&self) -> Self::Word {
        self.sh_entsize
    }
}
