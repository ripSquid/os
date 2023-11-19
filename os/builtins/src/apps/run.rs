use alloc::{boxed::Box, string::String, sync::Arc};
use base::{forth::Stack, LittleManApp, OsHandle, StartError};
use fs::{AppConstructor, Path};
use hashbrown::HashMap;
use spin::RwLock;

type RunningTable = Arc<RwLock<HashMap<String, Path>>>;
pub struct View(RunningTable);

pub struct ViewInstance(RunningTable, Option<Path>);

impl AppConstructor for View {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ViewInstance(self.0.clone(), None))
    }
}

impl LittleManApp for ViewInstance {
    fn start(&mut self, _args: &mut Stack) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        todo!()
    }
}
