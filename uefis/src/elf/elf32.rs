use core::fmt;
use core::result::Result;

use crate::elf::{ABIVersion, ElfHeaderFlags, ElfVersion, GenElf, GenElfHeader, GenProgramHeader, GenSectionHeader, ProgramFlags};
use crate::elf::elf_type::{ABI, ElfEndian, ElfKind, ElfMachine, ElfType, ProgramType, SectionType};
use crate::elf::flags::SectionHeaderFlags;

pub struct Elf32<'a>(&'a [u8]);

impl<'a> Elf32<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
}

impl<'a> GenElf for Elf32<'a> {
    type Word = u32;
    type ElfHeaderType = ElfHeader32;
    type ProgramHeaderType = ProgramHeader32;
    type SectionHeaderType = SectionHeader32;

    fn as_bytes(&self) -> &[u8] { self.0 }
}

impl<'a> fmt::Debug for Elf32<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Elf32 File")
            .field("Memory Location", &self.0.as_ptr())
            .finish()
    }
}

#[repr(C)]
pub struct ElfHeader32 {
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

impl GenElfHeader for ElfHeader32 {
    type Word = u32;

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

impl fmt::Debug for ElfHeader32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Elf32")
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
pub struct ProgramHeader32 {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: <Self as GenProgramHeader>::Word,
    pub p_vaddr: <Self as GenProgramHeader>::Word,
    pub p_paddr: <Self as GenProgramHeader>::Word,
    pub p_filesz: <Self as GenProgramHeader>::Word,
    pub p_memsz: <Self as GenProgramHeader>::Word,
    pub p_align: <Self as GenProgramHeader>::Word,
}

impl GenProgramHeader for ProgramHeader32 {
    type Word = u32;

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

#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader32 {
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

impl GenSectionHeader for SectionHeader32 {
    type Word = u32;

    fn name_off(&self) -> u32 {
        self.sh_name
    }

    fn sh_type(&self) -> SectionType {
        self.sh_type.into()
    }

    fn flags(&self) -> SectionHeaderFlags {
        SectionHeaderFlags::from_bits_truncate(self.sh_flags as u64)
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
