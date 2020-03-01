use core::fmt;

use crate::elf::{Elf32, Elf64, Error, GenElf, GenElfHeader};
use crate::elf::elf_type::{ElfKind, MAGIC_BITES};
use crate::elf::traits::{GenProgramHeader, GenSectionHeader};

////////////////////
//// Segments
////////////////////
pub struct ProgramHeader<'a, E: GenElf> {
    elf: &'a E,
    pub ph: &'a E::ProgramHeaderType,
}

impl<'a, E: GenElf> ProgramHeader<'a, E> {
    pub fn segment(&self) -> &[u8] {
        let seg_off = self.ph.offset().into() as usize;
        let seg_filesz = self.ph.filesz().into() as usize;
        &self.elf.as_bytes()[seg_off..seg_off + seg_filesz]
    }
}

impl<'a, E: GenElf> fmt::Debug for ProgramHeader<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Program Header")
            .field("type", &self.ph.ph_type())
            .field("flags", &self.ph.flags())
            .field("offset", &self.ph.offset())
            .field("vaddr", &self.ph.vaddr())
            .field("paddr", &self.ph.paddr())
            .field("filesize", &self.ph.filesz())
            .field("memsize", &self.ph.memsz())
            .field("alignment", &self.ph.align())
            .finish()
    }
}


pub struct SectionHeader<'a, E: GenElf> {
    elf: &'a E,
    pub sh: &'a E::SectionHeaderType,
}

impl<'a, E: GenElf> SectionHeader<'a, E> {
    pub fn segment(&'a self) -> &'a [u8] {
        let seg_off = self.sh.offset().into() as usize;
        let seg_filesz = self.sh.size().into() as usize;
        &self.elf.as_bytes()[seg_off..seg_off + seg_filesz]
    }

    pub fn section_name(&'a self) -> CChar {
        let name_off = self.sh.name_off() as usize;
        let shstr = self.elf.shstr_section();
        let name_len = shstr[name_off..].iter()
            .position(|&x| x == b'\0')
            .unwrap();
        CChar(&shstr[name_off..name_off + name_len])
    }
}

impl<'a, E: GenElf> fmt::Debug for SectionHeader<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Section Header")
            .field("name", &self.section_name())
            .field("type", &self.sh.sh_type())
            .field("flags", &self.sh.flags())
            .field("addr", &self.sh.addr())
            .field("offset", &self.sh.offset())
            .field("size", &self.sh.size())
            .field("link", &self.sh.link())
            .field("info", &self.sh.info())
            .field("address alignment", &self.sh.addralign())
            .field("entry size", &self.sh.entsize())
            .finish()
    }
}

////////////////////
//// Iterator
////////////////////
#[derive(Debug)]
pub enum Elf<'a> {
    Elf32(Elf32<'a>),
    Elf64(Elf64<'a>),
}

impl<'a> Elf<'a> {
    pub fn from_bytes(elf_buf: &'a [u8]) -> Result<Self, Error> {
        if elf_buf.len() < 0x14 {
            return Err(Error::BufferTooShort);
        }

        if !elf_buf.starts_with(&MAGIC_BITES) {
            return Err(Error::InvalidMagic);
        }

        let tmp_elf = Elf32::new(elf_buf);
        match tmp_elf.header().class.into() {
            ElfKind::Elf64 => {
                let elf = Elf64::new(elf_buf);
                if elf_buf.len() < elf.header().elf_header_size() as usize {
                    Err(Error::BufferTooShort)
                } else {
                    Ok(Elf::Elf64(elf))
                }
            }
            ElfKind::Elf32 => {
                let elf = Elf32::new(elf_buf);
                if elf_buf.len() < elf.header().elf_header_size() as usize {
                    Err(Error::BufferTooShort)
                } else {
                    Ok(Elf::Elf32(elf))
                }
            }
            ElfKind::Unknown(_) => { Err(Error::InvalidClass) }
        }
    }
}

pub struct CChar<'a> (pub &'a [u8]);

impl<'a> fmt::Debug for CChar<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use core::fmt::Write;
        for c in self.0.iter() {
            f.write_char(*c as char)?;
        }

        Ok(())
    }
}


////////////////////
//// Iterator
////////////////////
#[derive(Debug)]
pub struct ProgramHeaderIter<'a, E: GenElf> {
    elf: &'a E,
    ph: &'a [E::ProgramHeaderType],
    idx: u16,
}

impl<'a, E: GenElf> ProgramHeaderIter<'a, E> {
    pub fn new(e: &'a E, ph: &'a [E::ProgramHeaderType]) -> Self {
        ProgramHeaderIter {
            elf: e,
            ph: e.program_headers(),
            idx: 0,
        }
    }
}

impl<'a, E: GenElf> core::iter::Iterator for ProgramHeaderIter<'a, E> {
    type Item = ProgramHeader<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(ProgramHeader {
            elf: self.elf,
            ph: self.ph.get(self.idx as usize)?,
        });
        self.idx += 1;
        ret
    }
}

#[derive(Debug)]
pub struct SectionHeaderIter<'a, E: GenElf> {
    elf: &'a E,
    sh: &'a [E::SectionHeaderType],
    idx: u16,
}

impl<'a, E: GenElf> SectionHeaderIter<'a, E> {
    pub fn new(e: &'a E, ph: &'a [E::SectionHeaderType]) -> Self {
        SectionHeaderIter {
            elf: e,
            sh: e.section_headers(),
            idx: 0,
        }
    }
}


impl<'a, E: GenElf> core::iter::Iterator for SectionHeaderIter<'a, E> {
    type Item = SectionHeader<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(SectionHeader {
            elf: self.elf,
            sh: self.sh.get(self.idx as usize)?,
        });
        self.idx += 1;
        ret
    }
}
