use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    display::{BitmapVgaWriter, DefaultVgaWriter, UniversalVgaFormatter, VgaModeSwitch},
    forth::{ForthMachine, Stack},
    input::{Keyboard, KEYBOARD_QUEUE, ScanCode},
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