use core::ops::{Deref, DerefMut};

use x86_64::instructions::port::{Port, PortWriteOnly};

use super::{KernelDebug, KernelFormatter, ScreenBuffer};

const VGA_256COLORX_BUFFER_WIDTH: usize = 320;
const VGA_256COLORX_BUFFER_HEIGHT: usize = 200;

#[derive(Clone, Copy)]
pub struct VgaPaletteColor([u8; 3]);

impl VgaPaletteColor {
    pub const BLACK: Self = Self([0; 3]);
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r >> 2, g >> 2, b >> 2])
    }
    pub const fn from_grey(value: u8) -> Self {
        Self([value >> 2; 3])
    }
    pub const fn from_grey_usize(value: usize) -> Self {
        Self::from_grey(value as u8)
    }
    pub fn fade(self, factor: u8) -> Self {
        let Self([r,g,b]) = self;
        Self([
            ((r as u16) * (factor as u16) / (u8::MAX as u16)) as u8, 
            ((g as u16) * (factor as u16) / (u8::MAX as u16)) as u8,
            ((b as u16) * (factor as u16) / (u8::MAX as u16)) as u8,
        ])
    }
}

#[derive(Clone)]
pub struct VgaPalette([VgaPaletteColor; 256]);

impl VgaPalette {
    pub const ALL_BLACK: Self = Self([VgaPaletteColor::BLACK; 256]);
    pub fn greys() -> Self {
        Self(core::array::from_fn(VgaPaletteColor::from_grey_usize))
    }
    pub fn from_array(array: [VgaPaletteColor; 256]) -> Self {
        Self(array)
    }
    pub fn fade_factor(&self, factor: u8) -> Self {
        Self(core::array::from_fn(|i| self.0[i].fade(factor)))
    }
}

pub type Vga256ColorXModeBuffer =
    ScreenBuffer<u8, VGA_256COLORX_BUFFER_WIDTH, VGA_256COLORX_BUFFER_HEIGHT>;
pub struct BitmapVgaWriter {
    buffer: &'static mut Vga256ColorXModeBuffer,
    position: (usize, usize),
}

impl BitmapVgaWriter {
    pub fn set_position(&mut self, position: (usize, usize)) -> &mut Self {
        self.position = position;
        self
    }
    pub fn set_palette(&mut self, palette: VgaPalette) {
        let mut DAC_WRITE = x86_64::instructions::port::Port::<u8>::new(0x3C8u16);
        let mut DAC_DATA = x86_64::instructions::port::Port::<u8>::new(0x3C9u16);
        unsafe {
            //Prepare DAC for palette write starting at color index 0
            DAC_WRITE.write(0);
            //Write palette
            for byte in palette.0.into_iter().flat_map(|c| c.0) {
                DAC_DATA.write(byte);
            }

        }
    }

    pub const unsafe fn new_unsafe() -> Self {
        Self::new(&mut *(0xA0000 as *mut Vga256ColorXModeBuffer))
    }
    pub const fn new(buffer: &'static mut Vga256ColorXModeBuffer) -> Self {
        Self {
            buffer,
            position: (0, 0),
        }
    }
    pub fn next_line(&mut self) -> &mut Self {
        let (col, row) = &mut self.position;
        *col = 0;
        *row += 1;
        self
    }
    pub fn prepare_print(&mut self) {
        self.position.1 = 0;
    }

    pub fn write_char(&mut self, char: u8) {
        if self.position.0 >= self.buffer.width() {
            self.next_line();
        }
        if self.position.1 >= self.buffer.height() {
            self.prepare_print();
        }
        let (col, row) = self.position;
        self.buffer.chars[row][col] = char;
        self.position.0 += 1;
    }
}
