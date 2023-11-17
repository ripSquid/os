use alloc::{boxed::Box, format};
use fs::{OsHandle, StartError};

use fs::{AppConstructor, FileSystemError, Path};

use super::{LittleManApp, DefaultInstall};

#[derive(Default)]
pub struct ChangeDir;
pub struct ChangeDirApp(Option<Result<FileSystemError, &'static str>>);

impl DefaultInstall for ChangeDir {
    fn path() -> Path {
        Path::from("cd")
    }
}
impl AppConstructor for ChangeDir {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ChangeDirApp(None))
    }
}

impl LittleManApp for ChangeDirApp {
    fn start(&mut self, args: &[&str]) -> Result<(), StartError> {
        if args.is_empty() {
            self.0 = Some(Err("Please specify a directory."));
        } else {
            let path = Path::from(args[0]);
            let valid = match fs::get_file_relative(&path) {
                Ok(w) => {
                    let yeah = w.is_directory();
                    if !yeah {
                        self.0 = Some(Err("Specified path is not a directory."));
                    }
                    yeah
                },
                Err(err) => {
                    self.0 = Some(Ok(err));
                    false
                },
            };
            if valid {
                fs::set_active_directory(path);
            }
        }

        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        if let Ok(formatter) = handle.text_mode_formatter() {
            match &self.0 {
                Some(Ok(os_error)) => {formatter.write_str(&format!("Filesystem error response: {os_error:?}"));},
                Some(Err(other_error)) => {formatter.write_str(other_error);},
                None => (),
            }
        }
        handle.call_exit();
    }
}