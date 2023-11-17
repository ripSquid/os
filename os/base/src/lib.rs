#![no_std]
#![feature(const_mut_refs)]

use x86_64::instructions::port::PortWriteOnly;
pub mod display;
pub use display::macros::*;

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
        PortWriteOnly::new(0x03D5_u16).write(0x20_u8);
    }
}