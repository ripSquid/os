#[derive(Clone, Copy, Default)]
#[repr(C)]
struct GateDescriptor {
    offset_1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}

type IDT = [GateDescriptor; 256];

static mut idt: IDT = [GateDescriptor {offset_1: 0, selector: 0, ist: 0, type_attributes: 0, offset_2: 0, offset_3: 0, zero: 0}; 256];

pub fn setup_interrupt(address: u64) {
    // TODO: TRANSLATE ADDRESS INCASE NON-LINEAR GDT IS USED
    unsafe {
    idt[8].offset_1 = (address & 0xFFFF) as u16;
    idt[8].offset_2 = ((address & 0xFFFF0000) >> 16) as u16;
    idt[8].offset_3 = (((address & 0xFFFFFFFF_0000_0000)) >> 32) as u32;
    idt[8].type_attributes = 0x8E;
    }
}