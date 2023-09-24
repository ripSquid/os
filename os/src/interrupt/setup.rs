#[allow(unconditional_panic)]

use core::arch::asm;
use crate::display::macros::{print_str, print_hex, print_num};
use crate::interrupt::gatedescriptor::{GateDescriptor, SegmentSelector};
use super::gatedescriptor::{InterruptType, CPUPrivilege, TypeAttribute};
use pic8259;
use x86_64::instructions::hlt;
use x86_64::structures::idt::InterruptStackFrame;
// file /c/Users/antip/Projects/osdev/build/isofiles/boot/kernel.bin
extern "C" {

}

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
struct IDTDescriptor {
    size: u16,  
    offset: u64,
}

type IDT = [GateDescriptor; 256];

static mut idt: IDT = [GateDescriptor {offset_1: 0, offset_2: 0, offset_3: 0, selector: SegmentSelector(0), ist: 0, type_attributes: TypeAttribute(0), zero: 0}; 256];

static mut idtdescriptor: IDTDescriptor = IDTDescriptor {size: 0, offset: 0};

pub fn setup_interrupt(address: u64) {

    let mut chained_pics: pic8259::ChainedPics;
    unsafe {
        chained_pics = pic8259::ChainedPics::new(0x20, 0x28);
        chained_pics.initialize();
        chained_pics.write_masks(0xFF, 0xFF);
    }

    let tmp: SegmentSelector = SegmentSelector(0x80);

    // TODO: TRANSLATE ADDRESS INCASE PAGING IS USED
    unsafe {
        for i in 0..32 {
            idt[i] = GateDescriptor::new((exception_handler as *const ()) as u64, true, CPUPrivilege::KERNEL, InterruptType::Trap, SegmentSelector(0), 0);
        }
        
        for i in 32..=255 {
            idt[i] = GateDescriptor::new((interrupt_handler as *const ()) as u64, true, CPUPrivilege::KERNEL, InterruptType::Interrupt, SegmentSelector(0), 0);
        }

        //idt[32] = GateDescriptor::new((&interrupt_handler() as *const _) as u64, true, CPUPrivilege::KERNEL, InterruptType::Interrupt, SegmentSelector(0), 0);
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
        print_hex!((idtdecriptor_addr as *const IDTDescriptor) as u64);

        asm! {
            "cli",
            "lidt [{x}]",
            "sti",
            x = in(reg) (idtdecriptor_addr as *const IDTDescriptor) as u64,
        }

        print_str!("Loaded IDT");
        print_hex!((exception_handler as *const ()) as u64);

        //asm!("int 0");
    } 

}
 
pub extern "x86-interrupt"  fn interrupt_handler() {
    //print_str!("Interrupt UwU");
    loop {}
}


pub extern "x86-interrupt"  fn exception_handler() {
    //print_str!("Exception UwU");
    loop {}
}


