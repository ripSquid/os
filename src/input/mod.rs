use base::{
    input::{ ALT_MODIFIER, CTRL_MODIFIER, KEYBOARD_QUEUE, SHIFT_MODIFIER, ScanCode, KeyEvent},
    pic::pics,
};
use ps2::{error::ControllerError, flags::ControllerConfigFlags, Controller};
use x86_64::structures::idt::InterruptStackFrame;

static mut CONTROLLER: Controller = unsafe { Controller::with_timeout(10_000) };

#[derive(PartialEq)]
enum State {
    Waiting,
    WaitingWithByte(u8),
}

struct KeyboardState {
    command: State,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
}

impl KeyboardState {
    fn get_modifier_usize(&self) -> usize {
        (if self.ctrl_pressed { CTRL_MODIFIER } else { 0 })
            | (if self.shift_pressed {
                SHIFT_MODIFIER
            } else {
                0
            })
            | (if self.alt_pressed { ALT_MODIFIER } else { 0 })
    }
}

static mut KEYBOARD_STATE: KeyboardState = KeyboardState {
    command: State::Waiting,
    ctrl_pressed: false,
    shift_pressed: false,
    alt_pressed: false,
};

#[inline]
unsafe fn process_data(data: u8) {
    match data {
        0x2A => {
            // Shift Pressed
            KEYBOARD_STATE.shift_pressed = true;
        }
        0xAA => {
            // Shift released
            KEYBOARD_STATE.shift_pressed = false;
        }
        0x1D => {
            // CTRL pressed
            KEYBOARD_STATE.ctrl_pressed = true;
        }
        0x9D => {
            // CTRL released
            KEYBOARD_STATE.ctrl_pressed = false;
        }
        0x38 => {
            // Alt pressed
            KEYBOARD_STATE.alt_pressed = true;
        }
        0xB8 => {
            // Alt released
            KEYBOARD_STATE.alt_pressed = false;
        }
        0xE0 => {
            KEYBOARD_STATE.command = State::WaitingWithByte(0xE0);
            return;
        }
        _ => {
            let pressed = data & 0x80 == 0;
            let key = {
                let addon = match KEYBOARD_STATE.command {
                    State::Waiting => 0,
                    State::WaitingWithByte(byte) => (byte as usize) << 8,
                };
                KEYBOARD_STATE.command = State::Waiting;
                (data as usize & 0b_0111_1111) | addon
            };
            
            match pressed {
                true => KEYBOARD_QUEUE.insert(KeyEvent::KeyPressed { modifiers: KEYBOARD_STATE.get_modifier_usize().into(), key: ScanCode::new(key as usize) }),
                false => KEYBOARD_QUEUE.insert(KeyEvent::KeyReleased { key: ScanCode::new(key as usize) }),
            }
            return;
        }
        
    }
    KEYBOARD_QUEUE.insert(KeyEvent::ModifiersChanged { modifiers: KEYBOARD_STATE.get_modifier_usize().into() })
}

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    // Implement keyboard 2

    // 0x39 SPACE
    // 0x1C ENTER
    // 0xE  BACKSPACE
    // 0x2A 0xAA SHIFT
    // 0x1D 0x9D CTRL
    // 0x38 0xB8 ALT
    unsafe {
        if let Ok(data) = CONTROLLER.read_data() {
            process_data(data)
        }
    }
    //debug!("keyboard handler!");
    unsafe {
        pics.notify_end_of_interrupt(33);
    }
}

pub unsafe fn keyboard_initialize() -> Result<(), ControllerError> {
    // Ska egentligen kolla ifall tangentbord och ps2 kontroller finns men vi antar att det finns

    // Step 3: Disable devices
    CONTROLLER.disable_keyboard()?;
    CONTROLLER.disable_mouse()?;

    // Step 4: Flush data buffer
    let _ = CONTROLLER.read_data();

    // Step 5: Set config
    let mut config = CONTROLLER.read_config()?;
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT,
        false,
    );

    config.set(ControllerConfigFlags::ENABLE_TRANSLATE, true);

    CONTROLLER.write_config(config)?;

    // Step 6: Controller self-test
    CONTROLLER.test_controller()?;
    // Write config again in case of controller reset
    CONTROLLER.write_config(config)?;
    // Disable mouse. If there's no mouse, this is ignored
    //controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = CONTROLLER.test_keyboard().is_ok();
    // Step 9 - 10: Enable and reset devices
    config = CONTROLLER.read_config()?;
    if keyboard_works {
        CONTROLLER.enable_keyboard()?;
        config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
        config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
        config.set(ControllerConfigFlags::ENABLE_TRANSLATE, true);
        CONTROLLER.keyboard().reset_and_self_test().unwrap();
    }

    // Write last configuration to enable devices and interrupts
    CONTROLLER.write_config(config)?;

    Ok(())
}
