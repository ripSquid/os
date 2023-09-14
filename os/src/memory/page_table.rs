use core::{marker::PhantomData, ops::{IndexMut, Index}};

use super::{frame::MemoryFrame, MemoryAddress, PhysicalAddress};

pub trait PageLevel {}
pub trait PageLevelParent: PageLevel {
    type ChildLevel: PageLevel;
}


//These all have a size of 0, meaning they dissapear at compile time.
pub enum Level4Entry {}
pub enum Level3Entry {}
pub enum Level2Entry {}
pub enum Level1Entry {}

impl PageLevel for Level4Entry {}
impl PageLevel for Level3Entry {}
impl PageLevel for Level2Entry {}
impl PageLevel for Level1Entry {}

impl PageLevelParent for Level4Entry {
    type ChildLevel = Level3Entry;
}
impl PageLevelParent for Level3Entry {
    type ChildLevel = Level2Entry;
}
impl PageLevelParent for Level2Entry {
    type ChildLevel = Level1Entry;
}
pub const P4_TABLE: *mut PageTable<Level4Entry> = 0xffffffff_fffff000 as *mut _;
pub struct PageTableEntry(u64);

const PAGE_TABLE_ENTRY_COUNT: usize = 512;
pub struct PageTable<Level: PageLevel> {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
    level: PhantomData<Level>
}

impl<Level: PageLevel> Index<usize> for PageTable<Level> {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}
impl<Level: PageLevel> IndexMut<usize> for PageTable<Level> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}


impl<Level: PageLevel> PageTable<Level> {
    pub fn zero_out(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}
impl<Level: PageLevelParent> PageTable<Level> {
    
    /// calculates the virtual address of the table at entry ``index``
    /// 
    /// !!! this only works with tables using recursive memory mapping. !!!
    /// 
    pub fn child_table_addr(&self, index: usize) -> Option<usize> {
        //load flags for next table
        let entry_flags = self[index].flags();
        //make sure it actually exists and is 4kb
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE) {
            // get the address of ourself
            let table_address = self as *const Self as usize;
            // remove one layer of recursion and insert index
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    //Return a borrow to the table pointed to at entry ``index``
    pub fn child_table(&self, index: usize) -> Option<&PageTable<Level::ChildLevel>> {
        self.child_table_addr(index).map(|addr| unsafe { &*(addr as *const _)})
    }

    //Return a mutable borrow to the table pointed to at entry ``index``
    pub fn child_table_mut(&mut self, index: usize) -> Option<&mut PageTable<Level::ChildLevel>> {
        self.child_table_addr(index).map(|addr| unsafe { &mut *(addr as *mut _)})
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

impl PageTableEntry {
    fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }
    pub fn pointed_frame(&self) -> Option<MemoryFrame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(MemoryFrame::inside_address(self.page_address()))
        } else {
            None
        }
    }
    #[inline(always)]
    fn page_address(&self) -> PhysicalAddress {
        self.0 & 0x000fffff_fffff000
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn set_unused(&mut self) {
        self.0 = 0;
    }
    fn set(&mut self, frame: MemoryFrame, flags: EntryFlags) {
        //if any of these bits are set, it's an invalid adress.
        assert!(frame.starting_address() & !0x000fffff_fffff000 == 0);
        self.0 = (frame.starting_address() as u64) | flags.bits();
    }
}