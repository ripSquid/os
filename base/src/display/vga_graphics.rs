use super::ScreenBuffer;

const VGA_256COLORX_BUFFER_WIDTH: usize = 320;
const VGA_256COLORX_BUFFER_HEIGHT: usize = 200;

#[derive(Clone, Copy)]
pub struct VgaPaletteColor(pub(crate) [u8; 3]);

impl VgaPaletteColor {
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const CYAN: Self = Self::from_rgb(0, 255, 255);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const MAGENTA: Self = Self::from_rgb(255, 0, 255);
    pub const BROWN: Self = Self::from_rgb(60, 30, 0);
    pub const LIGHTGRAY: Self = Self::from_rgb(150, 150, 150);
    pub const DARKGRAY: Self = Self::from_rgb(40, 40, 40);
    pub const LIGHTBLUE: Self = Self::from_rgb(80, 80, 255);
    pub const LIGHTGREEN: Self = Self::from_rgb(80, 255, 80);
    pub const LIGHTCYAN: Self = Self::from_rgb(80, 255, 255);
    pub const LIGHTRED: Self = Self::from_rgb(255, 80, 80);
    pub const PINK: Self = Self::from_rgb(255, 100, 100);
    pub const YELLOW: Self = Self::from_rgb(255, 255, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);

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
pub struct VgaPalette<const N: usize>(pub(crate) [VgaPaletteColor; N], pub(crate) u8);

impl VgaPalette<256> {
    pub const ALL_BLACK: Self = Self([VgaPaletteColor::BLACK; 256], 0);
    pub fn greys() -> Self {
        Self(core::array::from_fn(VgaPaletteColor::from_grey_usize), 0)
    }
    pub fn from_array(array: [VgaPaletteColor; 256]) -> Self {
        Self(array, 0)
    }
}
impl<const N: usize> VgaPalette<N> {
    pub fn fade_factor(&self, factor: u8) -> Self {
        Self(core::array::from_fn(|i| self.0[i].fade(factor)), 0)
    }
    pub fn from_array_offset(array: [VgaPaletteColor; N], offset: u8) -> Self {
        Self(array, offset)
    }
    pub const DEFAULT_TEXTMODE: VgaPalette<64> = {
        VgaPalette(
            [
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLUE,
                VgaPaletteColor::GREEN,
                VgaPaletteColor::CYAN,
                VgaPaletteColor::RED,
                VgaPaletteColor::MAGENTA,
                VgaPaletteColor::BROWN,
                VgaPaletteColor::LIGHTGRAY,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::BLACK,
                VgaPaletteColor::DARKGRAY,
                VgaPaletteColor::LIGHTBLUE,
                VgaPaletteColor::LIGHTGREEN,
                VgaPaletteColor::LIGHTCYAN,
                VgaPaletteColor::LIGHTRED,
                VgaPaletteColor::PINK,
                VgaPaletteColor::YELLOW,
                VgaPaletteColor::WHITE,
            ],
            0,
        )
    };
}

pub type Vga256ColorXModeBuffer =
    ScreenBuffer<u8, VGA_256COLORX_BUFFER_WIDTH, VGA_256COLORX_BUFFER_HEIGHT>;
pub struct BitmapVgaWriter {
    buffer: &'static mut Vga256ColorXModeBuffer,
    position: (usize, usize),
}

impl BitmapVgaWriter {
    pub fn read_palette(&mut self) -> VgaPalette<256> {
        crate::read_vga_palette()
    }
    pub fn set_position(&mut self, position: (usize, usize)) -> &mut Self {
        self.position = position;
        self
    }
    pub fn set_palette<const N: usize>(&mut self, palette: VgaPalette<N>) {
        crate::switch_vga_palette(palette)
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
    pub fn plot_pixel(&mut self, x: usize, y: usize, byte: u8) -> &mut Self {
        self.buffer.chars[y][x] = byte;
        self
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
