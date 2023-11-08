use core::{mem::size_of, ops::{Index, IndexMut, Range}, alloc::Layout};

use crate::memory::{MemoryAddress, paging::page::{MemoryPageRange, MemoryPage}, PAGE_SIZE_4K};


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

const STATES_PER_PAGE: usize = 4096 / size_of::<PageState>();

pub struct BigManAllocator {
    range: MemoryPageRange,
    tree: Option<PageStateTree>,

}

pub struct PageStateTree(&'static mut [PageState]);

#[derive(Clone, Copy, Debug)]
pub struct TreeIndex(usize);

impl TreeIndex {
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
        if self.0 > 0 {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
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
    pub fn begin(range: MemoryPageRange) -> Self {
        let (free_page_count, state_page) = {
            let span = range.span();
            let pages_for_state = ((span * 2) - 1) / STATES_PER_PAGE;
            let free_span = span-pages_for_state;
            let state_page = {
                let mut page = range.start();
                page.move_page(free_span as isize);
                page
            };
            (free_span, state_page)
        };
        
        let tree =  Some(unsafe {PageStateTree::new( free_page_count, state_page.starting_address() as *mut PageState )});
        Self { range, tree }
    }
}

#[test]
unsafe fn test() {
    const PAGE_COUNT: usize = 40;
    let storage = [[0u8; 4096]; PAGE_COUNT];
    let addr = &storage as *const _ as u64;
    let alloc = BigManAllocator::begin(MemoryPageRange::new(MemoryPage::inside_address(addr), MemoryPage::inside_address(addr+(size_of::<[[u8;4096];PAGE_COUNT]>() as u64))));

}

impl PageStateTree {
    unsafe fn new(page_count: usize, start: *mut PageState) -> Self {
        let size = (1 << usize::ilog2(page_count+1)) - 1;
        let slice = core::slice::from_raw_parts_mut(start, page_count);
        for state in &mut *slice {
            state.allocations = 0;
            state.size = 0;
            state.offset = 0;
            state._padding = 0;
        }
        let mut ourself = Self(slice);
        for i in 1..=ourself.0.len() {
            let mut index = TreeIndex(ourself.0.len() - i);
            ourself[index].size = PAGE_SIZE_4K as u64;
            loop {
                if let Some(new_index) = index.parent() {
                    index = new_index;
                    ourself[index].size += PAGE_SIZE_4K as u64;
                } else {
                    break;
                }
            }
        } 
        ourself
        
        
    }
    pub fn allocate(&mut self, layout: Layout) -> Option<*mut u8> {
        None
    }
    pub fn mark_allocated_area(&mut self, range: Range<u64>) {
        let full_range = {
            0..self[TreeIndex::root()].size
        };
        self.mark_allocated_area_child(range, full_range, TreeIndex::root());
    }
    fn mark_allocated_area_child(&mut self, range: Range<u64>, self_range: Range<u64>, index: TreeIndex) {
        if self_range.contains(&range.start) || self_range.contains(&range.end) {
            
            let state = &mut self[index];
            assert_eq!(self_range.end - self_range.start, state.size);
            
            state.allocations += 1;
            state.offset = range.end.min(state.size);
            
            let mid_point = self_range.start + (state.size /2);
            self.mark_allocated_area_child(range.clone(), mid_point..self_range.end, index.right());
            self.mark_allocated_area_child(range, self_range.start..mid_point, index.left());

        } else if range.contains(&self_range.start) && range.contains(&self_range.end) {
            self.recursive_op(index, |state| {state.offset = state.size; state.allocations += 1;})
        }
    }
    fn recursive_op<F: Fn(&mut PageState)>(&mut self, start: TreeIndex, op: F) {
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