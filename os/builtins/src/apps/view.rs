use alloc::{boxed::Box, string::String, sync::Arc, format};
use base::{
    forth::{ForthMachine, Stack},
    LittleManApp, OsHandle, ProgramError,
};
use fs::{AppConstructor, Path, DefaultInstall};
use hashbrown::HashMap;
use spin::RwLock;

type RunningTable = Arc<RwLock<HashMap<String, Path>>>;
#[derive(Default)]
pub struct View(RunningTable);

pub struct ViewInstance(RunningTable);

impl DefaultInstall for View {
    fn path() -> Path {
        Path::from("view.run")
    }
}
impl AppConstructor for View {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ViewInstance(self.0.clone()))
    }
}

impl LittleManApp for ViewInstance {
    fn run(&mut self, machine: &mut ForthMachine) -> Result<(), ProgramError> {
       
        let program = {
            let arg1 = machine
                .stack
                .try_pop::<String>()
                .ok_or(ProgramError::InvalidStartParameter)?;
            match arg1.as_str() {
                "insert" => {
                    let path = machine
                        .stack
                        .try_pop::<String>()
                        .ok_or(ProgramError::InvalidParameter)?;
                    let extension = machine
                        .stack
                        .try_pop::<String>()
                        .ok_or(ProgramError::InvalidParameter)?;
                    let message = format!("Success! bound .{extension} to {path}");
                    self.0
                        .try_write()
                        .ok_or(ProgramError::InternalError)?
                        .insert(extension, Path::from(path));
                    machine.formatter.write_str(&message);
                    return Ok(());
                }
                _ => (),
            };
            let file_path = Path::from(arg1);

            let lock = self
            .0
            .try_read()
            .ok_or(ProgramError::InternalError)?;

            let program_path = lock.get(
                file_path.file_extension()
                    .ok_or(ProgramError::Custom("file has no extension!"))?,
            )
            .ok_or(ProgramError::Custom(
                "no default app set for this extension.",
            ))?;

            let mut string = String::new();
            string += "\"";
            string += file_path.as_str();
            string += "\" ";
            string += "\"";
            string += program_path.as_str();
            string += "\" ";
            string += "run";
            string
        };
        machine.instructions.add_instructions_to_end(&program.chars().collect());
        machine.run_to_end();
        Ok(())
    }
}
