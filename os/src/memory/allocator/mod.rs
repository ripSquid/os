use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    mem::size_of,
};
pub mod alternate;
use crate::display::macros::debug;

use self::alternate::BigManAllocator;

use super::{
    frame::{FrameAllocator, MemoryFrame},
    paging::{
        master::{Mapper, PageTableMaster},
        page::{MemoryPage, MemoryPageRange},
        table::EntryFlags,
    },
    ElfTrustAllocator, PAGE_SIZE_4K,
};

pub struct GlobalAllocator {
    next: (usize, usize),
    start: Option<BigManAllocator>,
}
impl GlobalAllocator {
    pub const fn null() -> Self {
        Self {
            next: (0, 0),
            start: None,
        }
    }
    pub fn is_active(&self) -> bool {
        self.start.is_some()
    }
    pub unsafe fn start_unchecked(&self) -> &mut BigManAllocator {
        (self as *const Self as *mut Self)
            .as_mut()
            .unwrap()
            .start
            .as_mut()
            .unwrap()
    }
    fn start(&self) -> &mut BigManAllocator {
        unsafe { self.start_unchecked() }
    }
    pub unsafe fn populate(
        &mut self,
        active_table: &mut PageTableMaster,
        allocator: &mut ElfTrustAllocator,
        available_pages: usize,
    ) {
        let available_pages = available_pages * 8 / 10;
        let pages = MemoryPageRange::new(
            MemoryPage::inside_address(0x8000000),
            MemoryPage::inside_address(0x8000000 + (available_pages * PAGE_SIZE_4K) as u64),
        );
        debug!("Total Available memory:", &available_pages, "* 4KB");
        let big_man = BigManAllocator::begin(pages, active_table, allocator);
        self.start = Some(big_man);
    }
}
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.start().allocate_mut(layout).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.start().deallocate_mut(ptr, layout);
    }
}

type RawPageMemory = [u8; 4096];

