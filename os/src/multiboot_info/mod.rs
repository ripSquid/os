pub mod elf;
pub mod memory_map;
use elf::*;
use memory_map::*;

use core::{mem::size_of, str::from_utf8};

use crate::{
    display::{
        macros::{print_hex, print_str},
        KernelDebug,
    },
    memory::frame::{FrameRangeInclusive, MemoryFrame},
};

#[derive(Clone, Copy)]
pub struct MultibootInfoUnparsed {
    pub header: MultibootInfoHeader,
    pub tags: MultiBootTags,
    pointer: u64,
}
impl<'a> KernelDebug<'a> for MultibootInfoUnparsed {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        formatter.debug_bytes(self.tags.bytes())
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MultibootInfoHeader {
    total_size: u32,
    _reserved: u32,
}
impl MultibootInfoUnparsed {
    pub unsafe fn from_pointer(pointer: *const MultibootInfoHeader) -> Option<Self> {
        if pointer.align_offset(8) != 0 || pointer.is_null() {
            return None;
        }
        let header = *pointer;
        let raw = pointer as u64;
        let tags =
            core::slice::from_raw_parts((raw + 24) as *const u8, (header.total_size - 24) as usize);
        let tags = MultiBootTags::from_slice(tags)?;
        Some(MultibootInfoUnparsed {
            header,
            tags,
            pointer: pointer as u64,
        })
    }
    fn size(&self) -> usize {
        self.header.total_size as usize
    }
    pub fn tag_iter(&self) -> MultiBootTagIter {
        MultiBootTagIter::new(*self)
    }
    pub unsafe fn frame_range(&self) -> FrameRangeInclusive {
        FrameRangeInclusive::new(
            MemoryFrame::inside_address(self.pointer),
            MemoryFrame::inside_address(self.pointer + (self.header.total_size as u64) - 1),
        )
    }
}

impl<'a> KernelDebug<'a> for MultiBootTag {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        match self {
            MultiBootTag::MemoryMap(map) => map.debug(formatter),
            MultiBootTag::BasicMem(_) => todo!(),
            MultiBootTag::BootoaderName(map) => formatter.debug_str(map.as_str()),
            MultiBootTag::ElfSymbols(map) => map.debug(formatter),
            MultiBootTag::End => formatter.debug_str("end"),
        }
    }
}
pub struct MultiBootTagIter {
    byte_index: usize,
    tags: MultibootInfoUnparsed,
}
impl MultiBootTagIter {
    pub fn new(info: MultibootInfoUnparsed) -> Self {
        Self {
            byte_index: 0,
            tags: info,
        }
    }
}
impl Iterator for MultiBootTagIter {
    type Item = MultiBootTag;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_index < self.tags.size() {
            let tag_head: &TagHeader = unsafe {
                &*transmute((self.tags.pointer + 24 + self.byte_index as u64) as *const u8)
            };
            self.byte_index += ((tag_head.size + 7) & MASK8) as usize;
            match tag_head.tag_type {
                TagType::BootoaderName => {
                    let info = unsafe { BootloaderNameTag::from_ref(&tag_head) };
                    return Some(MultiBootTag::BootoaderName(info));
                }
                TagType::BasicMemoryTag => {
                    let info = unsafe { BasicMemoryTag::from_ref(&tag_head) };
                    return Some(MultiBootTag::BasicMem(info));
                }
                TagType::MemoryMap => {
                    let info = unsafe { MemoryMapTag::from_ref(&tag_head) };
                    return Some(MultiBootTag::MemoryMap(info));
                }
                TagType::ElfSymbol => {
                    let info = unsafe { ElfSymbolTag::from_ref(&tag_head) };
                    return Some(MultiBootTag::ElfSymbols(info));
                }
                TagType::End => {
                    if tag_head.size == 8 {
                        return Some(MultiBootTag::End);
                    } else {
                        panic!();
                    }
                }
                TagType::FramebufferInfo => (),
                TagType::BootCommandLine => (),
                TagType::BiosBootDevice => (),
                TagType::ApmTable => (),
                TagType::VbeInfo => (),
                TagType::Module => (),
                _ => panic!(),
            }
            //rounds upward to nearest multiple of 8
        }
        None
    }
}

#[derive(Clone, Copy)]
pub struct MultiBootTags(&'static [u8]);

pub enum MultiBootTag {
    MemoryMap(MemoryMapTag),
    BasicMem(BasicMemoryTag),
    BootoaderName(BootloaderNameTag),
    ElfSymbols(ElfSymbolTag),
    End,
}
impl MultiBootTag {
    pub fn tag_type(&self) -> TagType {
        match self {
            MultiBootTag::MemoryMap(_) => TagType::MemoryMap,
            MultiBootTag::BootoaderName(_) => TagType::BootoaderName,
            MultiBootTag::ElfSymbols(_) => TagType::ElfSymbol,
            MultiBootTag::BasicMem(_) => TagType::BasicMemoryTag,
            MultiBootTag::End => TagType::End,
        }
    }
}

impl MultiBootTags {
    pub fn bytes(&self) -> &[u8] {
        self.0
    }
    pub fn from_slice(slice: &'static [u8]) -> Option<Self> {
        Some(Self(slice))
    }
}

//provides a mask that removes the last 3 bits of any u32 (rounding it to nearest multiple of 8)
const MASK8: u32 = u32::MAX - 0x07;

#[allow(dead_code)]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    End = 0,
    BootCommandLine = 1,
    BootoaderName = 2,
    Module = 3,
    BasicMemoryTag = 4,
    BiosBootDevice = 5,
    MemoryMap = 6,
    VbeInfo = 7,
    FramebufferInfo = 8,
    ElfSymbol = 9,
    ApmTable = 10,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TagHeader {
    tag_type: TagType,
    size: u32,
}

pub struct BootloaderNameTag {
    head: &'static TagHeader,
    name: &'static str,
}
impl BootloaderNameTag {
    pub unsafe fn from_ref(head: &'static TagHeader) -> Self {
        let pointer: *const u8 = type_after(head as *const TagHeader);
        let sting_len = head.size as usize - size_of::<TagHeader>() - 1;
        let string_bytes = core::slice::from_raw_parts(pointer as *const u8, sting_len);
        let name = from_utf8(string_bytes).unwrap();
        Self { head, name }
    }
    pub fn as_str(&self) -> &'static str {
        self.name
    }
}

/// Gives a pointer to a data type laid out after the one pointed to in `pointer`
pub unsafe fn type_after<B, A>(pointer: *const B) -> *const A {
    pointer.offset(1) as u64 as *const A
}

/// turn a pointer of one type into another, mega hacky!!!
pub unsafe fn transmute<B, A>(pointer: *const B) -> *const A {
    pointer as u64 as *const A
}
/// turn a pointer of one type into another, mega hacky!!!
pub unsafe fn transmute_ref<B, A>(pointer: &B) -> *const A {
    pointer as *const B as u64 as *const A
}
