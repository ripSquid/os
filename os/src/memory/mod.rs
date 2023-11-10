use core::{
    alloc::{GlobalAlloc, Layout},
    cell::{RefCell, UnsafeCell},
    iter::Filter,
    sync::atomic::AtomicUsize,
};

use alloc::{string::{String, ToString}, format, boxed::Box};

use crate::{
    display::macros::print_str,
    multiboot_info::memory_map::{MemoryMapEntry, MemoryType},
};

use self::{
    allocator::{GlobalAllocator},
    frame::{FrameRangeInclusive, MemoryFrame},
    paging::master::PageTableMaster,
};
pub mod allocator;
pub mod frame;
pub mod paging;

type MemoryAddress = u64;

type PhysicalAddress = MemoryAddress;
type VirtualAddress = MemoryAddress;

const PAGE_SIZE_4K: usize = 4096;

#[global_allocator]
static GLOBAL_ALLOCATOR: GymnasieAllocator = GymnasieAllocator::new();
struct GymnasieAllocator {
    readers: AtomicUsize,
    writers: AtomicUsize,
    actual_allocator: GlobalAllocator,
}
impl GymnasieAllocator {
    pub const fn new() -> Self {
        Self {
            readers: AtomicUsize::new(0),
            writers: AtomicUsize::new(0),
            actual_allocator: GlobalAllocator::null(),
        }
    }
    fn write_a(&self) -> &mut GlobalAllocator {
        assert!(self.actual_allocator.is_active());
        while self.writers.load(core::sync::atomic::Ordering::Relaxed) != 0 {}
        unsafe { self.write_a_unchecked() }
    }
    unsafe fn write_a_unchecked(&self) -> &mut GlobalAllocator {
        &mut (self as *const Self as *mut Self)
            .as_mut()
            .unwrap()
            .actual_allocator
    }
}
pub unsafe fn populate_global_allocator(
    active_table: &mut PageTableMaster,
    allocator: &mut ElfTrustAllocator,
) {
    GLOBAL_ALLOCATOR
        .write_a_unchecked()
        .populate(active_table, allocator);
    allocator_test();
}

fn allocator_test() {
    let test_string = String::from("this is a heap allocated string!");
    print_str!(&test_string.as_str());
    drop(test_string);
    let pro = 0;
    let test_string_2 = format!("this is a heap allocated string again, using format, have the number {}", 0);
    print_str!(&test_string_2.as_str());
    drop(test_string_2);
    let test_box = Box::new([0i32; 4096]);
}

unsafe impl GlobalAlloc for GymnasieAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.actual_allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.actual_allocator.dealloc(ptr, layout)
    }
}

pub struct ElfTrustAllocator {
    next_free_frame: MemoryFrame,
    available_frames: usize,
    active_area: Option<&'static MemoryMapEntry>,
    areas: MemoryAreaIter,
    multiboot: FrameRangeInclusive,
    kernel: FrameRangeInclusive,
}
#[derive(Clone)]
pub struct MemoryAreaIter {
    itera: Filter<
        core::slice::Iter<'static, MemoryMapEntry>,
        &'static dyn Fn(&&MemoryMapEntry) -> bool,
    >,
}
impl MemoryAreaIter {
    pub fn new(slice: &'static [MemoryMapEntry]) -> Self {
        Self {
            itera: slice
                .iter()
                .filter(&(|i| i.mem_type == MemoryType::Available)),
        }
    }
}
impl Iterator for MemoryAreaIter {
    type Item = &'static MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.itera.next()
    }
}
