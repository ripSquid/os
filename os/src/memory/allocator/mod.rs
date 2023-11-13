use core::alloc::{GlobalAlloc, Layout};
mod big_man;
mod binary_tree;
mod state;
use crate::display::macros::debug;
pub use binary_tree::{PageStateTree, TreeIndex};
pub use state::PageState;

use self::big_man::BigManAllocator;

use super::{
    paging::{MemoryPage, MemoryPageRange, PageTableMaster},
    ElfTrustAllocator, PAGE_SIZE_4K,
};

pub struct GlobalAllocator {
    start: Option<BigManAllocator>,
}
impl GlobalAllocator {
    pub const fn null() -> Self {
        Self { start: None }
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
        if self.is_active() {
            unsafe { self.start_unchecked() }
        } else {
            panic!()
        }
    }
    pub unsafe fn populate(
        &mut self,
        active_table: &mut PageTableMaster,
        allocator: &mut ElfTrustAllocator,
        available_pages: usize,
    ) -> MemoryPageRange {
        let available_pages = available_pages * 8 / 10;
        let pages = MemoryPageRange::new(
            MemoryPage::inside_address(0x8000000),
            MemoryPage::inside_address(0x8000000 + (available_pages * PAGE_SIZE_4K) as u64),
        );
        debug!("Total Available memory:", &available_pages, "* 4KB");
        self.start = {
            let big_man = BigManAllocator::begin(
                MemoryPageRange::new(pages.start(), pages.end()),
                active_table,
                allocator,
            );
            Some(big_man)
        };
        pages
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
