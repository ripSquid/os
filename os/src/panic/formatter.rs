use core::fmt::{Write, Result};

use crate::display::DefaultVgaWriter;

pub struct PanicBuffer<'a> {
    writer: &'a mut DefaultVgaWriter,
}
impl<'a> PanicBuffer<'a> {
    pub fn new(writer: &'a mut DefaultVgaWriter) -> Self {
        Self { writer }
    }
}
impl<'a> Write for PanicBuffer<'a> {
    fn write_str(&mut self, s: &str) -> Result {
        self.writer.write_str(s);
        Ok(())
    }
}

