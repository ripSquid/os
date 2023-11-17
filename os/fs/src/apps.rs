
use core::ops::{Deref, DerefMut};

use alloc::boxed::Box;

use base::display::{DefaultVgaWriter, BitmapVgaWriter};
use crate::{Path, AppConstructor};


pub trait DefaultInstall: AppConstructor {
    fn path() -> Path;
}
pub trait InstallableApp: AppConstructor {
    fn install() -> (Path, Box<dyn AppConstructor>);
}
impl<T> InstallableApp for T where T: Default + DefaultInstall + AppConstructor {
    fn install() -> (Path, Box<dyn AppConstructor>) {
        (T::path(), Box::new(T::default()))
    }
} 
pub trait LittleManApp: Send + Sync + 'static {
    fn start(&mut self, _args: &[&str]) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle);
    fn shutdown(&mut self) {}
}
#[derive(Debug)]
pub enum StartError {}
pub struct OsHandle {
    control_flow: ControlFlow,
    graphics: GraphicsHandle,
}
impl OsHandle {
    pub fn running(&self) -> bool {
        self.control_flow == ControlFlow::Running
    }
    pub fn new(formatter: impl Into<GraphicsHandle>) -> Self {
        Self { control_flow: ControlFlow::Running, graphics: formatter.into(),}
    }
    pub fn call_exit(&mut self) {
        self.control_flow = ControlFlow::Quit;
    }
    pub fn text_mode_formatter(&mut self) -> Result<&mut DefaultVgaWriter, VideoModeError> {
        if let GraphicsHandleType::TextMode(formatter) = &mut *self.graphics {
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
impl Into<GraphicsHandle> for GraphicsHandleType {
    fn into(self) -> GraphicsHandle {
        GraphicsHandle { formatter: self }
    }
}
impl Deref for GraphicsHandle {
    type Target = GraphicsHandleType;

    fn deref(&self) -> &Self::Target {
        &self.formatter
    }
}
impl DerefMut for GraphicsHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.formatter
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
