use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    cell::UnsafeCell,
    mem::size_of,
    ops::RangeInclusive,
    sync::atomic::{AtomicU16, Ordering},
};

use super::{
    frame::{FrameAllocator, MemoryFrame},
    paging::{
        master::{Mapper, PageTableMaster},
        page::{MemoryPage, MemoryPageRange},
        table::EntryFlags,
    },
};

struct GlobalAllocator {
    next: (usize, usize),
    start: Option<&'static mut AllocatorChunk>,
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}
impl AllocatorChunk {
    pub unsafe fn create<A: FrameAllocator>(
        page_table: &mut Mapper,
        allocator: &mut A,
        pages: MemoryPageRange,
    ) -> *mut Self {
        assert!((2..=128).contains(&pages.span()));

        let mut iterator = pages.iter();
        let page = iterator.next().unwrap();
        page_table.map(page, EntryFlags::PRESENT | EntryFlags::WRITABLE, allocator);
        let pointer = page.starting_address() as *mut Self;
        let mut size = 0;

        (*pointer).owners = core::array::from_fn(|_| FrameState::Unused);

        for (i, page) in iterator.enumerate() {
            page_table.map(page, EntryFlags::PRESENT | EntryFlags::WRITABLE, allocator);
            size += 1;
            *(page.starting_address() as *mut RawPageMemory) = [0; 4096];
            let handle = &mut *pointer;
            handle.owners[i] = FrameState::Unallocated(page);
        }
        {
            let handle = &mut *pointer;
            handle.next = 0;
            handle.allocations = 0;
            handle.frames = core::slice::from_raw_parts_mut(
                (pointer as u64 + size_of::<RawPageMemory>() as u64) as *mut RawPageMemory,
                size,
            );
        }
        pointer
    }
    pub fn size_in_pages(&self) -> usize {
        self.frames.len()
    }
}

type RawPageMemory = [u8; 4096];

#[repr(align(4096))]
pub struct AllocatorChunk {
    next: usize,
    allocations: usize,
    frames: &'static mut [RawPageMemory],
    owners: [FrameState; 127],
}

impl AllocatorChunk {
    pub fn allocate(&mut self, layout: Layout) -> Option<*mut u8> {
        self.allocations += 1;
        if self.next >= self.frames.len() {
            return None;
        }
        let owner = self.owners.get_mut(self.next)?;
        match owner.allocate(layout) {
            Some(allocation) => return Some(allocation),
            None => {
                self.next += 1;
                return self.allocate(layout);
            }
        }
    }
    pub fn deallocate(&mut self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
        self.allocations -= 1;
    }
}

struct FrameOwner {
    allocations: u16,
    offset: u16,
    index: u16,
    page: MemoryPage,
}
impl FrameState {
    fn allocate(
        &mut self,
        layout: Layout,
    ) -> Option<*mut u8> {
        match self {
            FrameState::Unallocated(..) | FrameState::Unused => None,
            FrameState::Allocated(owner) => {
                owner.allocations += 1;
                let alloc_start = (owner.offset as usize + layout.align() - 1) % layout.align();
                let alloc_end = alloc_start.saturating_add(layout.size());

                if alloc_end < 4096 {
                    owner.offset = alloc_end as u16;
                    Some((owner.page.starting_address() + alloc_start as u64) as *mut u8)
                } else {
                    None
                }
            }
        }
    }

    #[allow(invalid_reference_casting)]
    unsafe fn deallocate(&mut self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
        match self {
            FrameState::Unallocated(..) | FrameState::Unused => panic!("MAJOR DEALLOCATION ERROR"),
            FrameState::Allocated(owner) => {
                owner.allocations -= 1;
                if owner.allocations == 0 {
                    let state = FrameState::Unallocated(owner.page);
                    *self = state;
                }
            }
        }
    }
}

enum FrameState {
    Unallocated(MemoryPage),
    Allocated(FrameOwner),
    Unused,
}

pub struct AllocatorCell {
    frame: Option<MemoryFrame>,
}
