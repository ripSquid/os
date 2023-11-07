use core::{mem::size_of, ops::{Index, IndexMut}};

use crate::memory::{MemoryAddress, paging::page::MemoryPageRange};


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