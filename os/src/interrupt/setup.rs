use super::gatedescriptor::TypeAttribute;
use super::table::IDTable;
use crate::display::macros::debug;
use crate::interrupt::gatedescriptor::SegmentSelector;

use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::DescriptorTablePointer;
use x86_64::VirtAddr;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct IDTDescriptor {
    pub size: u16,
    pub offset: u64,
}

static mut idt: IDTable = IDTable::new();

static mut idtdescriptor: DescriptorTablePointer = DescriptorTablePointer {
    limit: 0,
    base: VirtAddr::zero(),
};

pub unsafe fn setup_interrupts() {
    idt.breakpoint.set_function(
        breakpoint,
        TypeAttribute(0b1000_1110_0000_0000),
        SegmentSelector(8),
    );
    idtdescriptor = idt.pointer();
    x86_64::instructions::tables::lidt(&idtdescriptor);
}

pub extern "x86-interrupt" fn breakpoint(_stack_frame: InterruptStackFrame) {
    debug!("breakpoint triggered!");
}
