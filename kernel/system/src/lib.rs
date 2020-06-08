#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(ptr_internals)]
#![feature(llvm_asm)]
#![feature(allocator_api)]
#![allow(unused_doc_comments)]
#[macro_use]
extern crate alloc;


pub use mutex::Mutex;
#[cfg(feature = "mutiboot")]
use multiboot2::{BootInformation, MemoryAreaType, ElfSection, ElfSectionType, ElfSectionFlags};
use crate::ia_32e::paging::{MemorySpace, MemoryArea};
#[allow(unused_imports)]
use crate::ia_32e::paging::MemoryType;

#[cfg(feature = "efi")]
use uefi::table::boot::{MemoryMapIter, MemoryType as UefiMem};
#[cfg(feature = "efi")]
use xmas_elf::{
    ElfFile, P64,
    sections::{SectionHeader, ShType},
};
use crate::bits::KernelSectionFlags;
use alloc::string::String;
#[cfg(feature = "efi")]
use alloc::string::ToString;
use alloc::vec::Vec;

pub mod bits;
mod mutex;
pub mod ia_32e;
pub mod result;
pub mod devices;
pub mod macros;
pub mod syscall;
#[macro_use]
pub mod console;
pub mod buddy_system_allocator;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KernelArgs {
    pub st: u64,
    pub iter: u64,
    pub kernel_elf: u64,
    pub kernel_start: u64,
    pub kernel_end: u64,
    pub stack_start: u64,
    pub stack_end: u64,
    pub frame_ptr: *mut u8,
    pub frame_size: usize,
}

pub struct KernelArea {
    pub start_addr: u64,
    pub end_addr: u64,
    pub size: u64,
    pub name: String,
    pub align_by: u64,
    pub flags: KernelSectionFlags,
    pub section_ty: KernelSectionType,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u32)]
pub enum KernelSectionType {
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have undefined
    /// values.
    Unused = 0,

    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramSection = 1,

    /// This section holds a linker symbol table.
    LinkerSymbolTable = 2,

    /// The section holds a string table.
    StringTable = 3,

    /// The section holds relocation entries with explicit addends, such as type
    /// Elf32_Rela for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelaRelocation = 4,

    /// The section holds a symbol hash table.
    SymbolHashTable = 5,

    /// The section holds dynamic linking tables.
    DynamicLinkingTable = 6,

    /// This section holds information that marks the file in some way.
    Note = 7,

    /// A section of this type occupies no space in the file but otherwise resembles
    /// `ProgramSection`. Although this section contains no bytes, the
    /// sh_offset member contains the conceptual file offset.
    Uninitialized = 8,

    /// The section holds relocation entries without explicit addends, such as type
    /// Elf32_Rel for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelRelocation = 9,

    /// This section type is reserved but has unspecified semantics.
    Reserved = 10,

    /// This section holds a dynamic loader symbol table.
    DynamicLoaderSymbolTable = 11,

    /// Values in this inclusive range (`[0x6000_0000, 0x6FFF_FFFF)`) are
    /// reserved for environment-specific semantics.
    EnvironmentSpecific = 0x6000_0000,

    /// Values in this inclusive range (`[0x7000_0000, 0x7FFF_FFFF)`) are
    /// reserved for processor-specific semantics.
    ProcessorSpecific = 0x7000_0000,
}


impl KernelArea {
    #[cfg(feature = "efi")]
    pub fn from_uefi(section: &SectionHeader) -> Self {
        let flag = KernelSectionFlags::from_bits(section.flags()).expect("invalid flags bit");
        let ty = match section.get_type().expect("not section type found!") {
            ShType::Null => KernelSectionType::Unused,
            ShType::ProgBits => KernelSectionType::ProgramSection,
            ShType::SymTab => KernelSectionType::LinkerSymbolTable,
            ShType::StrTab => KernelSectionType::StringTable,
            ShType::Rela => KernelSectionType::RelaRelocation,
            ShType::Hash => KernelSectionType::SymbolHashTable,
            ShType::Dynamic => KernelSectionType::DynamicLinkingTable,
            ShType::Note => KernelSectionType::Note,
            ShType::NoBits => KernelSectionType::Uninitialized,
            ShType::Rel => KernelSectionType::RelRelocation,
            ShType::DynSym => KernelSectionType::DynamicLoaderSymbolTable,
            ShType::ProcessorSpecific(_) => KernelSectionType::ProcessorSpecific,
            _ => panic!("kernel no support this section yet")
        };
        Self {
            start_addr: section.address(),
            end_addr: section.address() + section.size(),
            size: section.size(),
            name: section.name().to_string(),
            align_by: 0,
            flags: flag,
            section_ty: ty,
        }
    }

    #[cfg(feature = "mutiboot")]
    pub fn from_mutiboot(section: &ElfSection) -> Self {
        let flag = KernelSectionFlags::from_bits(section.flags().bits()).expect("invalid flags bit");

        let ty = match section.section_type() {
            ElfSectionType::ProgramSection => KernelSectionType::ProgramSection,
            ElfSectionType::Reserved => KernelSectionType::Reserved,
            ElfSectionType::StringTable => KernelSectionType::StringTable,
            ElfSectionType::Unused => KernelSectionType::Unused,
            ElfSectionType::Uninitialized => KernelSectionType::Uninitialized,
            ElfSectionType::LinkerSymbolTable => KernelSectionType::LinkerSymbolTable,
            ElfSectionType::RelaRelocation => KernelSectionType::RelaRelocation,
            ElfSectionType::SymbolHashTable => KernelSectionType::SymbolHashTable,
            ElfSectionType::DynamicLinkingTable => KernelSectionType::DynamicLinkingTable,
            ElfSectionType::Note => KernelSectionType::Note,
            ElfSectionType::RelRelocation => KernelSectionType::RelRelocation,
            ElfSectionType::DynamicLoaderSymbolTable => KernelSectionType::DynamicLoaderSymbolTable,
            ElfSectionType::EnvironmentSpecific => KernelSectionType::EnvironmentSpecific,
            ElfSectionType::ProcessorSpecific => KernelSectionType::ProcessorSpecific,
        };
        Self {
            start_addr: section.start_address(),
            end_addr: section.end_address(),
            size: section.size(),
            name: section.name().to_string(),
            align_by: section.addralign(),
            flags: flag,
            section_ty: ty,
        }
    }
}

pub struct SystemInformation {
    #[cfg(feature = "mutiboot")]
    info: BootInformation,
    #[cfg(feature = "efi")]
    efi: KernelArgs,
    mem_area: MemorySpace,
    kernel_area: Vec<KernelArea>,
    kernel_start: u64,
    kernel_end: u64,
}

impl SystemInformation {
    pub fn kernel_start(&self) -> u64 {
        self.kernel_start
    }
    pub fn kernel_end(&self) -> u64 {
        self.kernel_end
    }

    pub fn mem_area_iter(&self) -> impl Iterator<Item=&MemoryArea> + '_ {
        self.mem_area.iter()
    }
    pub fn kernel_area_iter(&self) -> impl Iterator<Item=&KernelArea> + '_ {
        self.kernel_area.iter()
    }

    pub fn new(k_args: usize) -> Self {
        let res = if cfg!(mutiboot) {
            let args = unsafe { multiboot2::load(k_args) };
            let k_start = args.elf_sections_tag().unwrap().sections().map(|s| s.start_address()).min().unwrap();
            let k_end = args.elf_sections_tag().unwrap().sections().map(|s| s.start_address()).max().unwrap();
            #[allow(unused_mut)]
                let mut s = Self {
                #[cfg(feature = "mutiboot")]
                info,
                mem_area: MemorySpace::new(),
                kernel_area: Vec::new(),
                kernel_start: k_start,
                kernel_end: k_end,
            };
            #[cfg(feature = "mutiboot")]
                s.load_kernel_area();
            #[cfg(feature = "mutiboot")]
                s.load_memory_area();
            s
        } else {
            let args = unsafe { &*((k_args) as *const KernelArgs) };
            let k_start = args.kernel_start;
            let k_end = args.kernel_end;
            #[allow(unused_mut)]
                let mut s = Self {
                #[cfg(feature = "efi")]
                efi: args.clone(),
                mem_area: MemorySpace::new(),
                kernel_area: Vec::new(),
                kernel_start: k_start,
                kernel_end: k_end,
            };
            #[cfg(feature = "efi")]
                s.load_kernel_area();
            #[cfg(feature = "efi")]
                s.load_memory_area();
            s
        };
        res
    }

    #[cfg(feature = "mutiboot")]
    fn load_kernel_area(&mut self) {
        for area in self.info.elf_sections_tag().expect("no elf tag found").sections() {
            self.kernel_area.push(KernelArea::from_mutiboot(&area))
        }
    }

    #[cfg(feature = "efi")]
    fn load_kernel_area(&mut self) {
        let sections = get_elf_section(&self.efi);
        for area in sections.section_iter() {
            self.kernel_area.push(KernelArea::from_uefi(&area))
        }
    }
    #[cfg(feature = "mutiboot")]
    fn load_memory_area(&mut self) {
        for area in self.info.memory_map_tag().expect("no mmp tag").memory_areas() {
            let ty = match area.typ() {
                MemoryAreaType::AcpiAvailable => MemoryType::ACPIArea,
                MemoryAreaType::Reserved => MemoryType::ReservedArea,
                MemoryAreaType::Available => MemoryType::FreeArea,
                MemoryAreaType::ReservedHibernate => MemoryType::ReservedHibernate,
                MemoryAreaType::Defective => MemoryType::Defective,
            };
            self.mem_area.add_area(area.start_address(), area.end_address(), ty, area.size())
        }
    }
    #[cfg(feature = "efi")]
    fn load_memory_areas(&mut self) {
        let mem_iter = get_mem_iter(&self.efi);
        for area in mem_iter {
            let ty = match area.ty {
                UefiMem::CONVENTIONAL => MemoryType::FreeArea,
                UefiMem::MMIO => MemoryType::MMIO,
                UefiMem::RUNTIME_SERVICES_DATA => MemoryType::UefiRunTimeData,
                UefiMem::RUNTIME_SERVICES_CODE => MemoryType::UefiRunTimeCode,
                UefiMem::ACPI_RECLAIM => MemoryType::ACPIArea,
                UefiMem::ACPI_NON_VOLATILE => MemoryType::ACPIReservedArea,
                UefiMem::RESERVED => MemoryType::ReservedArea,
                UefiMem::MMIO_PORT_SPACE => MemoryType::MMIOPortArea,
                UefiMem::UNUSABLE => MemoryType::ErrorArea,
                UefiMem::LOADER_CODE => MemoryType::UsedArea,
                UefiMem::LOADER_DATA => MemoryType::UsedArea,
                UefiMem::BOOT_SERVICES_CODE => MemoryType::FreeArea,
                UefiMem::BOOT_SERVICES_DATA => MemoryType::FreeArea,
                UefiMem::PERSISTENT_MEMORY => MemoryType::ErrorArea,
            };
            // in uefi memory align by 4096
            let mem_area_size = area.page_count * 0x1000;
            self.mem_area.add_area(area.phys_start, area.phys_start + mem_area_size, ty, mem_area_size);
        }
    }
}

#[cfg(feature = "efi")]
fn get_mem_iter(args: &KernelArgs) -> &mut MemoryMapIter {
    unsafe { &mut *(args.iter as *mut MemoryMapIter) }
}

#[cfg(feature = "efi")]
fn get_elf_section(args: &KernelArgs) -> &ElfFile {
    unsafe { &*(args.kernel_elf as *const ElfFile) }
}

#[cfg(test)]
mod tests;


