use core::ops::{Deref, DerefMut};

use x86_64::instructions::port::{Port, PortWriteOnly};

use super::{KernelDebug, KernelFormatter, ScreenBuffer};

const VGA_256COLORX_BUFFER_WIDTH: usize = 320;
const VGA_256COLORX_BUFFER_HEIGHT: usize = 200;

#[derive(Clone, Copy)]
pub struct VgaPaletteColor([u8; 3]);

impl VgaPaletteColor {
    pub const BLACK: Self =         Self::from_rgb(0, 0, 0);
    pub const BLUE: Self =          Self::from_rgb(255, 0, 0);
    pub const GREEN: Self =         Self::from_rgb(0, 255, 0);
    pub const CYAN: Self =          Self::from_rgb(255, 0, 255);
    pub const RED: Self =           Self::from_rgb(0,0,255);
    pub const MAGENTA: Self =       Self::from_rgb(255,0,128);
    pub const BROWN: Self =         Self::from_rgb(0,30,60);
    pub const LIGHTGRAY: Self =     Self::from_rgb(200,200,200);
    pub const DARKGRAY: Self =      Self::from_rgb(40,40,40);
    pub const LIGHTBLUE: Self =     Self::from_rgb(255,80,80);
    pub const LIGHTGREEN: Self =    Self::from_rgb(80,255,80);
    pub const LIGHTCYAN: Self =     Self::from_rgb(255,80,255);
    pub const LIGHTRED: Self =      Self::from_rgb(80,80,255);
    pub const PINK: Self =          Self::from_rgb(100,100,255);
    pub const YELLOW: Self =        Self::from_rgb(0, 255, 255);
    pub const WHITE: Self =         Self::from_rgb(255, 255, 255);



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
        let Self([r, g, b]) = self;
        Self([
            ((r as u16) * (factor as u16) / (u8::MAX as u16)) as u8,
            ((g as u16) * (factor as u16) / (u8::MAX as u16)) as u8,
            ((b as u16) * (factor as u16) / (u8::MAX as u16)) as u8,
        ])
    }
}

#[derive(Clone)]
pub struct VgaPalette<const N: usize>([VgaPaletteColor; N], u8);

impl VgaPalette<256> {
    
    pub const ALL_BLACK: Self = Self([VgaPaletteColor::BLACK; 256], 0);
    pub fn greys() -> Self {
        Self(core::array::from_fn(VgaPaletteColor::from_grey_usize), 0)
    }
    pub fn from_array(array: [VgaPaletteColor; 256]) -> Self {
        Self(array, 0)
    }
    pub fn fade_factor(&self, factor: u8) -> Self {
        Self(core::array::from_fn(|i| self.0[i].fade(factor)), 0)
    }
}
impl<const N: usize> VgaPalette<N> {
    pub fn from_array_offset(array: [VgaPaletteColor; N], offset: u8) -> Self {
        Self(array, offset)
    }
    pub const DEFAULT_TEXTMODE: VgaPalette<32> = {
        VgaPalette([
            VgaPaletteColor::BLACK,
            VgaPaletteColor::BLACK,
            VgaPaletteColor::BLUE,
            VgaPaletteColor::BLUE,
            VgaPaletteColor::GREEN,
            VgaPaletteColor::GREEN,
            VgaPaletteColor::CYAN,
            VgaPaletteColor::CYAN,
            VgaPaletteColor::RED,
            VgaPaletteColor::RED,
            VgaPaletteColor::MAGENTA,
            VgaPaletteColor::MAGENTA,
            VgaPaletteColor::BROWN,
            VgaPaletteColor::BROWN,
            VgaPaletteColor::LIGHTGRAY,
            VgaPaletteColor::LIGHTGRAY,
            VgaPaletteColor::DARKGRAY,
            VgaPaletteColor::DARKGRAY,
            VgaPaletteColor::LIGHTBLUE,
            VgaPaletteColor::LIGHTBLUE,
            VgaPaletteColor::LIGHTGREEN,
            VgaPaletteColor::LIGHTGREEN,
            VgaPaletteColor::LIGHTCYAN,
            VgaPaletteColor::LIGHTCYAN,
            VgaPaletteColor::LIGHTRED,
            VgaPaletteColor::LIGHTRED,
            VgaPaletteColor::PINK,
            VgaPaletteColor::PINK,
            VgaPaletteColor::YELLOW,
            VgaPaletteColor::YELLOW,
            VgaPaletteColor::WHITE,
            VgaPaletteColor::WHITE,
            
            ], 0)
    };
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
    pub fn set_palette<const N: usize>(&mut self, palette: VgaPalette<N>) {
        let mut DAC_WRITE = x86_64::instructions::port::Port::<u8>::new(0x3C8u16);
        let mut DAC_DATA = x86_64::instructions::port::Port::<u8>::new(0x3C9u16);
        assert!(palette.1 as usize + palette.0.len() <= 256);
        unsafe {
            //Prepare DAC for palette write starting at color index 0
            DAC_WRITE.write(palette.1);
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
