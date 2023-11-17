mod help;
mod dir;
use alloc::boxed::Box;
pub use help::*;
pub use dir::*;
use crate::{display::{DefaultVgaWriter, BitmapVgaWriter}, fs::{Path, AppConstructor}};
pub trait InstallableApp: AppConstructor {
    fn install() -> (Path, Box<dyn AppConstructor>);
}
pub trait LittleManApp: Send + Sync + 'static {
    fn start(&mut self, args: &[&str]) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle);
    fn shutdown(&mut self) {}
}
#[derive(Debug)]
pub enum StartError {}
pub struct OsHandle {
    control_flow: ControlFlow,
    graphics: GraphicsHandleType
}
impl OsHandle {
    pub fn running(&self) -> bool {
        self.control_flow == ControlFlow::Running
    }
    pub fn new(formatter: GraphicsHandleType) -> Self {
        Self { control_flow: ControlFlow::Running, graphics: formatter }
    }
    pub fn call_exit(&mut self) {
        self.control_flow = ControlFlow::Quit;
    }
    pub fn text_mode_formatter(&mut self) -> Result<&mut DefaultVgaWriter, VideoModeError> {
        if let GraphicsHandleType::TextMode(formatter) = &mut self.graphics {
            Ok(unsafe {&mut **formatter})
        } else {
            Err(VideoModeError::IsInGraphicsMode)
        }
    }
}
pub enum VideoModeError {
    IsInGraphicsMode
}
pub struct GraphicsHandle {
    formatter: GraphicsHandleType,
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
