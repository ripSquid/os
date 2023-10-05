use super::gatedescriptor::{CPUPrivilege, InterruptType, TypeAttribute};
use super::table::IDTable;
use crate::display::macros::{debug, print_hex, print_num, print_str};
use crate::interrupt::gatedescriptor::{GateDescriptor, SegmentSelector};
#[allow(unconditional_panic)]
use core::arch::asm;
use core::mem::size_of;
use pic8259;
use x86_64::VirtAddr;
use x86_64::instructions::hlt;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::DescriptorTablePointer;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
// file /c/Users/antip/Projects/osdev/build/isofiles/boot/kernel.bin
extern "C" {
    fn interrupt_wrapper();
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct IDTDescriptor {
    pub size: u16,
    pub offset: u64,
}


static mut idt: IDTable = IDTable::new();

static mut idtdescriptor: DescriptorTablePointer = DescriptorTablePointer {limit: 0, base: VirtAddr::zero()};

pub fn setup_interrupt(address: u64) {
    /* 
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
            idt[i] = GateDescriptor::new(
                address,
                true,
                CPUPrivilege::KERNEL,
                InterruptType::Trap,
                SegmentSelector(0),
                0,
            );
        }

        for i in 32..=255 {
            idt[i] = GateDescriptor::new(
                address,
                true,
                CPUPrivilege::KERNEL,
                InterruptType::Interrupt,
                SegmentSelector(0),
                0,
            );
        }

        //idt[32] = GateDescriptor::new((&interrupt_handler() as *const _) as u64, true, CPUPrivilege::KERNEL, InterruptType::Interrupt, SegmentSelector(0), 0);
    }

    // Load Interrupt Table
    unsafe {
        let addr: u64 = (&idt[0] as *const _) as u64;
        print_str!("idt addr");
        print_hex!(addr);

        idtdescriptor.size = (idt.len() * size_of::<GateDescriptor>() - 1) as u16;
        idtdescriptor.offset = addr;

        let idtdecriptor_addr = (&idtdescriptor as *const IDTDescriptor) as u64;
        print_str!("idtdesc addr");
        print_hex!(idtdecriptor_addr);

        asm! {
            "cli",
            "lidt [{x}]",
            "sti",
            x = in(reg) (idtdecriptor_addr) as u64,
        }

        //asm!("int 0");
    }
    */
}

pub extern "x86-interrupt" fn interrupt_handler() {
    //print_str!("Interrupt UwU");
    loop {}
}

pub extern "x86-interrupt" fn exception_handler() {
    //print_str!("Exception UwU");
    loop {}
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
pub unsafe fn setup_interrupts() {
    IDT.breakpoint.set_handler_fn(breakpoint);
    //IDT.load();
    let segment = SegmentSelector(x86_64::registers::segmentation::CS::get_reg().0);
    debug!(&segment.0);
    idt.breakpoint.set_function(breakpoint, TypeAttribute(0b1000_0111), segment);
    idtdescriptor = idt.pointer();
    x86_64::instructions::tables::lidt(&idtdescriptor);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    debug!("interruptt breakpoint");
}
