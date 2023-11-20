use alloc::boxed::Box;

use base::{display::VgaColor, forth::ForthMachine, LittleManApp, OsHandle, ProgramError};
use fs::{AppConstructor, DefaultInstall, PathString};

#[derive(Default)]
pub struct ClearScreen;
pub struct ClearScreenApp;

impl DefaultInstall for ClearScreen {
    fn path() -> PathString {
        PathString::from("cls.run")
    }
}
impl AppConstructor for ClearScreen {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ClearScreenApp)
    }
}
impl LittleManApp for ClearScreenApp {
    fn run(&mut self, handle: &mut ForthMachine) -> Result<(), ProgramError> {
        handle.formatter.clear_screen(VgaColor::Black);
        Ok(())
    }
}
