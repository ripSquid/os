use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    display::{BitmapVgaWriter, DefaultVgaWriter, UniversalVgaFormatter, VgaModeSwitch},
    forth::{ForthMachine, Stack},
    input::{Keyboard, KEYBOARD_QUEUE},
};

pub trait LittleManApp: Send + Sync + 'static {
    fn run(&mut self, _machine: &mut ForthMachine) -> Result<(), ProgramError>;
}

#[derive(Debug)]
pub enum ProgramError {
    InvalidStartParameter,
    InvalidParameter,
    FileSystemError,
    InternalError,
    Custom(&'static str),
    Crash,
}
pub struct OsHandle {
    fm: Option<*mut ForthMachine>,
    control_flow: ControlFlow,
    graphics: GraphicsHandle,
    calls: Vec<SystemCall>,
}
pub enum SystemCall {
    SwitchGraphics(VgaModeSwitch),
    ForthFunction(String),
}
impl OsHandle {
    pub fn keyboard(&self) -> &Keyboard<char> {
        unsafe { &KEYBOARD_QUEUE }
    }
    pub fn execute(&mut self, forth_command: impl ToString) -> bool {
        match self.fm.map(|s| unsafe { s.as_mut() }).flatten() {
            Some(accessible) => {
                /*
                accessible.run(
                    forth_command.to_string(),
                    self.text_mode_formatter().unwrap(),
                );
                */
                true
            }
            None => {
                self.calls
                    .push(SystemCall::ForthFunction(forth_command.to_string()));
                false
            }
        }
    }
    pub fn keyboard_mut(&mut self) -> &mut Keyboard<char> {
        unsafe { &mut KEYBOARD_QUEUE }
    }
    pub fn running(&self) -> bool {
        self.control_flow == ControlFlow::Running
    }
    pub fn new(formatter: impl Into<GraphicsHandle>) -> Self {
        Self {
            control_flow: ControlFlow::Running,
            graphics: formatter.into(),
            calls: Vec::new(),
            fm: None,
        }
    }
    pub unsafe fn new_complicated(
        formatter: impl Into<GraphicsHandle>,
        machine: &mut ForthMachine,
    ) -> Self {
        Self {
            control_flow: ControlFlow::Running,
            graphics: formatter.into(),
            calls: Vec::new(),
            fm: Some(machine as *mut _),
        }
    }
    pub fn flush_calls(&mut self) -> Vec<SystemCall> {
        self.calls.split_off(0)
    }
    pub fn call_exit(&mut self) {
        self.control_flow = ControlFlow::Quit;
    }
    pub fn text_mode_formatter(&mut self) -> Result<&mut DefaultVgaWriter, VideoModeError> {
        Ok(&mut self.graphics.formatter)
    }
}
#[derive(Debug)]
pub enum VideoModeError {
    IsInGraphicsMode,
}
pub struct GraphicsHandle {
    formatter: UniversalVgaFormatter,
}
impl GraphicsHandle {
    pub fn from_universal(formatter: UniversalVgaFormatter) -> Self {
        Self { formatter }
    }
}

pub enum GraphicsHandleType {
    TextMode(*mut DefaultVgaWriter),
    GraphicsMode(BitmapVgaWriter),
}
#[derive(PartialEq)]
pub enum ControlFlow {
    Running,
    Quit,
}
