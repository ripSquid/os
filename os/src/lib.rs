//this program can't use std since it's on bare metal
#![no_std]

use core::panic::PanicInfo;

use display::{ScreenBuffer, VgaChar};
mod display;

// Address of the default 80x25 vga text mode buffer left to us after grub.
const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start() -> ! {
    let mut writer = display::DefaultVgaWriter::new(unsafe {
        &mut *(VGA_BUFFER_ADDRESS as *mut ScreenBuffer<VgaChar>)
    });
    writer.write_str("Hello World");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
