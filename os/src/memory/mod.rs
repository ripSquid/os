/*===== WELCOME TO THE MEMORY MODULE OF THE OPERATING SYSTEM! =====*/
/* This is the module containing everything memory management.     */
/*   - The allocator Module contains the memory allocator          */
/*   - The paging module contains low level memory management      */
/*   - The frame module contains the basis for all other modules   */

use self::{
    allocator::GlobalAllocator,
    frame::{FrameRangeInclusive, MemoryFrame},
    paging::PageTableMaster,
};
use crate::{
    display::{macros::{debug, print_str}, DefaultVgaWriter, STATIC_VGA_WRITER},
    multiboot_info::memory_map::{MemoryMapEntry, MemoryType},
};
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec, alloc::Global,
};
use core::{
    alloc::{GlobalAlloc, Layout, Allocator},
    iter::Filter,
    slice::Iter,
    sync::atomic::AtomicUsize,
};

pub mod allocator;
pub mod frame;
pub mod paging;

type MemoryAddress = u64;
type PhysicalAddress = MemoryAddress;
type VirtualAddress = MemoryAddress;

#[global_allocator]
static GLOBAL_ALLOCATOR: GymnasieAllocator = GymnasieAllocator::new();

const PAGE_SIZE_4K: usize = 4096;

// The main allocator of the operating system.
// This is a very barebones (and dangerous) implementation, but allows for future improvements.
// Just like a student in Gymnasiet. (Secondary school)
struct GymnasieAllocator {
    readers: AtomicUsize,
    writers: AtomicUsize,
    actual_allocator: GlobalAllocator,
}
impl GymnasieAllocator {
    //creates a new allocator (done at startup)
    pub const fn new() -> Self {
        Self {
            readers: AtomicUsize::new(0),
            writers: AtomicUsize::new(0),
            actual_allocator: GlobalAllocator::null(),
        }
    }
    //This is the dumbest, and best part of the allocator. Allowing access to the inner part of it.
    unsafe fn write_unchecked(&self) -> &mut GlobalAllocator {
        &mut (self as *const Self as *mut Self)
            .as_mut()
            .unwrap()
            .actual_allocator
    }
}
unsafe impl GlobalAlloc for GymnasieAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.actual_allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.actual_allocator.dealloc(ptr, layout)
    }
}

//Initialize the global allocator, and run a simple test to make sure it allows for basic allocations.
pub unsafe fn populate_global_allocator(
    active_table: &mut PageTableMaster,
    allocator: &mut ElfTrustAllocator,
) {
    GLOBAL_ALLOCATOR.write_unchecked().populate(
        active_table,
        allocator,
        allocator.available_frames_left(),
    );
    allocator_test();
}
unsafe fn allocator_test() {
    let lingering_allocation = Box::new(0x7E57135_u64);
    const MEM_TEST_ITER: usize = 10000;
    for i in 0..MEM_TEST_ITER {
        STATIC_VGA_WRITER.write_horizontally_centerd(&format!("Iteration: {}", i), 0);
        let progress_bar = {
            let width = 30;
            let progress = i * width / MEM_TEST_ITER;
            let string: String = (0..width).into_iter().map(|num| if num > progress {"."} else {"O"}).collect();
            string
        };
        STATIC_VGA_WRITER.write_horizontally_centerd(&progress_bar, 2);
        
        let mut vec: Vec<u64> = Vec::with_capacity(i);
        vec.push(0);
    }
    debug!(&format!("were just doing {lingering_allocation}").as_str())
}

// This is the allocator initially used at startup
// It trusts the information given to us by GRUB/The multiboot information structure on startup,
// and is the basis for all known memory on the computer.
pub struct ElfTrustAllocator {
    next_free_frame: MemoryFrame,
    available_frames: usize,
    active_area: Option<&'static MemoryMapEntry>,
    areas: MemoryAreaIter,
    multiboot: FrameRangeInclusive,
    kernel: FrameRangeInclusive,
}
/// An Iterator over the computers memory areas
#[derive(Clone)]
pub struct MemoryAreaIter {
    itera: Filter<Iter<'static, MemoryMapEntry>, &'static dyn Fn(&&MemoryMapEntry) -> bool>,
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
