/*===== WELCOME TO THE MEMORY MODULE OF THE OPERATING SYSTEM! =====*/
/* This is the module containing everything memory management.     */
/*   - The allocator Module contains the memory allocator          */
/*   - The paging module contains low level memory management      */
/*   - The frame module contains the basis for all other modules   */

use self::{
    allocator::GlobalAllocator,
    frame::{FrameRangeInclusive, MemoryFrame, FrameAllocator},
    paging::{PageTableMaster, TemporaryPage, MemoryPage, InactivePageTable},
};
use crate::{
    display::STATIC_VGA_WRITER,
    multiboot_info::{memory_map::{MemoryMapEntry, MemoryType}, MultibootInfoUnparsed, MultiBootTag, TagType, elf::ElfSectionFlags}, memory::paging::EntryFlags,
};
use alloc::{boxed::Box, format, string::String, vec::Vec};
use x86_64::registers::{model_specific::{EferFlags, Efer}, control::{Cr0, Cr0Flags}};
use core::{
    alloc::{GlobalAlloc, Layout},
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
    _readers: AtomicUsize,
    _writers: AtomicUsize,
    actual_allocator: GlobalAllocator,
}
impl GymnasieAllocator {
    //creates a new allocator (done at startup)
    pub const fn new() -> Self {
        Self {
            _readers: AtomicUsize::new(0),
            _writers: AtomicUsize::new(0),
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
    if false {
        allocator_test();
    }
}
unsafe fn allocator_test() {
    STATIC_VGA_WRITER.clear_screen(crate::display::VgaColor::Black);
    STATIC_VGA_WRITER.write_horizontally_centerd("Performing a quick memory test...", 2);
    let lingering_allocation = Box::new(0x7E57135_u64);
    const MEM_TEST_ITER: usize = 15000;
    for i in 0..MEM_TEST_ITER {
        STATIC_VGA_WRITER
            .write_horizontally_centerd(&format!("{}% done", i * 100 / MEM_TEST_ITER), 20);
        let progress_bar = {
            let width = 30;
            let progress = i * width / MEM_TEST_ITER;
            let string: String = (0..width)
                .into_iter()
                .map(|num| if num > progress { "." } else { "O" })
                .collect();
            string
        };
        STATIC_VGA_WRITER.write_horizontally_centerd(&progress_bar, 21);

        let mut vec: Vec<u64> = Vec::with_capacity(i);
        vec.push(0);
    }
    STATIC_VGA_WRITER.write_horizontally_centerd(
        &format!("were just doing {lingering_allocation}").as_str(),
        1,
    );
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


pub(crate) fn remap_everything(
    info: MultibootInfoUnparsed,
    active_table: &mut PageTableMaster,
) -> ElfTrustAllocator {
    let MultiBootTag::MemoryMap(memory_tag) = info.find_tag(TagType::MemoryMap).unwrap() else {
        panic!()
    };
    let MultiBootTag::ElfSymbols(elf_tag) = info.find_tag(TagType::ElfSymbol).unwrap() else {
        panic!()
    };
    let multiboot = unsafe { info.frame_range() };
    let kernel = unsafe { elf_tag.frame_range() };
    let multiboot2 = unsafe { info.frame_range() };

    let mut allocator = ElfTrustAllocator::new(kernel, multiboot, memory_tag.area_iter());
    let mut temp_page = TemporaryPage::new(MemoryPage::inside_address(0xefaceea7), &mut allocator);

    let mut new_table = {
        let frame = allocator.allocate_frame().unwrap();
        InactivePageTable::new(frame, active_table, &mut temp_page)
    };
    switch_no_ex();
    switch_write_bit();
    active_table.with(&mut new_table, &mut temp_page, |mapper| {
        //map elf sections
        for section in elf_tag.entries.parsed.iter() {
            if !section.sh_flags.contains(ElfSectionFlags::ALLOCATED) {
                continue;
            }
            assert!(section.sh_addr % 4096 == 0);
            //debug!(
            //    "mapping section at addr:",
            //    &section.sh_addr, ", size:", &section.sh_size
            //);
            let flags = EntryFlags::from_elf_flags(&section.sh_flags);
            let start_frame = MemoryFrame::inside_address(section.sh_addr);
            let end_frame = MemoryFrame::inside_address(section.sh_addr + section.sh_size - 1);
            for frame in FrameRangeInclusive::new(start_frame, end_frame) {
                mapper.identity_map(frame, flags, &mut allocator);
            }
        }
        //map multiboot info
        for frame in multiboot2 {
            mapper.identity_map(frame, EntryFlags::PRESENT, &mut allocator);
        }
        //map vga buffer
        for i in 0xA0..0xBF {
            let vga_buffer_frame = MemoryFrame::inside_address(i * 0x1000);
            mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, &mut allocator);
        }
    });

    let _old_table = active_table.switch(new_table);
    //debug!(
    //    "available memory frames after remap:",
    //    &allocator.available_frames_left()
    //);
    //let old_p4_page = MemoryPage::inside_address(old_table.as_address());
    //active_table.unmap(old_p4_page, &mut allocator);
    //print_str!("PAGE TABLE SWITCH SUCCESFUL!");
    allocator
}

fn switch_no_ex() {
    unsafe {
        let efer = Efer::read();
        Efer::write(efer | EferFlags::NO_EXECUTE_ENABLE);
    }
}

fn switch_write_bit() {
    unsafe {
        let cr0 = Cr0::read();
        Cr0::write(cr0 | Cr0Flags::WRITE_PROTECT);
    }
}
