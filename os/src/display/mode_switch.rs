use x86_64::instructions::port::{PortWriteOnly, PortReadOnly};

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
struct VGASwitch {
    misc: u8,
    sequence_regs: [u8; 5],
    crtc: [u8; 25],
    gc: [u8; 9],
    ac: [u8; 21],
}

const VGA_320X200_BITMAP: VGASwitch = VGASwitch {


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
pub fn switch_graphics_mode() {
    unsafe {write_registers(VGA_320X200_BITMAP.clone()) };
}

unsafe fn write_registers(mut set: VGASwitch) {
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