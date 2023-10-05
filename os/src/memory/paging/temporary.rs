use crate::memory::{
    frame::{FrameAllocator, MemoryFrame},
    paging::table::EntryFlags,
    VirtualAddress,
};

use super::{
    master::Mapper,
    page::MemoryPage,
    table::{Level1Entry, PageTable},
};

pub struct TemporaryPage {
    page: MemoryPage,
    allo: TinyAllocator<3>,
}

impl TemporaryPage {
    pub fn new<A: FrameAllocator>(page: MemoryPage, allocator: &mut A) -> Self {
        Self {
            page,
            allo: TinyAllocator::new(allocator),
        }
    }
    pub fn map_table_frame(
        &mut self,
        frame: MemoryFrame,
        active_table: &mut Mapper,
    ) -> &mut PageTable<Level1Entry> {
        unsafe { &mut *(self.map(frame, active_table) as *mut PageTable<Level1Entry>) }
    }
    pub fn map(&mut self, frame: MemoryFrame, active_table: &mut Mapper) -> VirtualAddress {
        assert!(active_table.translate_page(self.page).is_none());
        active_table.map_page(self.page, frame, EntryFlags::WRITABLE, &mut self.allo);
        self.page.starting_address()
    }
    pub fn unmap(&mut self, active_table: &mut Mapper) {
        active_table.unmap(self.page, &mut self.allo)
    }
}

pub struct TinyAllocator<const N: usize>([Option<MemoryFrame>; N]);

impl<const N: usize> TinyAllocator<N> {
    pub fn new<A: FrameAllocator>(allocator: &mut A) -> Self {
        //fills every element of the array with the result from ``allocate_frame``.
        Self(core::array::from_fn(|_| allocator.allocate_frame()))
    }
}

impl<const N: usize> FrameAllocator for TinyAllocator<N> {
    fn allocate_frame(&mut self) -> Option<MemoryFrame> {
        for option in self.0.iter_mut() {
            if option.is_some() {
                return option.take();
            }
        }
        None
    }
    fn deallocate_frame(&mut self, frame: MemoryFrame) {
        for option in self.0.iter_mut() {
            if option.is_none() {
                *option = Some(frame);
                return;
            }
        }
        panic!();
    }
}
