//this program can't use std since it's on bare metal
#![no_std]

use crate::display::macros::*;
use x86_64::instructions::{hlt, port::PortWriteOnly};
pub mod display;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;
mod interrupt;
mod multiboot_info;

// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(address: u64, info: u64) -> ! {
    disable_cursor();

    print_str!("hello world");
    let multiboot_info =
        unsafe { multiboot_info::MultibootInfo::from_pointer(info as *const MultibootInfoHeader) }
            .unwrap();
    print_str!("12341234123412341234123412341234123412341234123412341234123412341234123412341234");
    debug!(" ", "bajs", "ja");
    multiboot_info.tags.memory_tag();

    //print_hex!(address);
    //print_hex!(info);

    hlt();
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
    print_str!("Interrupt");
}
