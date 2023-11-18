use alloc::{boxed::Box, format};

use base::{forth, LittleManApp, StartError, OsHandle};

use forth::{Stack, StackItem};

use fs::{AppConstructor, FileSystemError, Path, DefaultInstall};



#[derive(Default)]
pub struct ChangeDir;
pub struct ChangeDirApp(Option<Result<FileSystemError, &'static str>>);

impl DefaultInstall for ChangeDir {
    fn path() -> Path {
        Path::from("cd.run")
    }
}
impl AppConstructor for ChangeDir {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ChangeDirApp(None))
    }
}

impl LittleManApp for ChangeDirApp {
    fn start(&mut self, stack: &mut Stack) -> Result<(), StartError> {
        let path = {
            match stack.pop() {
                Some(StackItem::String(string)) => Ok(fs::active_directory().append(&string).clean()),
                Some(invalid) => {
                    stack.push(invalid);
                    Err("invalid stack item")
                }
                None => Err("please specify a directory"),
            }
        };
        match path {
            Ok(path) => {
                let valid = match fs::get_file(&path) {
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
            },
            Err(error) => self.0 = Some(Err(error)),
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