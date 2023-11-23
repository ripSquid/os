use base::{
    input::{keymap, ALT_MODIFIER, CTRL_MODIFIER, KEYBOARD_QUEUE, SHIFT_MODIFIER, Key, KeyEvent},
    pic::pics,
};
use ps2::{error::ControllerError, flags::ControllerConfigFlags, Controller};
use x86_64::structures::idt::InterruptStackFrame;

static mut controller: Controller = unsafe { Controller::with_timeout(10_000) };

#[derive(PartialEq)]
enum State {
    Waiting,
    KeyRelease,
    WaitingForRightCommand,
    RightKeyRelease,
}

struct KeyboardState {
    command: State,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    altgr_pressed: bool,
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

static mut keyboard_state: KeyboardState = KeyboardState {
    command: State::Waiting,
    ctrl_pressed: false,
    shift_pressed: false,
    alt_pressed: false,
    altgr_pressed: false,
};

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    // Implement keyboard 2

    // 0x39 SPACE
    // 0x1C ENTER
    // 0xE  BACKSPACE
    // 0x2A 0xAA SHIFT
    // 0x1D 0x9D CTRL
    // 0x38 0xB8 ALT

    unsafe {
        if let Ok(data) = controller.read_data() {
            match data {
                0x2A => {
                    // Shift Pressed
                    keyboard_state.shift_pressed = true;
                }
                0xAA => {
                    // Shift released
                    keyboard_state.shift_pressed = false;
                }
                0x1D => {
                    // CTRL pressed
                    keyboard_state.ctrl_pressed = true;
                }
                0x9D => {
                    // CTRL released
                    keyboard_state.ctrl_pressed = false;
                }
                0x38 => {
                    // Alt pressed
                    keyboard_state.alt_pressed = true;
                }
                0xB8 => {
                    // Alt released
                    keyboard_state.alt_pressed = false;
                }
                _ => {
                    KEYBOARD_QUEUE.insert(KeyEvent::KeyPressed { modifiers: keyboard_state.get_modifier_usize().into(), key: Key::new(keyboard_state.get_modifier_usize() | data as usize) })
                }
            }
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
    controller.disable_keyboard()?;
    controller.disable_mouse()?;

    // Step 4: Flush data buffer
    let _ = controller.read_data();

    // Step 5: Set config
    let mut config = controller.read_config()?;
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT,
        false,
    );

    config.set(ControllerConfigFlags::ENABLE_TRANSLATE, true);

    controller.write_config(config)?;

    // Step 6: Controller self-test
    controller.test_controller()?;
    // Write config again in case of controller reset
    controller.write_config(config)?;
    // Disable mouse. If there's no mouse, this is ignored
    //controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = controller.test_keyboard().is_ok();
    // Step 9 - 10: Enable and reset devices
    config = controller.read_config()?;
    if keyboard_works {
        controller.enable_keyboard()?;
        config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
        config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
        config.set(ControllerConfigFlags::ENABLE_TRANSLATE, true);
        controller.keyboard().reset_and_self_test().unwrap();
    }

    // Write last configuration to enable devices and interrupts
    controller.write_config(config)?;

    Ok(())
}
