use core::arch::asm;
use crate::display::macros::{print_str, print_hex};

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct IDTDescriptor {
    size: u16,  
    offset: u64,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct GateDescriptor { 
    offset_1: u16,
    selector: SegmentSelector,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}

#[repr(transparent)]
#[derive(Copy, Clone, Default)]
struct SegmentSelector(u16);

type IDT = [GateDescriptor; 256];

static mut idt: IDT = [GateDescriptor {offset_1: 0, selector: SegmentSelector(0), ist: 0, type_attributes: 0, offset_2: 0, offset_3: 0, zero: 0}; 256];

static mut idtdescriptor: IDTDescriptor = IDTDescriptor {size: 0, offset: 0};

pub fn setup_interrupt(address: u64) {

    let tmp: SegmentSelector = SegmentSelector(0x80);

    // TODO: TRANSLATE ADDRESS INCASE PAGING IS USED
    unsafe {
        idt[40].offset_1 = (address & 0xFFFF) as u16;
        idt[40].offset_2 = ((address & 0xFFFF0000) >> 16) as u16;
        idt[40].offset_3 = (((address & 0xFFFFFFFF_0000_0000)) >> 32) as u32;
        idt[40].type_attributes = 0x8F;
        idt[40].selector = tmp;
    }

    // Load Interrupt Table
    unsafe {
        let addr: u64 = (&idt as *const _) as u64;
        print_str!("idt addr");
        print_hex!(addr);

        idtdescriptor.size = (2 + 2 + 1 + 1 + 2 + 4 + 4) * 256 - 1;
        idtdescriptor.offset = addr;
        
        let idtdecriptor_addr = &idtdescriptor;
        print_str!("idtdesc addr");
        print_hex!((idtdecriptor_addr as *const _) as u64);

        asm! {
            "lidt [{x}]",
            "sti",
            x = in(reg) idtdecriptor_addr,
        }
        print_str!("Loaded IDT");
    }

}