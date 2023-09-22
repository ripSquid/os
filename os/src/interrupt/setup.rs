use core::arch::asm;
use crate::display::macros::{print_str, print_hex};
use crate::interrupt::gatedescriptor::{GateDescriptor, SegmentSelector};

use super::gatedescriptor::{InterruptType, CPUPrivilege};

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
struct IDTDescriptor {
    size: u16,  
    offset: u64,
}

type IDT = [GateDescriptor; 256];


/*
pub fn setup_interrupt(address: u64) {

    let tmp: SegmentSelector = SegmentSelector(0x80);

    // TODO: TRANSLATE ADDRESS INCASE PAGING IS USED
    unsafe {
        for i in 0..=255 {
            idt[i] = GateDescriptor::new(address, true, CPUPrivilege::KERNEL, InterruptType::Trap, SegmentSelector(0), 0);
        }
        
    }

    // Load Interrupt Table
    unsafe {
        let addr: u64 = (&idt[0] as *const _) as u64;
        print_str!("idt addr");
        print_hex!(addr);

        idtdescriptor.size = (2 + 2 + 1 + 1 + 2 + 4 + 4) * 256 - 1;
        idtdescriptor.offset = addr;
        
        let idtdecriptor_addr = &idtdescriptor;
        print_str!("idtdesc addr");
        print_hex!((idtdecriptor_addr as *const _) as u64);

        asm! {
            "cli",
            "lidt [{x}]",
            "sti",
            x = in(reg) idtdecriptor_addr,
        }
        print_str!("Loaded IDT");
    }

}
 */