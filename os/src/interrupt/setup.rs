use core::arch::asm;

use super::gatedescriptor::TypeAttribute;
use super::table::IDTable;
use super::timer;
use crate::display::macros::debug;
use crate::interrupt::gatedescriptor::SegmentSelector;
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

static mut idt: IDTable = IDTable::new();

static mut idtdescriptor: DescriptorTablePointer = DescriptorTablePointer {
    limit: 0,
    base: VirtAddr::zero(),
};

static mut pics: ChainedPics = unsafe { ChainedPics::new(0x20, 0x28) };

static mut controller: Controller = unsafe { Controller::new() };

pub unsafe fn setup_interrupts() {
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

    pics.write_masks(0b0000_0001, 0u8);
    pics.initialize();

    idtdescriptor = idt.pointer();
    x86_64::instructions::tables::lidt(&idtdescriptor);

    // ps2 setup (structuring no.)
    unsafe { _ = keyboard_initialize() };

    // Enable interrupts
    asm!("sti");
}

pub extern "x86-interrupt" fn breakpoint(_stack_frame: InterruptStackFrame) {
    debug!("breakpoint triggered!");
}

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    unsafe { if let Ok(data) = controller.read_data() {
        debug!(&data);
    } }
    debug!("keyboard handler!");
    unsafe {
        pics.notify_end_of_interrupt(33);
    }
}

pub extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    debug!(".");
    unsafe {
        pics.notify_end_of_interrupt(32);
    }
}

unsafe fn keyboard_initialize() -> Result<(), ControllerError> {
    // Ska egentligen kolla ifall tangentbord och ps2 kontroller finns men vi antar att det finns

    // Step 3: Disable devices
    controller.disable_keyboard()?;
    controller.disable_mouse()?;

    // Step 4: Flush data buffer
    let _ = controller.read_data();

    // Step 5: Set config
    let mut config = controller.read_config()?;
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
            | ControllerConfigFlags::ENABLE_TRANSLATE,
        false,
    );
    controller.write_config(config)?;

    // Step 6: Controller self-test
    controller.test_controller()?;
    // Write config again in case of controller reset
    controller.write_config(config)?;

    // Step 7: Determine if there are 2 devices
    let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
        controller.enable_mouse()?;
        config = controller.read_config()?;
        // If mouse is working, this should now be unset
        !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
    } else {
        false
    };
    // Disable mouse. If there's no mouse, this is ignored
    controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = controller.test_keyboard().is_ok();
    let _mouse_works = has_mouse && controller.test_mouse().is_ok();

    // Step 9 - 10: Enable and reset devices
    config = controller.read_config()?;
    if keyboard_works {
        controller.enable_keyboard()?;
        config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
        config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
        controller.keyboard().reset_and_self_test().unwrap();
    }

    // Write last configuration to enable devices and interrupts
    controller.write_config(config)?;

    Ok(())
}
