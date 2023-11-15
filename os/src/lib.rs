//this program can't use std since it's on bare metal
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#[macro_use]
extern crate bitflags;
extern crate alloc;

use crate::display::{macros::*, DefaultVgaWriter, VgaColorCombo, STATIC_VGA_WRITER, BitmapVgaWriter};

use crate::keyboard::KEYBOARD_QUEUE;
use crate::memory::frame::{FrameRangeInclusive, MemoryFrame};
use crate::memory::paging::EntryFlags;
use crate::memory::populate_global_allocator;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use memory::frame::FrameAllocator;
use memory::paging::{InactivePageTable, MemoryPage, PageTableMaster, TemporaryPage};

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
mod forth;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(info: u64) -> ! {
    disable_cursor();

    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");

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

    let author_text: String = authors.replace(":", " and ");
    let mut formatter = unsafe { DefaultVgaWriter::new_unsafe() };
    formatter
        .clear_screen(display::VgaColor::Black)
        .set_default_colors(VgaColorCombo::on_black(display::VgaColor::Green))
        .write_str("I've succesfully booted, hello world!")
        .next_line();

    formatter
        .next_line()
        .set_default_colors(VgaColorCombo::on_black(display::VgaColor::White))
        .write_str("Version: ")
        .write_str(version)
        .next_line()
        .write_str("Code by: ")
        .write_str(&author_text.as_str())
        .next_line()
        .write_str("CPU vendor: ")
        .write_str(cpu_info.vendor());


    // Start forth application
    
    
/* 
    let mut g_formatter = unsafe { BitmapVgaWriter::new_unsafe() };
    display::switch_graphics_mode();
    let mut i: u8 = 0;
    loop {
        
        

                g_formatter.write_char(i);
                i += 1;
    }
*/
    let alphabet = ('a'..='z').into_iter().chain(('0'..='9').into_iter()).collect::<Vec<char>>();
    unsafe {STATIC_VGA_WRITER.clear_screen(display::VgaColor::Black);
    loop {
        /* for character in &alphabet {
            STATIC_VGA_WRITER.write_str(&format!("Press {}", character));
            let mut chars_i = 0;
            while (chars_i < 2) {
                let tmp = KEYBOARD_QUEUE.dequeue();
                if let Some(c) = tmp {
                    if chars_i == 0 {
                        tmp_write(format!("{}:{:X};",character, c as u8));
                    }
                    chars_i += 1;
                }
            }
        } */
        if let Some(c) = KEYBOARD_QUEUE.dequeue() {
            if c == '\x08' {
                let mut pos = STATIC_VGA_WRITER.get_position();
                if pos.0 > 0 {
                    pos.0 -= 1;
                } else if pos.1 > 0 {
                    pos.1 -= 1;
                    pos.0 = STATIC_VGA_WRITER.get_size().0 - 1;
                }
                STATIC_VGA_WRITER.set_position(pos);
                STATIC_VGA_WRITER.write_raw_char(' ' as u8);
                STATIC_VGA_WRITER.set_position(pos);
            } else {
                STATIC_VGA_WRITER.write_raw_char(c as u8);
            }
            
        }

    }};
}

unsafe fn tmp_write(s: String) {
    for char in s.chars() {
        while ((x86_64::instructions::port::PortReadOnly::<u8>::new(0x3F8 + 5).read() & 0x20) == 0) {};
        PortWriteOnly::new(0x3f8).write(char as u8);
    }
    
    
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
        for i in 0xA0..0xBF {
            let vga_buffer_frame = MemoryFrame::inside_address(i * 0x1000);
            mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, &mut allocator);
        }
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
