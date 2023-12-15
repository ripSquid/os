use alloc::{boxed::Box, format};

use base::{
    forth::{self, ForthMachine},
    LittleManApp, OsHandle, ProgramError,
};

use forth::{Stack, StackItem};

use fs::{AppConstructor, DefaultInstall, FileSystemError, PathString};

#[derive(Default)]
pub struct ChangeDir;
pub struct ChangeDirApp(Option<Result<FileSystemError, &'static str>>);

impl DefaultInstall for ChangeDir {
    fn path() -> PathString {
        PathString::from("cd.run")
    }
}
impl AppConstructor for ChangeDir {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ChangeDirApp(None))
    }
}

impl LittleManApp for ChangeDirApp {
    fn run(&mut self, machine: &mut ForthMachine) -> Result<(), ProgramError> {
        let path = {
            match machine.stack.pop() {
                Some(StackItem::String(string)) => {
                    Ok(fs::active_directory().append(&string).clean())
                }
                Some(invalid) => {
                    machine.stack.push(invalid);
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
                    }
                    Err(err) => {
                        self.0 = Some(Ok(err));
                        false
                    }
                };
                if valid {
                    fs::set_active_directory(path);
                }
            }
            Err(error) => self.0 = Some(Err(error)),
        }

        match &self.0 {
            Some(Ok(os_error)) => {
                machine
                    .formatter
                    .write_str(&format!("Filesystem error response: {os_error:?}"));
            }
            Some(Err(other_error)) => {
                machine.formatter.write_str(other_error);
            }
            None => (),
        }

        Ok(())
    }
}
