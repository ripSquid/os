use core::alloc::{GlobalAlloc, Layout};
mod frame;
mod paging;

type MemoryAddress = u64;

type PhysicalAddress = MemoryAddress;
type VirtualAddress = MemoryAddress;

const PAGE_SIZE_4K: usize = 4096;

#[global_allocator]
static GLOBAL_ALLOCATOR: GymnasieAllocator = GymnasieAllocator;
struct GymnasieAllocator;

unsafe impl GlobalAlloc for GymnasieAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}
