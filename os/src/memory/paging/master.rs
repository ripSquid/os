use x86_64::VirtAddr;

use crate::memory::{
    frame::{FrameAllocator, MemoryFrame},
    PhysicalAddress, VirtualAddress, PAGE_SIZE_4K,
};

use super::{
    page::MemoryPage,
    table::{EntryFlags, Level4Entry, PageTable},
};

pub const P4_TABLE: *mut PageTable<Level4Entry> = 0xffffffff_fffff000 as *mut _;
pub struct PageTableMaster<'a> {
    p4: &'a mut PageTable<Level4Entry>,
}
impl<'a> PageTableMaster<'a> {
    pub unsafe fn new() -> Self {
        Self { p4: &mut *P4_TABLE }
    }
    fn p4(&self) -> &PageTable<Level4Entry> {
        &self.p4
    }
    fn p4_mut(&mut self) -> &mut PageTable<Level4Entry> {
        &mut self.p4
    }

    pub fn translate_page(&self, page: MemoryPage) -> Option<MemoryFrame> {
        let p3 = self.p4().child_table(page.p4_index());
        p3.and_then(|p3| p3.child_table(page.p3_index()))
            .and_then(|p2| p2.child_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
    }

    pub fn translate_addr(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let index = virtual_address % PAGE_SIZE_4K as u64;

        self.translate_page(MemoryPage::inside_address(virtual_address))
            .map(|frame| frame.starting_address() + index)
    }

    pub fn map_page<A: FrameAllocator>(
        &mut self,
        page: MemoryPage,
        frame: MemoryFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let mut p3 = p4.child_table_search(page.p4_index(), allocator);
        let mut p2 = p3.child_table_search(page.p3_index(), allocator);
        let mut p1 = p2.child_table_search(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn map<A: FrameAllocator>(
        &mut self,
        page: MemoryPage,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_page(page, frame, flags, allocator)
    }

    pub fn identity_map<A: FrameAllocator>(
        &mut self,
        frame: MemoryFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let page = MemoryPage::inside_address(frame.starting_address());
        self.map_page(page, frame, flags, allocator)
    }
    pub fn unmap<A: FrameAllocator>(&mut self, page: MemoryPage, allocator: &mut A) {
        assert!(self.translate_addr(page.starting_address()).is_some());
        let p1 = self
            .p4_mut()
            .child_table_mut(page.p4_index())
            .and_then(|p3| p3.child_table_mut(page.p3_index()))
            .and_then(|p2| p2.child_table_mut(page.p2_index()))
            .expect("");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        x86_64::instructions::tlb::flush(VirtAddr::new(page.starting_address()));
        allocator.deallocate_frame(frame);
    }
}
impl MemoryPage {
    fn p4_index(&self) -> usize {
        (self.0 >> 27) & 0x1FF
    }
    fn p3_index(&self) -> usize {
        (self.0 >> 18) & 0x1FF
    }
    fn p2_index(&self) -> usize {
        (self.0 >> 9) & 0x1FF
    }
    fn p1_index(&self) -> usize {
        self.0 & 0o777
    }
}
