#![no_main]
#![no_std]


use core::panic::PanicInfo;

#[no_mangle]
pub extern fn rust_start() -> ! {
    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}