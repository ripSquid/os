#![no_std]
#![feature(const_mut_refs)]

pub mod display;
pub mod app;
pub mod forth;
pub mod input;
pub mod pic;
pub use app::*;
use display::{VgaPalette, VgaPaletteColor};
pub use display::macros::*;
extern crate alloc;

pub(crate) fn switch_vga_palette<const N: usize>(palette: VgaPalette<N>) {
    let mut dac_write = x86_64::instructions::port::Port::<u8>::new(0x3C8u16);
    let mut dac_data = x86_64::instructions::port::Port::<u8>::new(0x3C9u16);
    assert!(palette.1 as usize + palette.0.len() <= 256);
    unsafe {
        //Prepare DAC for palette write starting at the color index of the offset

        //Write palette
        dac_write.write(palette.1);
        for color in palette.0.into_iter().map(|c| c.0) {
            for byte in color {
                dac_data.write(byte);
            }
        }
    }
}

pub(crate) fn read_vga_palette() -> VgaPalette<256> {
    let mut dac_write = x86_64::instructions::port::Port::<u8>::new(0x3C8u16);
    let mut dac_data = x86_64::instructions::port::Port::<u8>::new(0x3C9u16);
   
    unsafe {
        dac_write.write(0);
        VgaPalette(core::array::from_fn(|_|VgaPaletteColor(core::array::from_fn(|_| dac_data.read()))), 0)
    }
}