mod help;
pub use help::*;

use crate::display::{DefaultVgaWriter, BitmapVgaWriter};
pub trait KaggApp: Send + Sync + 'static {
    fn start(&mut self, args: &[&str]) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut KaggHandle);
    fn shutdown(&mut self) {}
}
pub enum StartError {}
pub struct KaggHandle {
    control_flow: ControlFlow,
    graphics: GraphicsHandleType
}
impl KaggHandle {
    pub fn call_exit(&mut self) {
        self.control_flow = ControlFlow::Quit;
    }
    pub fn text_mode_formatter(&mut self) -> Result<&mut DefaultVgaWriter, VideoModeError> {
        if let GraphicsHandleType::TextMode(formatter) = &mut self.graphics {
            Ok(formatter)
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
    TextMode(DefaultVgaWriter),
    GraphicsMode(BitmapVgaWriter),
}
pub enum ControlFlow {
    Running,
    Quit,
}
