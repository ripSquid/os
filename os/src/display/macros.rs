

macro_rules! print_str {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_str($term);
    };
}

pub(crate) use print_str;

macro_rules! print_hex {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_bytes($term.as_hexadecimal_ascii());
    };
}
macro_rules! print_num {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_bytes($term.as_numeric_ascii());
    };
}