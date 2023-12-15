use core::{mem::size_of, slice::from_raw_parts};

use crate::{
    display::{KernelDebug, KernelFormatter},
    memory::frame::{FrameRangeInclusive, MemoryFrame},
};

use super::{transmute, type_after, TagHeader, TagType};

#[repr(C)]
pub struct ElfSymbolTagHeader {
    typ: TagType,
    size: u32,
    number: u32,
    entry_size: u32,
    sh_index: u32,
}

#[allow(dead_code)]
pub struct ElfSectionHeaders<'a> {
    raw: &'a [u32],
    pub parsed: &'a [ElfSectionHeader],
}

pub struct ElfSymbolTag {
    pub header: &'static ElfSymbolTagHeader,
    pub entries: ElfSectionHeaders<'static>,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum ElfSectionType {
    NULL = 0,
    PROGBITS = 1,
    SYMTAB = 2,
    STRTAB = 3,
    RELA = 4,
    HASH = 5,
    DYNAMIC = 6,
    NOTE = 7,
    NOBITS = 8,
    REL = 9,
    SHLIB = 10,
    DYNSYM = 11,
    LOOS = 0x60000000,
    HIOS = 0x6FFFFFFF,
    LOPROC = 0x70000000,
    HIPROC = 0x7FFFFFFF,
}

#[repr(C)]
pub struct ElfSectionHeader {
    pub sh_name: u32,
    pub sh_type: ElfSectionType,
    pub sh_flags: ElfSectionFlags,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

bitflags! {
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct ElfSectionFlags: u64 {
        /// The section contains data that should be writable during program execution.
        const WRITABLE = 1 << 0 ;

        /// The section occupies memory during the process execution.
        const ALLOCATED = 1 << 1;

        /// The section contains executable machine instructions.
        const EXECUTABLE = 1 << 2;
    }
}

impl ElfSymbolTag {
    pub unsafe fn from_ref(pointer: &'static TagHeader) -> Self {
        let header: &ElfSymbolTagHeader = &*transmute(pointer as *const TagHeader);

        assert!(header.typ == TagType::ElfSymbol);

        let len = header.size as usize - size_of::<ElfSymbolTagHeader>();

        let sections_pointer = {
            let pointer: *const u8 = type_after(header as *const ElfSymbolTagHeader);
            pointer
        };

        let raw = from_raw_parts(transmute(sections_pointer), len / 4);
        let parsed = from_raw_parts(transmute(sections_pointer), header.number as usize);
        let entries = ElfSectionHeaders { raw, parsed };
        Self { header, entries }
    }
    pub unsafe fn frame_range(&self) -> FrameRangeInclusive {
        let mut start = u64::MAX;
        let mut end = u64::MIN;
        for entry in self.entries.parsed.iter() {
            if entry.sh_type == ElfSectionType::NULL {
                continue;
            }
            start = start.min(entry.sh_addr);
            end = end.max(entry.sh_addr + entry.sh_size - 1);
        }

        FrameRangeInclusive::new(
            MemoryFrame::inside_address(start),
            MemoryFrame::inside_address(end),
        )
    }
}

impl<'a> KernelDebug<'a> for ElfSymbolTag {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("ElfSymbolsTag")
            .debug_field("header", self.header)
            .debug_field("entries", &self.entries)
            .finish()
    }
}
impl<'a> KernelDebug<'a> for ElfSectionHeader {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("ElfSection")
            .debug_field("addr", &self.sh_addr)
            //.debug_field("name", &self.sh_name)
            //.debug_field("flags", &self.sh_flags)
            //.debug_field("align", &self.sh_addralign)
            //.debug_field("entsize", &self.sh_entsize)
            //.debug_field("info", &self.sh_info)
            //.debug_field("link", &self.sh_link)
            //.debug_field("offset", &self.sh_offset)
            .debug_field("size", &self.sh_size)
            .debug_field("type", &self.sh_type)
            .finish()
    }
}
impl<'a> KernelDebug<'a> for ElfSectionType {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        match self {
            ElfSectionType::NULL => formatter.debug_str("NULL"),
            ElfSectionType::PROGBITS => formatter.debug_str("PROGB"),
            ElfSectionType::SYMTAB => formatter.debug_str("SYMTAB"),
            ElfSectionType::STRTAB => formatter.debug_str("STRTAB"),
            ElfSectionType::RELA => formatter.debug_str("RELA"),
            ElfSectionType::HASH => formatter.debug_str("HASH"),
            ElfSectionType::DYNAMIC => formatter.debug_str("DYNAMIC"),
            ElfSectionType::NOTE => formatter.debug_str("NOTE"),
            ElfSectionType::NOBITS => formatter.debug_str("NOBITS"),
            ElfSectionType::REL => formatter.debug_str("REL"),
            ElfSectionType::SHLIB => formatter.debug_str("SHLIB"),
            ElfSectionType::DYNSYM => formatter.debug_str("DYNSYM"),
            ElfSectionType::LOOS => formatter.debug_str("LOOS"),
            ElfSectionType::HIOS => formatter.debug_str("HIOS"),
            ElfSectionType::LOPROC => formatter.debug_str("LOPROC"),
            ElfSectionType::HIPROC => formatter.debug_str("HIPROC"),
        }
    }
}

impl<'a> KernelDebug<'a> for ElfSectionHeaders<'a> {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        self.parsed.debug(formatter)
    }
}
impl<'a> KernelDebug<'a> for ElfSymbolTagHeader {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("ElfSymbolHeader")
            .debug_field("entrysize", &self.entry_size)
            .debug_field("entrycount", &self.number)
            .finish()
    }
}
