use alloc::{boxed::Box, format};

use fs::{AppConstructor, Path, OsHandle};

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
        if let Ok(formatter) = handle.text_mode_formatter() {
            let path = fs::active_directory();
            formatter.next_line().write_str(&format!("Listing Items inside \"{}\"", path.as_str())).next_line();
            match fs::read_dir(path) {
                Ok(dirs) => {
                    for item in dirs.items() {
                        formatter.write_str(item.as_str()).next_line();
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