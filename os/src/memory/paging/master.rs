use core::ops::{Deref, DerefMut};

use x86_64::{registers::control::Cr3Flags, structures::paging::PhysFrame, PhysAddr, VirtAddr};

use crate::memory::{
    frame::{FrameAllocator, MemoryFrame},
    PhysicalAddress, VirtualAddress, PAGE_SIZE_4K,
};

use super::{
    page::MemoryPage,
    table::{EntryFlags, Level4Entry, PageTable},
    temporary::TemporaryPage,
};

pub const P4_TABLE: *mut PageTable<Level4Entry> = 0xffffffff_fffff000 as *mut _;
pub struct Mapper<'a> {
    p4: &'a mut PageTable<Level4Entry>,
}

pub struct InactivePageTable(MemoryFrame);

impl InactivePageTable {
    pub fn new(
        frame: MemoryFrame,
        active_table: &mut Mapper,
        temp_page: &mut TemporaryPage,
    ) -> Self {
        {
            let table = temp_page.map_table_frame(frame.clone(), active_table);
            table.zero_out();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temp_page.unmap(active_table);
        Self(frame)
    }
    pub fn as_address(&self) -> u64 {
        self.0.starting_address()
    }
}

pub struct PageTableMaster<'a> {
    page_table: Mapper<'a>,
}
impl<'a> Deref for PageTableMaster<'a> {
    type Target = Mapper<'a>;

    fn deref(&self) -> &Self::Target {
        &self.page_table
    }
}
impl<'a> DerefMut for PageTableMaster<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.page_table
    }
}

impl<'a> PageTableMaster<'a> {
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable(MemoryFrame::inside_address(
            x86_64::registers::control::Cr3::read()
                .0
                .start_address()
                .as_u64(),
        ));
        unsafe {
            x86_64::registers::control::Cr3::write(
                PhysFrame::containing_address(PhysAddr::new(new_table.0.starting_address())),
                Cr3Flags::empty(),
            );
        }
        old_table
    }
    pub fn with<F: FnOnce(&mut Mapper)>(
        &mut self,
        table: &mut InactivePageTable,
        temp_page: &mut TemporaryPage,
        f: F,
    ) {
        {
            let backup = MemoryFrame::inside_address(
                x86_64::registers::control::Cr3::read_raw()
                    .0
                    .start_address()
                    .as_u64(),
            );
            let p4_table = temp_page.map_table_frame(backup.clone(), self);
            self.p4_mut()[511].set(table.0.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            x86_64::instructions::tlb::flush_all();
            f(&mut self.page_table);

            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            x86_64::instructions::tlb::flush_all();
        }
        temp_page.unmap(self);
    }
    pub unsafe fn new() -> Self {
        Self {
            page_table: Mapper::new(),
        }
    }
}

impl<'a> Mapper<'a> {
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
    pub fn page_present<A: FrameAllocator>(&mut self, page: MemoryPage, allocator: &mut A) -> bool {
        let p4 = self.p4_mut();
        let p3 = p4.child_table_search(page.p4_index(), allocator);
        let p2 = p3.child_table_search(page.p3_index(), allocator);
        let p1 = p2.child_table_search(page.p2_index(), allocator);

        !p1[page.p1_index()].is_unused()
    }
    pub fn map_page<A: FrameAllocator>(
        &mut self,
        page: MemoryPage,
        frame: MemoryFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.child_table_search(page.p4_index(), allocator);
        let p2 = p3.child_table_search(page.p3_index(), allocator);
        let p1 = p2.child_table_search(page.p2_index(), allocator);

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
