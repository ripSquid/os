use core::{mem::{size_of, align_of}, slice::from_raw_parts};

use crate::display::{KernelDebug, KernelFormatter};

use super::{transmute, type_after, TagHeader, TagType};

#[repr(C)]
pub struct ElfSymbolTagHeader {
    typ: TagType,
    size: u32,
    number: u16,
    entry_size: u16,
    sh_index: u16,
    _reserved: u16,
}
pub struct ElfSectionHeaders<'a> {
    raw: &'a [u32],
    parsed: &'a [ElfSectionHeader],
}

pub struct ElfSymbolTag<'a> {
    header: &'a ElfSymbolTagHeader,
    entries: ElfSectionHeaders<'a>,
}

#[repr(C)]
pub struct ElfSectionHeader {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

impl<'a> ElfSymbolTag<'a> {
    pub unsafe fn from_ref(pointer: &'a TagHeader) -> Self {
        let header: &'a ElfSymbolTagHeader = &*transmute(pointer as *const TagHeader);

        assert!(header.typ == TagType::ElfSymbol);
        
        let len = header.size as usize - size_of::<ElfSymbolTagHeader>();

        let sections_pointer = {
            let pointer: *const u8 = type_after(pointer as *const TagHeader);
            assert!(pointer as u64 % 8 == 0);
            pointer.add(4)
        };
        
        let raw = from_raw_parts(transmute(sections_pointer), len/4);
        let parsed = from_raw_parts(
            transmute(sections_pointer),
            header.number as usize,
        );
        let entries = ElfSectionHeaders { raw, parsed };
        Self { header, entries }
    }
}

impl<'a> KernelDebug<'a> for ElfSymbolTag<'a> {
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
            .debug_field("name", &self.sh_name)
            .debug_field("flags", &self.sh_flags)
            .debug_field("align", &self.sh_addralign)
            .debug_field("entsize", &self.sh_entsize)
            .debug_field("info", &self.sh_info)
            .debug_field("link", &self.sh_link)
            .debug_field("offset", &self.sh_offset)
            .debug_field("size", &self.sh_size)
            .debug_field("type", &self.sh_type)
            .finish()
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
