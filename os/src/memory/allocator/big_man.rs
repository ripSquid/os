use core::{alloc::Layout, mem::size_of};

use super::PageState;
use crate::{
    display::macros::debug,
    memory::{
        allocator::PageStateTree,
        paging::{EntryFlags, MemoryPage, MemoryPageRange, PageTableMaster},
        ElfTrustAllocator, PAGE_SIZE_4K,
    },
};

const STATES_PER_PAGE: usize = PAGE_SIZE_4K / size_of::<PageState>();

pub struct BigManAllocator {
    range: MemoryPageRange,
    tree: PageStateTree,
}

impl BigManAllocator {
    pub unsafe fn begin(
        range: MemoryPageRange,
        page_table: &mut PageTableMaster,
        allocator: &mut ElfTrustAllocator,
    ) -> Self {
        {
            assert!(range.span() > 1);
            for page in range.iter() {
                page_table.map(page, EntryFlags::PRESENT | EntryFlags::WRITABLE, allocator);
            }
        }
        let (free_page_count, state_page) = {
            let span = range.span();
            let pages_for_state = ((span * 2) - 1) / STATES_PER_PAGE;
            let free_span = span - pages_for_state - 1;
            let state_page = {
                MemoryPage::inside_address(
                    range.start().starting_address() + (free_span * PAGE_SIZE_4K) as u64,
                )
            };
            (free_span, state_page)
        };
        debug!("Available memory:", &free_page_count, "* 4KB");

        let tree = unsafe {
            PageStateTree::new(
                free_page_count,
                state_page.starting_address() as *mut PageState,
            )
        };
        Self { range, tree }
    }
    pub fn deallocate_mut(&mut self, ptr: *mut u8, layout: Layout) {
        self.tree.unallocate(ptr, layout, &self.range)
    }
    pub fn allocate_mut(&mut self, layout: Layout) -> Option<*mut u8> {
        self.tree.allocate(layout, &self.range)
    }
}
