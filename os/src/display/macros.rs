

macro_rules! print_str {
    ($term: expr) => {
        {
            let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
                &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
            });
            writer.prepare_print();
            writer.write_str($term);
        }
    };
}


pub(crate) use print_str;

macro_rules! print_bytes {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_bytes($term);
    };
}

pub(crate) use print_bytes;


macro_rules! print_hex {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_bytes(crate::display::primitives::PrimitiveDisplay::as_hexadecimal_ascii(&$term).as_ref());
    };
}

pub(crate) use print_hex;

macro_rules! print_num {
    ($term: expr) => {
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
        });
        writer.prepare_print();
        writer.write_bytes(crate::display::primitives::PrimitiveDisplay::as_numerical_ascii(&$term).as_ref());
    };
}

pub(crate) use print_num;

macro_rules! debug {
    ($($term: expr),+) => {
        {
            let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
                &mut *(0xB8000 as *mut crate::display::DefaultVgaBuffer)
            });
            let mut formatter = crate::display::KernelFormatter::new(&mut writer);
            $(
                crate::display::KernelDebug::debug($term, &mut formatter);
            )*
        }
    };
}
pub(crate) use debug;
