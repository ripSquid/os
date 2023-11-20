use alloc::{boxed::Box, string::String};
use base::{
    display::{VgaColor, VgaColorCombo},
    input::KEYBOARD_QUEUE,
    LittleManApp,
};
use fs::{AppConstructor, DefaultInstall, PathString};

use crate::{
    cpuid, 
    display::{
        restore_text_mode_font,  VgaPalette,
        VgaPaletteColor, 
    },
    interrupt::setup::global_os_time,
};

#[derive(Default)]
pub struct SplashScreen;

impl AppConstructor for SplashScreen {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(Self)
    }
}
impl DefaultInstall for SplashScreen {
    fn path() -> PathString {
        PathString::from("/bin/splash.run")
    }
}

impl LittleManApp for SplashScreen {
    fn run(&mut self, args: &mut base::forth::ForthMachine) -> Result<(), base::ProgramError> {
        let mut skipped = false;
        let timestamp = unsafe { global_os_time };
        let g_formatter = args.formatter.switch_to_graphics_mode();
        let mut fade = 0u8;
        let lars = include_bytes!("LarsKagg2.bmp");
        let width = i32::from_le_bytes(core::array::from_fn(|i| lars[i + 0x12]));
        let height = i32::from_le_bytes(core::array::from_fn(|i| lars[i + 0x16]));
        let palette = VgaPalette::from_array(core::array::from_fn(|i| {
            let chunk = &lars[0x36 + (i * 4)..0x36 + ((i + 1) * 4)];
            VgaPaletteColor::from_rgb(chunk[2], chunk[1], chunk[0])
        }));
        g_formatter.set_palette(VgaPalette::ALL_BLACK);
        let x_pos = (320 - width) / 2;
        for i in 0..height {
            for j in 0..width {
                let (x, y) = (
                    x_pos as usize + j as usize,
                    height as usize - i as usize - 1,
                );
                let char = lars[0x436 + (j as usize + (i as usize * width as usize))];
                g_formatter.plot_pixel(x, y, char);
            }
        }
        let duration = 3000;
        let total_range = 0..duration;
        let visible_range = 1000..2000;
        while unsafe { global_os_time } < timestamp + duration {
            if let Some('w') = unsafe { KEYBOARD_QUEUE.try_getch() } {
                skipped = true;
                break;
            }
            let time = unsafe { global_os_time / 10 } as u8;
            for line in 196..200 {
                g_formatter.set_position((0, line));
                for i in 0..160u16 {
                    g_formatter.write_char(time + i as u8);
                    g_formatter.write_char(time + i as u8);
                }
            }

            let old_fade = fade;
            let time = unsafe { global_os_time } - timestamp;
            if !visible_range.contains(&time) {
                if time < visible_range.start {
                    fade = ((time - total_range.start) * u8::MAX as u64
                        / (visible_range.start - total_range.start))
                        as u8;
                }
                if time >= visible_range.end {
                    fade = u8::MAX
                        - ((time - visible_range.end) * u8::MAX as u64
                            / (total_range.end - visible_range.end))
                            as u8;
                }
            }
            if old_fade != fade {
                g_formatter.set_palette(palette.fade_factor(fade))
            }
        }

        let text_fm = args.formatter.switch_to_text_mode().disable_cursor();
        
        let cpu_info = cpuid::ProcessorIdentification::gather();
        let version = env!("CARGO_PKG_VERSION");
        let authors = env!("CARGO_PKG_AUTHORS");
        let author_text: String = authors.replace(":", " and ");

        unsafe {
            restore_text_mode_font();
        }

        text_fm
            .clear_screen(VgaColor::Black)
            .set_default_colors(VgaColorCombo::on_black(VgaColor::Green))
            .write_str("I've succesfully booted, hello world!")
            .next_line();

        text_fm
            .next_line()
            .set_default_colors(VgaColorCombo::on_black(VgaColor::White))
            .write_str("Version: ")
            .write_str(version)
            .next_line()
            .write_str("Code by: ")
            .write_str(&author_text.as_str())
            .next_line()
            .write_str("CPU vendor: ")
            .write_str(cpu_info.vendor())
            .next_line()
            .next_line()
            .write_str("Skriv [ \"help\" run ] för en introduktion till OperativSystemet")
            .set_position((0,7));

        if !skipped {
            let timestamp = unsafe { global_os_time };
            let duration = 500;

            let mut fade = 0;
            while (unsafe { global_os_time } < timestamp + duration) {
                let time = unsafe { global_os_time } - timestamp;
                let old_fade = fade;
                fade = ((time * u8::MAX as u64) / duration) as u8;
                if fade != old_fade {
                    text_fm.set_palette(VgaPalette::<32>::DEFAULT_TEXTMODE.fade_factor(fade));
                }
            }
        }
        text_fm.set_palette(VgaPalette::<32>::DEFAULT_TEXTMODE);
        Ok(())
    }
}
