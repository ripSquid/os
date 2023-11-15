use core::ops::{Deref, DerefMut};

use x86_64::instructions::port::{Port, PortWriteOnly};

use super::{KernelDebug, KernelFormatter, ScreenBuffer};

const VGA_256COLORX_BUFFER_WIDTH: usize = 320;
const VGA_256COLORX_BUFFER_HEIGHT: usize = 200;


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
