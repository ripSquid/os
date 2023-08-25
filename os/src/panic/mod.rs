use core::panic::PanicInfo;
use crate::{display::{VgaColorCombo, DefaultVgaBuffer, VgaColor, DefaultVgaWriter}, VGA_BUFFER_ADDRESS};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = DefaultVgaWriter::new(unsafe {&mut *(VGA_BUFFER_ADDRESS as *mut DefaultVgaBuffer)});
    let error_color = VgaColorCombo::new(VgaColor::White, VgaColor::Red);
    writer.set_default_colors(error_color);
    writer.write_str("PANIC OCCURED:");
    if let Some(location) = info.location() {
        writer.next_line();
        writer.write_str("   file: ");
        writer.write_str(location.file());
        writer.next_line();
        writer.write_str("   line: ");
        writer.write_bytes(U32Str::from(location.line()).as_ref());
        writer.next_line();
        writer.write_str("   column: ");
        writer.write_bytes(U32Str::from(location.column()).as_ref());
        
    }


    loop {}
}

/// a datatype with enough capacity to hold any u32 value
struct U32Str {
    value: [u8; 10],
    len: usize,
}
impl U32Str {
    pub fn as_bytes(&self) -> &[u8] {
        &self.value[..self.len]
    }
    pub fn from(u: u32) -> Self {
        let mut value: [u8; 10] = [0; 10];
        let mut len = 0;
        let mut accounted = 0;
        for i in (0..10).rev() {
            let mut mult = 1;
            for _ in 0..i {
                mult *= 10;
            }
            let this_char = (u - accounted) / mult;
            if this_char > 0 {
                accounted += this_char * mult;
            } else {
                continue;
            }
            if accounted > 0 {
                value[len] = (this_char as u8) + 0x30;
                len += 1;
            }
        }
        U32Str { value, len }
    }
}
impl AsRef<[u8]> for U32Str {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}