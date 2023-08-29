use super::ScreenBuffer;




const DEFAULT_VGA_BUFFER_WIDTH: usize = 80;
const DEFAULT_VGA_BUFFER_HEIGHT: usize = 25;

pub type DefaultVgaBuffer = ScreenBuffer<VgaChar, DEFAULT_VGA_BUFFER_WIDTH, DEFAULT_VGA_BUFFER_HEIGHT>;
pub struct DefaultVgaWriter {
    buffer: &'static mut DefaultVgaBuffer,
    position: (usize, usize),
    fallback_color: VgaColorCombo,
}

impl DefaultVgaWriter {
    pub fn new(
        buffer: &'static mut DefaultVgaBuffer
    ) -> Self {
        Self {
            buffer,
            position: (0, 0),
            fallback_color: VgaColorCombo::new(VgaColor::White, VgaColor::Black),
        }
    }
    pub fn set_default_colors(&mut self, color: VgaColorCombo) {
        self.fallback_color = color;
    }
    pub fn write_str(&mut self, chars: &str) {
        for byte in chars.bytes() {
            self.write_byte(byte);
        }
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte)
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.write_char(VgaChar { char: byte, color: self.fallback_color })
    }
    pub fn next_line(&mut self) {
        let buffer_height = self.buffer.height();
        let (col, row) = &mut self.position;
        *col = 0;
        if *row < buffer_height {
            *row += 1
        } else {
            *row = 0;
        }
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
    }
    pub fn prepare_print(&mut self) {
        let last_line = self.buffer.height()-1;
        self.position = (0, last_line);
        for i in 1..=last_line {
            self.buffer.chars[i-1] = self.buffer.chars[i]
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
}
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VgaColorCombo(u8);
impl VgaColorCombo {
    pub fn new(foreground: VgaColor, background: VgaColor) -> Self {
        VgaColorCombo((background as u8) << 4 | (foreground as u8))
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
