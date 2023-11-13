use core::{
    alloc::Layout,
    ops::{Index, IndexMut, Range},
};

use x86_64::align_up;

use super::PageState;

use crate::{
    display::{macros::{print_str, debug}, KernelDebug, DEFAULT_VGA_WRITER},
    memory::{paging::MemoryPageRange, MemoryAddress, PAGE_SIZE_4K},
};

/// A Binary tree keeping track of the state of a bunch of memory regions.
/// the pointer must point to an unused piece of memory, that can never be allocated.
pub struct PageStateTree(&'static mut [PageState]);

impl PageStateTree {
    /// Create a new Tree.
    /// This function assumes the pointer points to an unused piece of memory that will never be allocated
    pub unsafe fn new(page_count: usize, start: *mut PageState) -> Self {
        let size = (1 << usize::ilog2(page_count + 1)) - 1;
        let slice = core::slice::from_raw_parts_mut(start, size);
        for state in &mut *slice {
            *state = PageState::default();
        }
        let mut ourself = Self(slice);
        for i in 0..ourself.0.len() {
            let index = TreeIndex(i);
            let size = ourself.size_of(index);
            ourself[index].set_size(size);
        }
        ourself
    }

    /// Generic public allocation
    pub fn allocate(&mut self, layout: Layout, memory_span: &MemoryPageRange) -> Option<*mut u8> {
        self.try_allocate(TreeIndex::root(), layout, memory_span)
    }

    /// Generic public Deallocation
    pub fn unallocate(&mut self, pointer: *mut u8, layout: Layout, memory_span: &MemoryPageRange) {
        let start = pointer as u64 - memory_span.start().starting_address();
        let end = start + layout.size() as u64;
        let self_range = 0..self.size_of(TreeIndex::root()) as u64;
        self.mark_area_unnallocated(start..end, self_range, TreeIndex::root());
    }

    /// Try to allocate a piece of memory
    /// TODO: look for sub levels of the tree for free memory.
    fn try_allocate(
        &mut self,
        index: TreeIndex,
        layout: Layout,
        memory_span: &MemoryPageRange,
    ) -> Option<*mut u8> {
        let addr = self.address_of(index);
        let state = &mut self[index];
        let first = align_up(state.offset(), layout.align() as u64) + addr;
        let last = first + layout.size() as u64;
        if last <= state.size() {
            self.mark_allocated_area_child(first..last, TreeIndex::root());
            Some((self.address_of(index) + memory_span.start().starting_address()) as *mut u8)
        } else {
            print_str!("NO MORE MEMORY!!!!!!");
            None
        }
    }

    //Given a memory span, what region of it does this index reffer to?
    fn address_of(&self, index: TreeIndex) -> MemoryAddress {
        // How many pages does the index span?
        let span = self.size_of(index) as u64;

        // Find How far away this state is from the 0th of this level.
        let state_offset = self.offset(index) as u64;

        //The memory offset compared to the start of the memory range
        let final_offset = state_offset * span;

        final_offset
    }
    fn size_of(&self, index: TreeIndex) -> usize {
        let level = TreeIndex(self.0.len() - 1).level() - index.level();
        2usize.pow(level as u32) * PAGE_SIZE_4K
    }
    //what offset this index has on it's level of the binary tree
    fn offset(&self, index: TreeIndex) -> usize {
        (index.0 + 1) - (1 << (index.0 + 1).ilog2())
    }

    /// Marks an area of the tree as unallocated, which drips down to all levels below
    fn mark_area_unnallocated(
        &mut self,
        range: Range<u64>,
        self_range: Range<u64>,
        index: TreeIndex,
    ) {
        if self_range.contains(&range.start) || self_range.contains(&(range.end)) {
            {
                let state = &mut self[index];
                assert_eq!(self_range.end - self_range.start, state.size());
                state.deallocate_once();
                //debug!(&state.size, &state.offset, &state.allocations, &index);
            }

            let mid_point = self_range.start + (self.size_of(index) as u64 / 2);

            if index.right().0 < self.0.len() {
                self.mark_area_unnallocated(
                    range.clone(),
                    mid_point..self_range.end,
                    index.right(),
                );
            }
            if index.left().0 < self.0.len() {
                self.mark_area_unnallocated(range, self_range.start..mid_point, index.left());
            }
        } else if range.contains(&self_range.start) && range.contains(&(self_range.end)) {
            self.recursive_op(index, &|state: &mut PageState| {
                state.deallocate_whole();
            });
        }
    }
    fn mark_allocated_area_child(&mut self, range: Range<u64>, index: TreeIndex) {
        let (self_range, state) = {
            let base = self.address_of(index);
            let state = &mut self[index];
            (base..base + state.size(), state)
        };

        if self_range.contains(&range.start) || self_range.contains(&(range.end)) {
            state.allocate_once(range.end - self_range.start);
        } else if range.contains(&self_range.start) && range.contains(&(self_range.end)) {
            unsafe {
                DEFAULT_VGA_WRITER.next_line().jump_lines(15).write_debugable(range.start);
            }
            state.allocate_whole();
        } else {
            return;
        }
        if index.right().0 < self.0.len() {
            self.mark_allocated_area_child(range.clone(), index.right());
        }
        if index.left().0 < self.0.len() {
            self.mark_allocated_area_child(range, index.left());
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

#[derive(Clone, Copy, Debug)]
pub struct TreeIndex(usize);

impl<'a> KernelDebug<'a> for TreeIndex {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
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
}
