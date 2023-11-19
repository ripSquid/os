use alloc::{boxed::Box, format, vec::Vec};
use base::{forth::ForthMachine, LittleManApp, OsHandle, ProgramError};
use fs::{AppConstructor, DefaultInstall, FileMetadata, Path};

#[derive(Default)]
pub struct Dir;
pub struct DirApp;

impl DefaultInstall for Dir {
    fn path() -> Path {
        Path::from("dir.run")
    }
}
impl AppConstructor for Dir {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(DirApp)
    }
}

impl LittleManApp for DirApp {
    fn run(&mut self, handle: &mut ForthMachine) -> Result<(), ProgramError> {
        const END_BRACKET: u8 = 0xC8;
        const BRACKET: u8 = 0xCC;
        const FOLDER: &[u8] = &[0xC0, 0xC1];
        const RUNNER: &[u8] = &[0xC2, 0xC3];
        const FILE: &[u8] = &[0xC4, 0xC5];

        let path = fs::active_directory();
        handle
            .formatter
            .next_line()
            .write_str("Listing ")
            .write_str(path.as_str())
            .next_line();

        match fs::read_dir(path) {
            Ok(dirs) => {
                let vec: Vec<_> = dirs.items().collect();
                for (i, item) in vec.iter().enumerate() {
                    handle.formatter.write_str("  ");
                    if i == vec.len() - 1 {
                        handle.formatter.write_raw_char(END_BRACKET);
                    } else {
                        handle.formatter.write_raw_char(BRACKET);
                    };
                    let FileMetadata { path, filetype } = item;
                    let icon = match filetype {
                        fs::FileType::Directory => FOLDER,
                        fs::FileType::Data => FILE,
                        fs::FileType::App => RUNNER,
                    };
                    handle
                        .formatter
                        .write_str("[")
                        .write_raw_char(icon[0])
                        .write_raw_char(icon[1])
                        .write_str("] ")
                        .write_str(path.as_str())
                        .next_line();
                }
            }
            Err(error) => {
                handle
                    .formatter
                    .write_str(&format!("CRITICAL ERROR: {error:?}"));
            }
        }
        Ok(())
    }
}
