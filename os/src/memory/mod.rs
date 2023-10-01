use core::{alloc::{GlobalAlloc, Layout}, iter::Filter};


use crate::multiboot_info::memory_map::{MemoryMapEntry, MemoryType};

use self::frame::{MemoryFrame, FrameRangeInclusive, FrameAllocator};
pub mod frame;
pub mod paging;

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


pub struct ElfTrustAllocator {
    next_free_frame: MemoryFrame,
    active_area: Option<&'static MemoryMapEntry>,
    areas: MemoryAreaIter,
    multiboot: FrameRangeInclusive,
    kernel: FrameRangeInclusive,

}
#[derive(Clone)]
pub struct MemoryAreaIter {
    itera: Filter<core::slice::Iter<'static, MemoryMapEntry>, &'static dyn Fn(&&MemoryMapEntry) -> bool>
}
impl MemoryAreaIter {
    pub fn new(slice: &'static [MemoryMapEntry]) -> Self {
        Self { itera: slice.iter().filter(&(|i| i.mem_type == MemoryType::Available)) }
    }
}
impl Iterator for MemoryAreaIter {
    type Item = &'static MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.itera.next()
    }
}

