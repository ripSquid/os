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
        let tags = core::slice::from_raw_parts(raw as *const u8, (header.total_size-8) as usize );
        let tags = MultiBootTags::from_slice(tags)?;
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
        None
    }
}

#[repr(C)]
struct Tag {
    type_: u32,
    size: u32,
}

#[repr(C)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub type_: u32,
    _reserved: u32
}

