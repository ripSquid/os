use alloc::{fmt::format, string::ToString};
use ps2::{Controller, flags::ControllerConfigFlags, error::ControllerError};
use x86_64::structures::idt::InterruptStackFrame;

use crate::{interrupt::setup::pics, display::{macros::debug, KernelDebug}};

static mut controller: Controller = unsafe {Controller::new()};

enum Event {
    Waiting,
    KeyRelease,
}

#[repr(u8)]
enum Key {
    A = 0x1C,
    B = 0x32,
    C = 0x21,
    D = 0x23,
    E = 0x24,
    F = 0x2B,
    G = 0x34,
    H = 0x33,
    I = 0x43,
    J = 0x3B,
    K = 0x42,
    L = 0x4B,
    M = 0x3A,
    N = 0x31,
    O = 0x44,
    P = 0x4D,
    Q = 0x15,
    R = 0x2D,
    S = 0x1B,
    T = 0x2C,
    U = 0x3C,
    V = 0x2A,
    W = 0x1D,
    X = 0x22,
    Y = 0x35,
    Z = 0x1A,
    Å = 0x54,
    Ä = 0x52,
    Ö = 0x4C,
    One = 0x16,
    Two = 0x1E,
    Three = 0x26,
    Four = 0x25,
    Five = 0x2E,
    Six = 0x36,
    Seven = 0x3D,
    Eight = 0x3E,
    Nine = 0x46,
    Zero = 0x45,
}

impl<'a> KernelDebug<'a> for Key {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter.debug_str(self.to_str())
    }
}

impl Key {
    fn to_str(&self) -> &str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Å => "Å",
            Self::Ä => "Ä",
            Self::Ö => "Ö",
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Zero => "0",
            _ => "?"
        }
    }
}

// ABCDEFGHIJKLMNOPQRSTUVWXYZÅÄÖ1234567890

enum KeyEvent {
    KeyDown(Key),
    KeyUp(Key),
}

struct KeyboardState {
    command: Event,
}

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {

    // Implement keyboard 2



    unsafe { if let Ok(data) = controller.read_data() {
        debug!(&data);
    } }
    debug!("keyboard handler!");
    unsafe {
        pics.notify_end_of_interrupt(33);
    }
}


pub unsafe fn keyboard_initialize() -> Result<(), ControllerError> {
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
