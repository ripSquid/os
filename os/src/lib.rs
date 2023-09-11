//this program can't use std since it's on bare metal
#![no_std]

use x86_64::instructions::{port::PortWriteOnly, hlt};
use core::arch::asm;
use crate::display::{DefaultVgaBuffer, macros::*, primitives::PrimitiveDisplay};
pub mod display;
mod panic;
use crate::interrupt::setup::setup_interrupt;
mod interrupt;



// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(address: u64) -> ! {
    disable_cursor();

    setup_interrupt(address);
    
    print_str!("Yes?");
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
