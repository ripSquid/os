//this program can't use std since it's on bare metal
#![no_std]
#![feature(adt_const_params)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate bitflags;

use core::arch::asm;

use crate::display::macros::*;
use memory::ElfTrustAllocator;
use memory::frame::FrameAllocator;
use multiboot_info::{MultibootInfoUnparsed, TagType, MultiBootTag};
use x86_64::instructions::{hlt, port::PortWriteOnly};
pub mod display;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;
mod interrupt;
mod memory;
mod multiboot_info;
use crate::interrupt::setup::{self, setup_interrupt};

// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(address: u64, info: u64) -> ! {
    disable_cursor();

    print_str!("hello world");
    let multiboot_info = unsafe {
        multiboot_info::MultibootInfoUnparsed::from_pointer(info as *const MultibootInfoHeader)
    }
    .unwrap();

    print_hex!(0xE as u32);

    //hlt();
    //setup_interrupt(address);
    //debug!(&multiboot_info);
    remap_everything(multiboot_info);
    //print_str!("Yes?");
    hlt();
    loop { unsafe {asm!("nop");} }
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
    let MultiBootTag::MemoryMap(memory_tag) = info.tag_iter().find(|tag| tag.tag_type() == TagType::MemoryMap).unwrap() else {panic!()};
    let MultiBootTag::ElfSymbols(elf_tag) = info.tag_iter().find(|tag| tag.tag_type() == TagType::ElfSymbol).unwrap() else {panic!()};
    let multiboot = unsafe {info.frame_range()};
    let kernel = unsafe { elf_tag.frame_range()};
    for area in memory_tag.area_iter() {
        debug!(area);
    }
    let mut allocator = ElfTrustAllocator::new(kernel, multiboot, memory_tag.area_iter());
    for i in 0u32..10u32 {
        print_num!(i);
        let frame = allocator.allocate_frame();
        debug!(&frame);
    }
}

