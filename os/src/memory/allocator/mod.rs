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
    ElfTrustAllocator,
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
    ) {
        let pages = MemoryPageRange::new(
            MemoryPage::inside_address(0x8000000),
            MemoryPage::inside_address(0x8040000),
        );
        let big_man = BigManAllocator::begin(pages, active_table, allocator);
        //let reserve = AllocatorChunk::create(active_table, allocator, pages)
        //    .as_mut()
        //    .unwrap();
        //debug!("reserved:", &reserve.size_in_pages(), "pages.");
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

