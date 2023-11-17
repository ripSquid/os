use alloc::{boxed::Box, format, vec::Vec};

use fs::{AppConstructor, Path, OsHandle, FileMetadata};

use super::{LittleManApp, InstallableApp};



pub struct Dir;
pub struct DirApp;

impl InstallableApp for Dir {
    fn install() -> (Path, Box<dyn AppConstructor>) {
       (Path::from("dir"), Box::new(Dir))
    }
}
impl AppConstructor for Dir {
    fn instantiate(&self) -> Box<dyn super::LittleManApp> {
        Box::new(DirApp)
    }
}

impl LittleManApp for DirApp {
    fn update(&mut self, handle: &mut OsHandle) {
        const END_BRACKET: u8 = 0xC8;
        const BRACKET: u8 = 0xCC;
        const FOLDER: &[u8] = &[0xC0, 0xC1];
        const RUNNER: &[u8] = &[0xC2, 0xC3];
        const FILE: &[u8] = &[0xC4, 0xC5];
        if let Ok(formatter) = handle.text_mode_formatter() {
            let path = fs::active_directory();
            formatter.next_line().write_str("Listing ").write_str(path.as_str()).next_line();
            match fs::read_dir(path) {
                Ok(dirs) => {
                    let vec: Vec<_> = dirs.items().collect();
                    for (i, item) in vec.iter().enumerate() {
                        formatter.write_str("  ");
                        if i == vec.len()-1 {
                            formatter.write_raw_char(END_BRACKET);
                        } else {
                            formatter.write_raw_char(BRACKET);
                        };
                        let FileMetadata {
                            path,
                            filetype,
                        } = item;
                        let icon = match filetype {
                            fs::FileType::Directory => FOLDER,
                            fs::FileType::Data => FILE,
                            fs::FileType::App => RUNNER,
                        };
                        formatter.write_str("[").write_raw_char(icon[0]).write_raw_char(icon[1]).write_str("] ").write_str(path.as_str()).next_line();
                    }
                },
                Err(error) => {
                    formatter.write_str(&format!("CRITICAL ERROR: {error:?}"));
                },
            }
        }
        handle.call_exit()
    }
}