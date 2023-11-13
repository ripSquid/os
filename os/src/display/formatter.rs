use super::{primitives::PrimitiveDisplay, DefaultVgaWriter, VgaColor, VgaColorCombo};

impl<'a> KernelDebug<'a> for [u8] {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter.debug_bytes_fancy(self)
    }
}
impl<'a> KernelDebug<'a> for str {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter.debug_str(self)
    }
}

impl<'a, T> KernelDebug<'a> for &[T]
where
    T: KernelDebug<'a>,
{
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        let mut form = formatter;
        if !self.is_empty() {
            form = KernelDebug::debug(&self[0], form);
            for item in &self[1..] {
                form = form.debug_str(",");
                form = KernelDebug::debug(item, form);
            }
        }

        form
    }
}

impl<'a> KernelDebug<'a> for char {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter.debug_str(self.encode_utf8(&mut [0; 4]))
    }
}

impl<'a, T> KernelDebug<'a> for &T where T: KernelDebug<'a> {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        (*self).debug(formatter)
    }
}
pub trait KernelDebug<'a> {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a>;
}
pub struct KernelFormatter<'a> {
    writer: &'a mut DefaultVgaWriter,
}
impl<'a> KernelFormatter<'a> {
    pub fn debug_str(self, str: &str) -> Self {
        self.writer.write_str(str);
        self
    }
    pub fn debug_bytes(self, bytes: &[u8]) -> Self {
        self.writer.write_bytes(bytes);
        self
    }
    pub fn debug_hex<const CAP: usize, const LEN: usize>(
        self,
        hex: impl PrimitiveDisplay<CAP, LEN>,
    ) -> Self {
        self.writer.write_bytes(hex.as_hexadecimal_ascii().as_ref());
        self
    }
    pub fn debug_num<const CAP: usize, const LEN: usize>(
        self,
        hex: impl PrimitiveDisplay<CAP, LEN>,
    ) -> Self {
        self.writer.write_bytes(hex.as_hexadecimal_ascii().as_ref());
        self
    }
    pub fn debug_bytes_fancy(self, bytes: &[u8]) -> Self {
        self.writer.write_str("[ ");
        for byte in bytes {
            self.writer
                .write_bytes((*byte).as_hexadecimal_ascii().as_ref());
            self.writer.write_str(" ");
        }
        self.writer.write_str("]");
        self
    }
    pub fn new(writer: &'a mut DefaultVgaWriter) -> Self {
        Self { writer }
    }
    pub fn debug_struct(self, struct_name: &str) -> StructFormatter<'a> {
        self.writer
            .set_default_colors(VgaColorCombo::new(VgaColor::Black, VgaColor::White));
        self.writer.write_str(struct_name);
        self.writer
            .set_default_colors(VgaColorCombo::on_black(VgaColor::Yellow));
        self.writer.write_str("{");
        StructFormatter::new(self)
    }
}
pub struct StructFormatter<'a> {
    formatter: KernelFormatter<'a>,
    count: usize,
}
impl<'a> StructFormatter<'a> {
    pub fn new(formatter: KernelFormatter<'a>) -> Self {
        Self {
            formatter,
            count: 0,
        }
    }
    pub fn debug_field(mut self, name: &str, field: &impl KernelDebug<'a>) -> Self {
        if self.count != 0 {
            self.formatter
                .writer
                .set_default_colors(VgaColorCombo::on_black(VgaColor::DarkGray));
            self.formatter.writer.write_str(",");
        }
        self.formatter
            .writer
            .set_default_colors(VgaColorCombo::on_black(VgaColor::LightCyan));
        self.formatter.writer.write_str(name);
        self.formatter
            .writer
            .set_default_colors(VgaColorCombo::on_black(VgaColor::LightGray));
        self.formatter.writer.write_str(":");
        {
            self.formatter
                .writer
                .set_default_colors(VgaColorCombo::on_black(VgaColor::Green));
            self = Self {
                count: self.count + 1,
                formatter: field.debug(self.formatter),
            };
        }
        self
    }
    pub fn finish(self) -> KernelFormatter<'a> {
        self.formatter
            .writer
            .set_default_colors(VgaColorCombo::on_black(VgaColor::Yellow));
        self.formatter.writer.write_str("} ");
        self.formatter
    }
    pub fn finish_none_exhaustive(self) -> KernelFormatter<'a> {
        self.formatter
            .writer
            .set_default_colors(VgaColorCombo::on_black(VgaColor::Yellow));
        self.formatter.writer.write_str(",.. } ");
        self.formatter
    }
}
