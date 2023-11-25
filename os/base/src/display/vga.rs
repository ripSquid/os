use core::ops::{Deref, DerefMut};

use x86_64::instructions::port::{Port, PortWriteOnly};

use super::{KernelDebug, KernelFormatter, ScreenBuffer, VgaPalette};

const DEFAULT_VGA_BUFFER_WIDTH: usize = 80;
const DEFAULT_VGA_BUFFER_HEIGHT: usize = 25;

pub static mut STATIC_VGA_WRITER: StaticVgaWriter = unsafe { StaticVgaWriter::new() };
pub struct StaticVgaWriter(Option<DefaultVgaWriter>);
impl StaticVgaWriter {
    const unsafe fn new() -> Self {
        Self(None)
    }
}
impl Deref for StaticVgaWriter {
    type Target = DefaultVgaWriter;

    fn deref(&self) -> &Self::Target {
        unreachable!("No &self methods exists for default vga writer")
    }
}
impl DerefMut for StaticVgaWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .get_or_insert_with(|| unsafe { DefaultVgaWriter::new_unsafe() })
    }
}

pub type DefaultVgaBuffer =
    ScreenBuffer<VgaChar, DEFAULT_VGA_BUFFER_WIDTH, DEFAULT_VGA_BUFFER_HEIGHT>;
pub struct DefaultVgaWriter {
    buffer: &'static mut DefaultVgaBuffer,
    position: (usize, usize),
    fallback_color: VgaColorCombo,
    cursor: bool,
}

impl DefaultVgaWriter {
    pub fn set_position(&mut self, position: (usize, usize)) -> &mut Self {
        self.position = position;
        self
    }

    pub fn get_position(&mut self) -> (usize, usize) {
        self.position
    }
    pub fn back_up(&mut self, mut steps: usize) -> &mut Self {
        if steps >= self.position.0 {
            steps -= self.position.0;
            self.position.0 = 0;
            self.position.1 -= 1;
        }
        while steps >= self.buffer.width() {
            steps -= self.buffer.width();
            self.position.1 -= 1;
        }
        self.position.0 -= steps;
        self
    }

    pub fn get_size(&mut self) -> (usize, usize) {
        (self.buffer.width(), self.buffer.height())
    }

    pub fn disable_cursor(&mut self) -> &mut Self {
        self.cursor = false;
        unsafe {
            PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
            PortWriteOnly::new(0x03D5_u16).write(0x20_u8);
        }
        self
    }
    pub fn update_cursor(&mut self, x: u8, y: u8) -> &mut Self {
        let pos: u16 = y as u16 * 80 + x as u16;
        unsafe {
            PortWriteOnly::new(0x03D4_u16).write(0x0F_u8);
            PortWriteOnly::new(0x03D5_u16).write(pos as u8 & 0xFF);
            PortWriteOnly::new(0x03D4_u16).write(0x0E_u8);
            PortWriteOnly::new(0x03D5_u16).write((pos >> 8) as u8 & 0xFF);
        }
        self
    }
    pub fn read_palette(&mut self) -> VgaPalette<256> {
        crate::read_vga_palette()
    }
    pub fn enable_cursor(&mut self) -> &mut Self {
        unsafe {
            PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
            let mut d5 = Port::<u8>::new(0x03D5_u16);
            let val = d5.read();
            d5.write(val & 0xC0 | 15);

            PortWriteOnly::new(0x013D4_u16).write(0x0B_u8);
            let val = d5.read();
            d5.write(val & 0xE0 | 13);
        }
        self.cursor = true;
        self
    }
    pub const unsafe fn new_unsafe() -> Self {
        Self::new(&mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer))
    }
    pub const fn new(buffer: &'static mut DefaultVgaBuffer) -> Self {
        Self {
            buffer,
            position: (0, 0),
            fallback_color: VgaColorCombo::new(VgaColor::White, VgaColor::Black),
            cursor: false,
        }
    }
    pub fn set_palette<const N: usize>(&mut self, palette: VgaPalette<N>) {
        crate::switch_vga_palette(palette)
    }
    pub fn write_horizontally_centerd(&mut self, text: &str, line: usize) -> &mut Self {
        self.position.1 = line;
        self.position.0 = (self.buffer.width() - text.len().min(self.buffer.width())) / 2;
        self.write_str(text);
        self
    }
    pub fn set_default_colors(&mut self, color: VgaColorCombo) -> &mut Self {
        self.fallback_color = color;
        self
    }
    pub fn clear_screen(&mut self, color: VgaColor) -> &mut Self {
        let color = VgaColorCombo::new(VgaColor::White, color);
        for line in self.buffer.chars.iter_mut() {
            *line = [VgaChar::BLANK.color(color); DefaultVgaBuffer::BUFFER_WIDTH];
        }
        self.position = (0, 0);
        self
    }
    pub fn write_str<T: AsRef<str>>(&mut self, chars: T) -> &mut Self {
        if self.cursor {
            self.update_cursor(self.position.0 as u8, self.position.1 as u8);
        }
        for byte in chars.as_ref().chars() {
            self.write_unicode_char(byte);
        }
        self
    }
    pub fn write_unicode_char(&mut self, char: char) -> &mut Self {
        let byte = match char {
            'å' => 0x86,
            'ä' => 0x84,
            'ö' => 0x94,
            'Å' => 0x8F,
            'Ä' => 0x8E,
            'Ö' => 0x99,
            _ => char as u8,
        };
        self.write_raw_char(byte);
        self
    }
    pub fn write_debugable<T: for<'a> KernelDebug<'a>>(&mut self, data: T) -> &mut Self {
        data.debug(KernelFormatter::new(self));
        self
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        for byte in bytes {
            self.write_raw_char(*byte);
        }
        self
    }
    pub fn write_raw_char(&mut self, byte: u8) -> &mut Self {
        self.write_char(VgaChar {
            char: byte,
            color: self.fallback_color,
        });
        self
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.write_debugable(byte);
    }
    pub fn next_line(&mut self) -> &mut Self {
        let (col, row) = &mut self.position;
        *col = 0;
        *row += 1;
        self
    }
    pub fn jump_lines(&mut self, count: usize) -> &mut Self {
        let (_col, row) = &mut self.position;
        *row += count;
        self
    }
    pub fn write_char(&mut self, char: VgaChar) {
        match char.char {
            b'\n' => {
                self.next_line();
            }
            _ => {
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
        if self.cursor {
            self.update_cursor(self.position.0 as u8, self.position.1 as u8);
        }
    }
    pub fn prepare_print(&mut self) {
        let last_line = self.buffer.height() - 1;
        self.position = (0, last_line);
        for i in 1..=last_line {
            self.buffer.chars[i - 1] = self.buffer.chars[i]
        }
        self.buffer.chars[last_line] = [VgaChar::BLANK; 80]
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VgaChar {
    char: u8,
    color: VgaColorCombo,
}
impl VgaChar {
    const BLANK: Self = Self {
        char: b' ',
        color: VgaColorCombo(0),
    };
    pub fn color(mut self, color: VgaColorCombo) -> Self {
        self.color = color;
        self
    }
}
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VgaColorCombo(u8);
impl VgaColorCombo {
    pub const fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    pub const fn new(foreground: VgaColor, background: VgaColor) -> Self {
        VgaColorCombo((background as u8) << 4 | (foreground as u8))
    }
    pub const fn on_black(color: VgaColor) -> Self {
        VgaColorCombo::new(color, VgaColor::Black)
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}
