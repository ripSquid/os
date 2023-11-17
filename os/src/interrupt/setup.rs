use core::arch::asm;

use super::gatedescriptor::TypeAttribute;
use super::table::IDTable;
use super::timer;
use crate::display::STATIC_VGA_WRITER;
use crate::display::macros::debug;
use crate::input::{keyboard_handler, keyboard_initialize, setup_keymap};
use crate::interrupt::gatedescriptor::SegmentSelector;
use alloc::format;
use pic8259::ChainedPics;
use ps2::{error::ControllerError, flags::ControllerConfigFlags, Controller};
use x86_64::registers::segmentation::Segment;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::DescriptorTablePointer;
use x86_64::VirtAddr;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct IDTDescriptor {
    pub size: u16,
    pub offset: u64,
}
pub static mut global_os_time: u64 = 0;
static mut idt: IDTable = IDTable::new();

static mut idtdescriptor: DescriptorTablePointer = DescriptorTablePointer {
    limit: 0,
    base: VirtAddr::zero(),
};

pub static mut pics: ChainedPics = unsafe { ChainedPics::new(0x20, 0x28) };

pub unsafe fn setup_interrupts() {
    setup_keymap();

    idt.breakpoint.set_function(
        breakpoint,
        TypeAttribute(0b1000_1110_0000_0000),
        SegmentSelector(8),
    );
    idt.user_interupts[1].set_function(
        keyboard_handler,
        TypeAttribute(0b1000_1110_0000_0000),
        SegmentSelector(8),
    );
    idt.user_interupts[0].set_function(
        timer,
        TypeAttribute(0b1000_1110_0000_0000),
        SegmentSelector(8),
    );

    // --- TIMER TESTING

    // --- TIMER TESTING

    pics.write_masks(0b0000_0000, 0u8);
    pics.initialize();

    idtdescriptor = idt.pointer();
    x86_64::instructions::tables::lidt(&idtdescriptor);

    // ps2 setup (structuring no.)
    unsafe {
        let rs = keyboard_initialize();
        if let Err(e) = rs {
            STATIC_VGA_WRITER.write_str(&format!("{:?}", e));
            loop {}
        }

        
    };



    // Enable interrupts
    asm!("sti");
}

pub extern "x86-interrupt" fn breakpoint(_stack_frame: InterruptStackFrame) {
    debug!("breakpoint triggered!");
}

pub extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    unsafe {
        global_os_time += 1;
        pics.notify_end_of_interrupt(32);
    }
}
