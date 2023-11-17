use alloc::boxed::Box;

use base::display::VgaColor;
use fs::{Path, AppConstructor, OsHandle};
use fs::apps::{DefaultInstall, LittleManApp};

#[derive(Default)]
pub struct ClearScreen;
pub struct ClearScreenApp;

impl DefaultInstall for ClearScreen {
    fn path() -> Path {
        Path::from("cls")
    }
}
impl AppConstructor for ClearScreen {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ClearScreenApp)
    }
}
impl LittleManApp for ClearScreenApp {
    fn update(&mut self, handle: &mut OsHandle) {
        if let Ok(formatter) = handle.text_mode_formatter() {
            formatter.clear_screen(VgaColor::Black);
        }
        handle.call_exit();
    }
}