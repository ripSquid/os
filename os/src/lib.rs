

//this program can't use std since it's on bare metal
#![no_std]

use core::panic::PanicInfo;

use x86_64::instructions::{port::PortWriteOnly, hlt};

use display::{ScreenBuffer, VgaChar};
mod display;

// Address of the default 80x25 vga text mode buffer left to us after grub.
const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start() -> ! {
    disable_cursor();

    let mut writer = display::DefaultVgaWriter::new(unsafe {
        &mut *(VGA_BUFFER_ADDRESS as *mut ScreenBuffer<VgaChar>)
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut writer = display::DefaultVgaWriter::new(unsafe {&mut *(VGA_BUFFER_ADDRESS as *mut ScreenBuffer<VgaChar>)
    });

    writer.write_str("PANIC xD   ");


    loop {}
}
