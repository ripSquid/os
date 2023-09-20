use crate::memory::{VirtualAddress, PAGE_SIZE_4K};

//A Virtual area of memory that with maps to a frame.
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
