use crate::display::{BitmapVgaWriter, switch_graphics_mode, VgaModeSwitch, VgaPalette, VgaPaletteColor};

pub fn show_lars() {
    let mut g_formatter = unsafe {
        switch_graphics_mode(VgaModeSwitch::VGA_320X200_BITMAP_N);
        BitmapVgaWriter::new_unsafe()
    };
    let lars = include_bytes!("Lars_Kagg.bmp");
    let width = i32::from_le_bytes(core::array::from_fn(|i| lars[i+0x12]));
    let height = i32::from_le_bytes(core::array::from_fn(|i| lars[i+0x16]));
    let palette = VgaPalette::from_array(core::array::from_fn(|i|{
        let chunk = &lars[0x36+(i*4)..0x36+((i+1)*4)];
        VgaPaletteColor::from_rgb(chunk[2], chunk[1], chunk[0])
    }));
    g_formatter.set_palette(palette);
    let x_pos = (320 - width) / 2;
    for i in (0..height) {
        g_formatter.set_position((x_pos as usize, height as usize - i as usize - 1));
        for j in 0..width {
            g_formatter.write_char(lars[0x436+(j as usize+(i as usize*width as usize))]);
        }
    }
}