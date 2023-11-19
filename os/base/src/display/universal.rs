use core::ops::{Deref, DerefMut};

use super::{DefaultVgaWriter, BitmapVgaWriter, VgaPalette, switch_graphics_mode, VgaModeSwitch};


#[derive(Clone, Copy)]
enum CurrentBufferType {
    TextMode80x25,
    VideoMode320x200,
}

pub struct UniversalVgaFormatter {
    default: DefaultVgaWriter,
    graphics: BitmapVgaWriter,
    current: CurrentBufferType,
    palette_storage: Option<VgaPalette<256>>,
}
impl UniversalVgaFormatter {
    pub fn new_unsafe() -> Self {
        Self::new(unsafe {
            DefaultVgaWriter::new_unsafe()
        })
    }
    pub fn new(default: DefaultVgaWriter) -> Self {
        let graphics = unsafe { BitmapVgaWriter::new_unsafe() };
        let current = CurrentBufferType::TextMode80x25;
        Self { default, graphics, current, palette_storage: None }
    }
    pub fn switch_to_text_mode(&mut self) -> &mut DefaultVgaWriter {
        match self.current {
            CurrentBufferType::TextMode80x25 => &mut self.default,
            CurrentBufferType::VideoMode320x200 => {
                let old_paletter = self.palette_storage.replace(self.default.read_palette());
                switch_graphics_mode(VgaModeSwitch::VGA_80X25_TEXT);
                if let Some(palette) = old_paletter {
                    self.default.set_palette(palette);
                }
                self.current = CurrentBufferType::TextMode80x25;
                &mut self.default
            },
        }
    }
    pub fn switch_to_graphics_mode(&mut self) -> &mut BitmapVgaWriter {
        match self.current {
            CurrentBufferType::VideoMode320x200 => &mut self.graphics,
            CurrentBufferType::TextMode80x25 => {
                let old_paletter = self.palette_storage.replace(self.default.read_palette());
                switch_graphics_mode(VgaModeSwitch::VGA_320X200_BITMAP_N);
                if let Some(palette) = old_paletter {
                    self.default.set_palette(palette);
                }
                self.current = CurrentBufferType::TextMode80x25;
                &mut self.graphics 
            },
        }
    }
}

impl Deref for UniversalVgaFormatter {
    type Target = DefaultVgaWriter;

    fn deref(&self) -> &Self::Target {
        &self.default
    }
}
impl DerefMut for UniversalVgaFormatter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.switch_to_text_mode()
    }
}