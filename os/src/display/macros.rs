macro_rules! print_str {
    ($term: expr) => {{
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_str($term);
    }};
}

pub(crate) use print_str;

macro_rules! debug {
    ($($term: expr),+) => {
        {
            let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
                &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
            });
            writer.prepare_print();
            let mut formatter = crate::display::KernelFormatter::new(&mut writer);
            $(
                formatter = crate::display::KernelDebug::debug($term, formatter).debug_str(" ");
            )*
        }
    };
}
pub(crate) use debug;
