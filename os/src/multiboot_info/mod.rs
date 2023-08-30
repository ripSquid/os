use core::mem::size_of;

use crate::display::macros::{print_str, print_hex};

pub struct MultibootInfo<'a> {
    pub header: MultibootInfoHeader,
    pub tags: MultiBootTags<'a>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MultibootInfoHeader {
    total_size: u32,
    _reserved: u32,
}
impl<'a> MultibootInfo<'a> {
    pub unsafe fn from_pointer(pointer: *const MultibootInfoHeader) -> Option<Self> {
        if pointer.align_offset(8) != 0 || pointer.is_null() {
            return None;
        }
        let header = *pointer;
        let raw = pointer as u64;
        let tags = core::slice::from_raw_parts((raw + 40) as *const u8, (header.total_size-40) as usize );
        let tags = MultiBootTags::from_slice(tags)?;
        print_hex!(header.total_size);
        Some(MultibootInfo { header, tags })
    }
    pub fn size(&self) -> usize {
        self.header.total_size as usize
    }
}

pub struct MultiBootTags<'a>(&'a [u8]);

impl<'a> MultiBootTags<'a> {
    pub fn bytes(&self) -> &[u8] {
        self.0
    }
    pub fn from_slice(slice: &'a [u8]) -> Option<Self> {
        Some(Self(slice))
    }
    pub fn memory_tag(&self) -> Option<&'a [MemoryMapEntry]> {
        let mut searching = true;
        let mut counter = 0;
        let tag_head_pointer = &self.0[counter] as *const u8 as u64;
        let tag_head = unsafe {&*(tag_head_pointer as *const TagHeader)};
        print_hex!(self.0[0]);
        
        
        match tag_head.tag_type {
            TagType::BootCommandLine => print_str!("command line"),
            TagType::BootLoaderName => {
                print_str!("name");
            },
            TagType::Module => print_str!("modul"),
            TagType::BasicMemoryTag => print_str!("basic mem"),
            TagType::BiosBootDevice => print_str!("boot device"),
            TagType::MemoryMap => {
                print_str!("memory!!!");
            },
            TagType::VbeInfo => print_str!("vbe"),
            TagType::FramebufferInfo => print_str!("frame info"),
            TagType::ElfSymbol => print_str!("elf symbol"),
            TagType::ApmTable => print_str!("apm"),
            TagType::End => {
                print_str!("end tag");
                if tag_head.size == 8 {
                    return None;
                } else {
                    print_str!("SOMETHIGN IS WROGN WITH END TAG!");
                    panic!();
                }
            },
            _ => print_str!("PISS AND SHIT AND FUCK")
        }

        return None;
        
        let moving = {
            let unpadded = tag_head.size as usize;
            if unpadded & 0x07 == 0 {
                unpadded
            } else {
                (unpadded + 8) & 0xFFFF_FFFF_FFFF_FFF8
            }
        };
        counter += moving;

        None
        
    }
}

#[repr(u32)]
enum TagType {
    End = 0,
    BootCommandLine = 1,
    BootLoaderName = 2,
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
struct TagHeader {
    tag_type: TagType,
    size: u32,
}

#[repr(C)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub mem_type: u32,
    _reserved: u32
}

#[repr(C)]
pub struct MemoryMapHeader {
    tag_type: TagType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

