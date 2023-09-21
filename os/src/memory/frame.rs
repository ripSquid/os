use x86_64::{structures::paging::frame, VirtAddr};

use super::{
    paging::{
        page::MemoryPage,
        table::{EntryFlags, Level4Entry, PageTable},
    },
    MemoryAddress, PhysicalAddress, VirtualAddress, PAGE_SIZE_4K,
};

//A Physical area of memory where usize is offset by 0x1000
#[derive(Clone, Copy)]
pub struct MemoryFrame(usize);

impl MemoryFrame {
    #[inline]
    pub fn inside_address(address: PhysicalAddress) -> Self {
        Self(address as usize / PAGE_SIZE_4K)
    }
    #[inline]
    pub fn starting_address(&self) -> PhysicalAddress {
        (self.0 * PAGE_SIZE_4K) as u64
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<MemoryFrame>;
    fn deallocate_frame(&mut self, frame: MemoryFrame);
}
