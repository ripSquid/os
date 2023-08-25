

//this program can't use std since it's on bare metal
#![no_std]



use x86_64::instructions::{port::PortWriteOnly, hlt};



use crate::display::DefaultVgaBuffer;
pub mod display;
mod panic;
// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start() -> ! {
    disable_cursor();

    let mut writer = display::DefaultVgaWriter::new(unsafe {
        &mut *(VGA_BUFFER_ADDRESS as *mut DefaultVgaBuffer)
    });
    writer.write_str("Hello World                                                                                                                                          test");
    
    hlt();

    panic!();
}

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4 as u16).write(0x0A as u8);
        PortWriteOnly::new(0x03D5 as u16).write(0x20 as u8);
    }
}