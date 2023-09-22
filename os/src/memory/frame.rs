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


impl Iterator for FrameIter {
    type Item = MemoryFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame.0 <= self.end {
            self.frame.0 += 1;
            Some(MemoryFrame(self.frame.0-1))
        } else {
            None
        }
    }
    
}
pub struct FrameIter {
    frame: MemoryFrame,
    start: usize,
    end: usize,
}

pub struct FrameRangeInclusive {
    start_frame: usize,
    end_frame: usize,
}

impl FrameRangeInclusive {
    pub fn contains(&self, frame: &MemoryFrame) -> bool {
        (self.start_frame..=self.end_frame).contains(&frame.0)
    }
    pub fn new(start: MemoryFrame, end: MemoryFrame) -> Self {
        Self { start_frame: start.0, end_frame: end.0 }
    }
}

impl IntoIterator for FrameRangeInclusive {
    type Item = MemoryFrame;

    type IntoIter = FrameIter;

    fn into_iter(self) -> Self::IntoIter {
        FrameIter {frame: MemoryFrame(self.start_frame), start: self.start_frame, end: self.end_frame}
    }
}