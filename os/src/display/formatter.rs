use super::{primitives::PrimitiveDisplay, DefaultVgaWriter};

impl KernelDebug for [u8] {
    fn debug(&self, formatter: &mut KernelFormatter) {
        formatter.debug_bytes_fancy(self);
    }
}
impl KernelDebug for str {
    fn debug(&self, formatter: &mut KernelFormatter) {
        formatter.debug_str(self);
    }
}


pub trait KernelDebug {
    fn debug(&self, formatter: &mut KernelFormatter);
}
pub struct KernelFormatter<'a> {
    writer: &'a mut DefaultVgaWriter,
}
impl<'a> KernelFormatter<'a> {
    pub fn debug_str(&mut self, str: &str) -> &mut Self {
        self.writer.write_str(str);
        self
    }
    pub fn debug_hex<const CAP: usize, const LEN: usize>(&mut self, hex: impl PrimitiveDisplay<CAP, LEN>) -> &mut Self {
        self.writer.write_bytes(hex.as_hexadecimal_ascii().as_ref());
        self
    }
    pub fn debug_num<const CAP: usize, const LEN: usize>(&mut self, hex: impl PrimitiveDisplay<CAP, LEN>) -> &mut Self {
        self.writer.write_bytes(hex.as_hexadecimal_ascii().as_ref());
        self
    }
    pub fn debug_bytes_fancy(&mut self, bytes: &[u8]) -> &mut Self {
        self.writer.write_str("[ ");
        for byte in bytes {
            self.writer.write_bytes((*byte).as_hexadecimal_ascii().as_ref());
            self.writer.write_str(" ");
        }
        self.writer.write_str("]");
        self

    }
    pub fn new(writer: &'a mut DefaultVgaWriter) -> Self {
        Self { writer }
    }
}