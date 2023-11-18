use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::{string::String, vec::Vec};

use base::forth::ForthMachine;
use base::{LittleManApp, OsHandle};
use fs::{AppConstructor, DefaultInstall, Path};
use spin::rwlock::RwLock;

#[derive(Default)]
pub struct ForthFile {
    fm: Arc<RwLock<ForthMachine>>
}

impl DefaultInstall for ForthFile {
    fn path() -> Path {
        Path::from("forth.run")
    }
}
impl AppConstructor for ForthFile {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(Terminal(Vec::new(),self.fm.clone()))
    }
}

pub struct Terminal(Vec<String>, Arc<RwLock<ForthMachine>>);

impl LittleManApp for Terminal {
    fn update(&mut self, handle: &mut OsHandle) {
        handle.call_exit();
    }
}
