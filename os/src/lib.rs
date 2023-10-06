//this program can't use std since it's on bare metal
#![no_std]
#![feature(adt_const_params)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate bitflags;



use crate::display::macros::*;

use crate::memory::frame::{FrameRangeInclusive, MemoryFrame};
use crate::memory::paging::table::EntryFlags;

use memory::frame::FrameAllocator;
use memory::paging::master::{InactivePageTable, PageTableMaster};
use memory::paging::page::MemoryPage;
use memory::paging::temporary::TemporaryPage;
use memory::ElfTrustAllocator;
use multiboot_info::elf::ElfSectionFlags;
use multiboot_info::{MultiBootTag, MultibootInfoUnparsed, TagType};
use x86_64::instructions::{port::PortWriteOnly};
use x86_64::registers::control::{Cr0, Cr0Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};

pub mod cpuid;
pub mod display;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;
mod interrupt;
mod memory;
mod multiboot_info;


// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;
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

    remap_everything(multiboot_info);
    unsafe { interrupt::setup::setup_interrupts() }
    x86_64::instructions::interrupts::int3();
    let cpu_info = cpuid::ProcessorIdentification::gather();
    debug!(&cpu_info);
    print_str!("everything went well");
    loop {}
}

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4 as u16).write(0x0A as u8);
        PortWriteOnly::new(0x03D5 as u16).write(0x20 as u8);
    }
}

#[no_mangle]
pub extern "C" fn keyboard_handler() {
    print_str!("Interrupt Keyboard");
    panic!();
}

fn remap_everything(info: MultibootInfoUnparsed) {
    let MultiBootTag::MemoryMap(memory_tag) = info
        .tag_iter()
        .find(|tag| tag.tag_type() == TagType::MemoryMap)
        .unwrap()
    else {
        panic!()
    };
    let MultiBootTag::ElfSymbols(elf_tag) = info
        .tag_iter()
        .find(|tag| tag.tag_type() == TagType::ElfSymbol)
        .unwrap()
    else {
        panic!()
    };
    let multiboot = unsafe { info.frame_range() };
    let kernel = unsafe { elf_tag.frame_range() };
    let multiboot2 = unsafe { info.frame_range() };

    let mut allocator = ElfTrustAllocator::new(kernel, multiboot, memory_tag.area_iter());
    let mut temp_page = TemporaryPage::new(MemoryPage::inside_address(0xcafebabe), &mut allocator);
    let mut active_table = unsafe { PageTableMaster::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().unwrap();
        InactivePageTable::new(frame, &mut active_table, &mut temp_page)
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
                "mapping section at addr: ",
                &section.sh_addr, ", size: ", &section.sh_size
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

    //let old_p4_page = MemoryPage::inside_address(old_table.as_address());
    //active_table.unmap(old_p4_page, &mut allocator);
    print_str!("PAGE TABLE SWITCH SUCCESFUL!");
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
