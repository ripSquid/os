use core::alloc::{GlobalAlloc, Layout};


use crate::multiboot_info::memory_map::MemoryMapEntry;

use self::frame::{MemoryFrame, FrameRangeInclusive, FrameAllocator};
mod frame;
mod paging;

type MemoryAddress = u64;

type PhysicalAddress = MemoryAddress;
type VirtualAddress = MemoryAddress;

const PAGE_SIZE_4K: usize = 4096;

#[global_allocator]
static GLOBAL_ALLOCATOR: GymnasieAllocator = GymnasieAllocator::new();
struct GymnasieAllocator {

}
impl GymnasieAllocator {
    pub const fn new() -> Self {
        Self {

        }
    }
}

unsafe impl GlobalAlloc for GymnasieAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}


struct ElfTrustAllocator {
    next_free_frame: MemoryFrame,
    memory_areas: Option<&'static MemoryMapEntry>,
    areas: MemoryAreaIter,
    multiboot: FrameRangeInclusive,
    kernel: FrameRangeInclusive,

}
struct MemoryAreaIter {
    areas: &'static [MemoryMapEntry],
    index: usize,
}
impl MemoryAreaIter {
    pub fn new(slice: &'static [MemoryMapEntry]) -> Self {
        Self { areas: slice, index: 0 }
    }
}
impl Iterator for MemoryAreaIter {
    type Item = &'static MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.areas.get(self.index-1)
    }
}

impl FrameAllocator for ElfTrustAllocator {
    fn allocate_frame(&mut self) -> Option<MemoryFrame> {
        todo!()
    }

    fn deallocate_frame(&mut self, frame: MemoryFrame) {
        todo!()
    }
}
