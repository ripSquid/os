

macro_rules! print_str {
    ($term: expr) => {
        use crate::{display::primitives::PrimitiveDisplay as Npd, DefaultVgaBuffer as Vpb};
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut Vpb)
        });
        writer.prepare_print();
        writer.write_str($term);
    };
}

pub(crate) use print_str;

macro_rules! print_hex {
    ($term: expr) => {
        use crate::{display::primitives::PrimitiveDisplay as Hpd, DefaultVgaBuffer as Hpb};
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut Hpb)
        });
        writer.prepare_print();
        writer.write_bytes($term.as_hexadecimal_ascii().as_ref());
    };
}

pub(crate) use print_hex;

macro_rules! print_num {
    ($term: expr) => {
        use crate::DefaultVgaBuffer as StrVb;
        let mut writer = crate::display::DefaultVgaWriter::new(unsafe {
            &mut *(0xB8000 as *mut StrVb)
        });
        writer.prepare_print();
        writer.write_bytes($term.as_numeric_ascii().as_ref());
    };
}

pub(crate) use print_num;
