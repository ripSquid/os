use core::{
    alloc::{Layout, GlobalAlloc},
    mem::size_of,
    ops::{Index, IndexMut, Range},
};

use x86_64::{align_up, registers::debug};

use crate::{memory::{
    paging::{page::{MemoryPage, MemoryPageRange}, master::PageTableMaster, table::EntryFlags},
    MemoryAddress, PAGE_SIZE_4K, ElfTrustAllocator, allocator::RawPageMemory,
}, display::{macros::{debug, print_str}, KernelDebug}};

pub struct PageState {
    size: u64,
    offset: u64,
    allocations: u64,
    _padding: u64,
}
impl PageState {
    pub fn is_null(&self) -> bool {
        self.size == 0
    }
}

const STATES_PER_PAGE: usize = PAGE_SIZE_4K / size_of::<PageState>();

pub struct BigManAllocator {
    range: Option<MemoryPageRange>,
    tree: Option<PageStateTree>,
}
pub struct PageStateTree(&'static mut [PageState]);

#[derive(Clone, Copy, Debug)]
pub struct TreeIndex(usize);

impl<'a> KernelDebug<'a> for TreeIndex {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter.debug_num(self.0)
    }
}
impl TreeIndex {
    pub const fn level(self) -> usize {
        (usize::BITS - (self.0 + 1).leading_zeros()) as usize
    }
    pub fn root() -> Self {
        Self(0)
    }
    pub const fn left(&self) -> Self {
        Self(self.0 * 2 + 1)
    }
    pub const fn right(&self) -> Self {
        Self(self.0 * 2 + 2)
    }
    pub const fn is_left(&self) -> bool {
        self.0 % 2 != 0
    }
    pub const fn is_right(&self) -> bool {
        self.0 % 2 == 0
    }
    pub const fn parent(&self) -> Option<Self> {
        match self.0 > 0 {
            true => Some(Self((self.0 - 1) / 2)),
            false => None,
        }
    }
}

impl Index<TreeIndex> for PageStateTree {
    type Output = PageState;

    fn index(&self, index: TreeIndex) -> &Self::Output {
        &self.0[index.0]
    }
}
impl IndexMut<TreeIndex> for PageStateTree {
    fn index_mut(&mut self, index: TreeIndex) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}
impl BigManAllocator {
    pub const fn uninitialized() -> Self {
        let range = None;
        let tree = None;
        Self { range, tree }
    }
    pub unsafe fn begin(range: MemoryPageRange, page_table: &mut PageTableMaster, allocator: &mut ElfTrustAllocator) -> Self {
        {
            assert!(range.span() > 1);
            for page in range.iter() {
                page_table.map(page, EntryFlags::PRESENT | EntryFlags::WRITABLE, allocator);
                *(page.starting_address() as *mut RawPageMemory) = [0; 4096];
            }
        }
        let (free_page_count, state_page) = {
            let span = range.span();
            let pages_for_state = ((span * 2) - 1) / STATES_PER_PAGE;
            let free_span = span - pages_for_state - 1;
            let state_page = {
                MemoryPage::inside_address(range.start().starting_address() + (free_span * PAGE_SIZE_4K) as u64)
            };
            (free_span, state_page)
        };
        let range = Some(range);
        let tree = Some(unsafe {
            PageStateTree::new(
                free_page_count,
                state_page.starting_address() as *mut PageState,
            )
        });
        Self { range, tree }
    }
    pub fn deallocate_mut(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(tree) = self.tree.as_mut() {
            tree.unallocate(ptr, layout, self.range.as_ref().unwrap())
        } else {
            panic!();
        };
    }
    pub fn allocate_mut(&mut self, layout: Layout) -> Option<*mut u8> {
        if let Some(tree) = self.tree.as_mut() {
            tree.allocate(layout, self.range.as_ref().unwrap())
        } else {
            None
        }
    }
}


impl PageStateTree {
    unsafe fn new(page_count: usize, start: *mut PageState) -> Self {
        let size = (1 << usize::ilog2(page_count + 1)) - 1;
        let slice = core::slice::from_raw_parts_mut(start, size);
        for state in &mut *slice {
            state.allocations = 0;
            state.size = 0;
            state.offset = 0;
            state._padding = 0;
        }
        let mut ourself = Self(slice);
        for i in 0..ourself.0.len() {
            let index = TreeIndex(i);
            ourself[index].size = ourself.size_of(index) as u64;
        }
        ourself
    }
    pub fn allocate(&mut self, layout: Layout, memory_span: &MemoryPageRange) -> Option<*mut u8> {
        self.try_allocate(TreeIndex::root(), layout, memory_span)
    }
    pub fn unallocate(&mut self, pointer: *mut u8, layout: Layout, memory_span: &MemoryPageRange) -> Result<(), ()> {
        let start = pointer as u64 - memory_span.start().starting_address();
        let end = start + layout.size() as u64;
        let self_range = 0..self.size_of(TreeIndex::root()) as u64;
        self.mark_area_unnallocated(start..end, self_range, TreeIndex::root());
        Ok(())
    }
    fn try_allocate(&mut self, index: TreeIndex, layout: Layout, memory_span: &MemoryPageRange) -> Option<*mut u8> {
        let addr = self.address_of(index, memory_span);
        let state = &mut self[index];
        //debug!(&state.offset);
        let first = align_up(state.offset, layout.align() as u64) + addr - memory_span.start().starting_address();
        //debug!(&first);
        let last = first + layout.size() as u64;
        if last <= state.size {
            self.mark_allocated_area(first..last, true);
            Some(self.address_of(index, memory_span) as *mut u8)
        } else {
            print_str!("NO MORE MEMORY!!!!!!");
            None
        }
    }

    //Given a memory span, what region of it does this index reffer to?
    fn address_of(&self, index: TreeIndex, memory_span: &MemoryPageRange) -> MemoryAddress {
        // How many pages does the index span?
        let span = self.size_of(index) as u64;

        // Find How far away this state is from the 0th of this level.
        let state_offset = self.offset(index) as u64;

        //The memory offset compared to the start of the memory range
        let final_offset = state_offset * span;

        assert!(final_offset + span <= memory_span.end().starting_address());
        
        memory_span.start().starting_address() + final_offset
    }
    fn size_of(&self, index: TreeIndex) -> usize {
        let level = TreeIndex(self.0.len()-1).level() - index.level();
        2usize.pow(level as u32) * PAGE_SIZE_4K
    }
    //what offset this index has on it's level of the binary tree
    fn offset(&self, index: TreeIndex) -> usize {
        (index.0+1) - (1 << (index.0 + 1).ilog2())
    }
    pub fn mark_allocated_area(&mut self, range: Range<u64>, allocate: bool) {
        let full_range = 0..self.size_of(TreeIndex::root()) as u64;
        if allocate { 
            self.mark_allocated_area_child(range, full_range, TreeIndex::root());
        } else {

        }
    }
    fn mark_area_unnallocated(&mut self, range: Range<u64>, self_range: Range<u64>, index: TreeIndex) {
        if self_range.contains(&range.start) || self_range.contains(&range.end) {
            {
                let state = &mut self[index];
                assert_eq!(self_range.end - self_range.start, state.size);

                state.allocations -= 1;
                if state.allocations == 0 {
                    state.offset = 0;
                }
                //debug!(&state.size, &state.offset, &state.allocations, &index);
            }
            
            let mid_point = self_range.start + (self.size_of(index) as u64 / 2);

            if index.right().0 < self.0.len() {
                self.mark_area_unnallocated(range.clone(), mid_point..self_range.end, index.right());
            }
            if index.left().0 < self.0.len() {
                self.mark_area_unnallocated(range, self_range.start..mid_point, index.left());
            }
            
        } else if range.contains(&self_range.start) && range.contains(&self_range.end) {
            self.recursive_op(index, &|state: &mut PageState| {
                assert!(state.allocations == 1);
                state.offset = 0;
                state.allocations -= 1;
            });
        }
    }
    fn mark_allocated_area_child(
        &mut self,
        range: Range<u64>,
        self_range: Range<u64>,
        index: TreeIndex,
    ) {
        let mid_point = self_range.start + (self.size_of(index) as u64 / 2);
        if self_range.contains(&range.start) || self_range.contains(&range.end) {
            
                let state = &mut self[index];
                assert_eq!(self_range.end - self_range.start, state.size);

                state.allocations += 1;
                state.offset = (range.end-self_range.start).min(state.size);
                //debug!(&state.size, &state.offset, &state.allocations, &index);
            
        } else if range.contains(&self_range.start) && range.contains(&self_range.end) {
            debug!("hit recursive op", &index);
            let state = &mut self[index];
            assert!(state.allocations == 0);
            state.offset = state.size;
            state.allocations += 1;    
        }
        if index.right().0 < self.0.len() {
            self.mark_allocated_area_child(range.clone(), mid_point..self_range.end, index.right());
        }
        if index.left().0 < self.0.len() {
            self.mark_allocated_area_child(range, self_range.start..mid_point, index.left());
        }
        
        
    }
    fn recursive_op(&mut self, start: TreeIndex, op: &dyn Fn(&mut PageState)) {
        let state = &mut self[start];
        op(state);
        if start.right().0 < self.0.len() {
            self.recursive_op(start.right(), &op);
        }
        if start.left().0 < self.0.len() {
            self.recursive_op(start.left(), &op);
        }
    }
}
