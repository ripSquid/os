use x86_64::instructions::port::{PortWriteOnly, PortReadOnly, Port};

const VGA_AC_INDEX: u16 =		    0x3C0;
const VGA_AC_WRITE: u16 =		    0x3C0;
const VGA_AC_READ: u16 =		    0x3C1;
const VGA_MISC_WRITE: u16 =		    0x3C2;
const VGA_SEQ_INDEX: u16 =		    0x3C4;
const VGA_SEQ_DATA: u16 =		    0x3C5;
const VGA_DAC_READ_INDEX: u16 =	    0x3C7;
const VGA_DAC_WRITE_INDEX: u16 =	0x3C8;
const VGA_DAC_DATA: u16 =		    0x3C9;
const VGA_MISC_READ: u16 =		    0x3CC;
const VGA_GC_INDEX: u16 =		    0x3CE;
const VGA_GC_DATA: u16 = 		    0x3CF;
const VGA_CRTC_INDEX: u16 =		    0x3D4;		/* 0x3B4 */
const VGA_CRTC_DATA: u16 =		    0x3D5;		/* 0x3B5 */
const VGA_INSTAT_READ: u16 =	    0x3DA;

#[derive(Clone)]
pub struct VgaModeSwitch {
    misc: u8,
    sequence_regs: [u8; 5],
    crtc: [u8; 25],
    gc: [u8; 9],
    ac: [u8; 21],
}
impl VgaModeSwitch {
    pub const VGA_320X200_BITMAP_MODEX: VgaModeSwitch = VgaModeSwitch {


        /* MISC */
            misc: 0x63,
        /* SEQ */
            sequence_regs: [0x03, 0x01, 0x0F, 0x00, 0x06],
        /* CRTC */
            crtc: [0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
            0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x9C, 0x0E, 0x8F, 0x28, 0x00, 0x96, 0xB9, 0xE3,
            0xFF],
        /* GC */
            gc: [0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F,
            0xFF],
        /* AC */
            ac: [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
            0x41, 0x00, 0x0F, 0x00, 0x00],
    };
    pub const VGA_320X200_BITMAP_N: VgaModeSwitch = VgaModeSwitch {
        /* MISC */
            misc: 0x63,
        /* SEQ */
            sequence_regs: [0x03, 0x01, 0x0F, 0x00, 0x0E],
        /* CRTC */
           crtc: [ 0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
            0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x9C, 0x0E, 0x8F, 0x28,	0x40, 0x96, 0xB9, 0xA3,
            0xFF],
        /* GC */
           gc: [ 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F,
            0xFF],
        /* AC */
            ac: [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
            0x41, 0x00, 0x0F, 0x00,	0x00]
        };
        
    pub const VGA_80X25_TEXT: VgaModeSwitch = VgaModeSwitch {
        /* MISC */
        misc:    0x67,
        /* SEQ */
        sequence_regs:    [0x03, 0x00, 0x03, 0x00, 0x02],
        /* CRTC */
            crtc: [0x5F, 0x4F, 0x50, 0x82, 0x55, 0x81, 0xBF, 0x1F,
            0x00, 0x4F, 0x0D, 0x0E, 0x00, 0x00, 0x00, 0x50,
            0x9C, 0x0E, 0x8F, 0x28, 0x1F, 0x96, 0xB9, 0xA3,
            0xFF],
        /* GC */
            gc: [0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x0E, 0x00,
            0xFF],
        /* AC */
            ac: [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
            0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
            0x0C, 0x00, 0x0F, 0x08, 0x00]
    };
}



pub fn switch_graphics_mode(mode: VgaModeSwitch) {
    unsafe {write_registers(mode) };
}

pub unsafe fn restore_text_mode_font() {
    let font = include_bytes!("font.bin");
    let (s2, s4, gc4, gc5, gc6);
    let mut gc_index = Port::<u8>::new(VGA_GC_INDEX);
    let mut gc_data = Port::<u8>::new(VGA_GC_DATA);
    let mut seq_index = Port::<u8>::new(VGA_SEQ_INDEX);
    let mut seq_data = Port::<u8>::new(VGA_SEQ_DATA);
    seq_index.write(2);
    s2 = seq_data.read();

    seq_index.write(4);
    s4 = seq_data.read();

    seq_data.write( s4 | 0x04);

    gc_index.write(4);
    gc4 = gc_data.read();

    gc_index.write(5);
    gc5 = gc_data.read();

    gc_data.write(gc5 & !0x10);

    gc_index.write(6);
    gc6 = gc_data.read();

    gc_data.write(gc6 & !0x02);

    set_plane(2);

    let start = get_start_addr();
    for (i, byte) in font.iter().enumerate() {
        *((start + i as u64) as *mut u8) = *byte;
   }

    seq_index.write(2);
    seq_data.write(s2);

    seq_index.write(4);
    seq_data.write(s4);

    gc_index.write(4);
    gc_data.write(gc4);
    gc_index.write(5);
    gc_data.write(gc5);
    gc_index.write(6);
    gc_data.write(gc6);


}
unsafe fn get_start_addr() -> u64 {
    PortWriteOnly::<u8>::new(VGA_GC_INDEX).write(6);
    let mut seg: u8 = PortReadOnly::new(VGA_GC_DATA).read();
    seg >>= 2;
    seg &= 3;
    match seg {
        0 | 1 => {
            0xA0000
        },
        2 => {
            0xB0000
        }
        3 => {
            0xB8000
        }
        _ => unreachable!(),
    }
}
unsafe fn set_plane(mut plane: u8) {
    plane &= 3;
    let pmask = 1 << plane;
    PortWriteOnly::<u8>::new(VGA_GC_INDEX).write(4);
    PortWriteOnly::<u8>::new(VGA_GC_DATA).write(plane);
    PortWriteOnly::<u8>::new(VGA_SEQ_INDEX).write(2);
    PortWriteOnly::<u8>::new(VGA_SEQ_DATA).write(pmask);
}

unsafe fn write_registers(mut set: VgaModeSwitch) {
    PortWriteOnly::<u8>::new(VGA_MISC_WRITE).write(set.misc);
    for (i, data) in set.sequence_regs.iter().enumerate() {
        PortWriteOnly::<u8>::new(VGA_SEQ_INDEX).write(i as u8);
        PortWriteOnly::<u8>::new(VGA_SEQ_DATA).write(*data);
    }
    PortWriteOnly::<u8>::new(VGA_CRTC_INDEX).write(0x03);
    PortWriteOnly::<u8>::new(VGA_CRTC_DATA).write(PortReadOnly::<u8>::new(VGA_CRTC_DATA).read() | 0x80);
    PortWriteOnly::<u8>::new(VGA_CRTC_INDEX).write(0x11);
    PortWriteOnly::<u8>::new(VGA_CRTC_DATA).write(PortReadOnly::<u8>::new(VGA_CRTC_DATA).read() & !0x80);
    
    set.crtc[0x03] |= 0x80;
    set.crtc[0x11] &= !0x80;

    for (i,data) in set.crtc.iter().enumerate() {
        PortWriteOnly::<u8>::new(VGA_CRTC_INDEX).write(i as u8);
        PortWriteOnly::<u8>::new(VGA_CRTC_DATA).write(*data);
    }

    for (i,data) in set.gc.iter().enumerate() {
        PortWriteOnly::<u8>::new(VGA_GC_INDEX).write(i as u8);
        PortWriteOnly::<u8>::new(VGA_GC_DATA).write(*data);
    }

    for (i,data) in set.ac.iter().enumerate() {
        PortReadOnly::<u8>::new(VGA_INSTAT_READ).read();
        PortWriteOnly::<u8>::new(VGA_AC_INDEX).write(i as u8);
        PortWriteOnly::<u8>::new(VGA_AC_WRITE).write(*data);
    }

    PortReadOnly::<u8>::new(VGA_INSTAT_READ).read();
    PortWriteOnly::<u8>::new(VGA_AC_INDEX).write(0x20);
}