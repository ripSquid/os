//this program can't use std since it's on bare metal
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(const_mut_refs)]
#[macro_use]
extern crate bitflags;
extern crate alloc;

use core::arch::asm;

use crate::display::macros::*;

use crate::memory::frame::{FrameRangeInclusive, MemoryFrame};
use crate::memory::paging::table::EntryFlags;
use crate::memory::populate_global_allocator;

use memory::allocator::AllocatorChunk;
use memory::frame::FrameAllocator;
use memory::paging::master::{InactivePageTable, PageTableMaster};
use memory::paging::page::{MemoryPage, MemoryPageRange};
use memory::paging::temporary::TemporaryPage;
use memory::ElfTrustAllocator;
use multiboot_info::elf::ElfSectionFlags;
use multiboot_info::{MultiBootTag, MultibootInfoUnparsed, TagType};
use x86_64::instructions::port::PortWriteOnly;
use x86_64::registers::control::{Cr0, Cr0Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};

pub mod cpuid;
pub mod display;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;
mod interrupt;
mod keyboard;
mod memory;
mod multiboot_info;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(info: u64) -> ! {
    disable_cursor();

    print_str!("hello world");
    let multiboot_info = unsafe {
        multiboot_info::MultibootInfoUnparsed::from_pointer(info as *const MultibootInfoHeader)
    }
    .unwrap();
    let mut active_table = unsafe { PageTableMaster::new() };
    let mut allocator = remap_everything(multiboot_info, &mut active_table);
    unsafe {
        populate_global_allocator(&mut active_table, &mut allocator);
    }

    unsafe { interrupt::setup::setup_interrupts() }
    x86_64::instructions::interrupts::int3();
    let cpu_info = cpuid::ProcessorIdentification::gather();
    debug!(&cpu_info);
    print_str!("everything went well");
    loop {}
}

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
        PortWriteOnly::new(0x03D5_u16).write(0x20_u8);
    }
}

#[no_mangle]
pub extern "C" fn keyboard_handler() {
    print_str!("Interrupt Keyboard");
    panic!();
}

unsafe fn reserve_memory(active_table: &mut PageTableMaster, allocator: &mut ElfTrustAllocator) {}

fn remap_everything(
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
            debug!(
                "mapping section at addr:",
                &section.sh_addr, ", size:", &section.sh_size
            );
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
        let vga_buffer_frame = MemoryFrame::inside_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, &mut allocator);
    });

    let _old_table = active_table.switch(new_table);
    debug!(
        "available memory frames after remap:",
        &allocator.available_frames_left()
    );
    //let old_p4_page = MemoryPage::inside_address(old_table.as_address());
    //active_table.unmap(old_p4_page, &mut allocator);
    print_str!("PAGE TABLE SWITCH SUCCESFUL!");
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
