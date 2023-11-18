use alloc::{string::String, boxed::Box, sync::Arc};
use base::{forth::Stack, LittleManApp, StartError, OsHandle};
use fs::{AppConstructor, Path};
use hashbrown::HashMap;
use spin::RwLock;

type RunningTable = Arc<RwLock<HashMap<String, Path>>>;
pub struct Run(RunningTable);

pub struct RunInstance(RunningTable, Option<Path>);

impl AppConstructor for Run {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(RunInstance(self.0.clone(), None))
    }
}
impl LittleManApp for RunInstance {
    fn start(&mut self, _args: &mut Stack) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        todo!()
    }
}