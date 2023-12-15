use core::ops::Range;

use crate::memory::{VirtualAddress, PAGE_SIZE_4K};

//A Virtual area of memory that with maps to a frame.
#[derive(Clone, Copy)]
pub struct MemoryPage(pub(super) usize);

impl MemoryPage {
    pub fn inside_address(addr: VirtualAddress) -> Self {
        // panics if address doesn't have the right sign extension
        assert!(addr < 0x0000_8000_0000_0000 || addr >= 0xffff_8000_0000_0000);
        Self(addr as usize / PAGE_SIZE_4K)
    }
    #[inline]
    pub fn starting_address(&self) -> VirtualAddress {
        (self.0 * PAGE_SIZE_4K) as u64
    }
}

pub struct MemoryPageRange {
    range: Range<usize>,
}
impl MemoryPageRange {
    pub const fn new(start: MemoryPage, end: MemoryPage) -> Self {
        Self {
            range: start.0..end.0,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = MemoryPage> {
        self.range.clone().into_iter().map(|i| MemoryPage(i))
    }
    pub fn span(&self) -> usize {
        self.range.end.saturating_sub(self.range.start)
    }
    pub fn start(&self) -> MemoryPage {
        MemoryPage(self.range.start)
    }
    pub fn end(&self) -> MemoryPage {
        MemoryPage(self.range.end)
    }
}
