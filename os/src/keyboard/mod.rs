use alloc::{fmt::format, string::ToString};
use ps2::{Controller, flags::ControllerConfigFlags, error::ControllerError};
use x86_64::structures::idt::InterruptStackFrame;
use alloc::format;
use crate::{interrupt::setup::{pics, self}, display::{macros::debug, KernelDebug}};

static mut controller: Controller = unsafe {Controller::new()};

static mut keymap: [char; 4096] = ['\x00'; 4096];

const SHIFT_MODIFIER: usize = 0b0100_0000_0000;
const CTRL_MODIFIER:  usize = 0b1000_0000_0000;
const ALT_MODIFIER:   usize = 0b0010_0000_0000;
const ALTGR_MODIFIER: usize = 0b0001_0000_0000;

pub unsafe fn setup_keymap() {
    // 0000 / 0000 0000
    // Highest 4 bits are for CTRL, SHIFT, ALT, ALTGR
    // Lowest 8 bits are for the character/keycode from keyboard
    
    keymap[0x1C] = 'a';
    keymap[0x32] = 'b';
    keymap[0x21] = 'c';
    keymap[0x23] = 'd';
    keymap[0x24] = 'e';
    keymap[0x2B] = 'f';
    keymap[0x34] = 'g';
    keymap[0x33] = 'h';
    keymap[0x43] = 'i';
    keymap[0x3B] = 'j';
    keymap[0x42] = 'k';
    keymap[0x4B] = 'l';
    keymap[0x3A] = 'm';
    keymap[0x31] = 'n';
    keymap[0x44] = 'o';
    keymap[0x4D] = 'p';
    keymap[0x15] = 'q';
    keymap[0x2D] = 'r';
    keymap[0x1B] = 's';
    keymap[0x2C] = 't';
    keymap[0x3C] = 'u';
    keymap[0x2A] = 'v';
    keymap[0x1D] = 'w';
    keymap[0x22] = 'x';
    keymap[0x35] = 'y';
    keymap[0x1A] = 'Z';
    keymap[0x54] = 'å';
    keymap[0x52] = 'ä';
    keymap[0x4C] = 'ö';

    keymap[SHIFT_MODIFIER | 0x1C] = 'A';
    keymap[SHIFT_MODIFIER | 0x32] = 'B';
    keymap[SHIFT_MODIFIER | 0x21] = 'C';
    keymap[SHIFT_MODIFIER | 0x23] = 'D';
    keymap[SHIFT_MODIFIER | 0x24] = 'E';
    keymap[SHIFT_MODIFIER | 0x2B] = 'F';
    keymap[SHIFT_MODIFIER | 0x34] = 'G';
    keymap[SHIFT_MODIFIER | 0x33] = 'H';
    keymap[SHIFT_MODIFIER | 0x43] = 'I';
    keymap[SHIFT_MODIFIER | 0x3B] = 'J';
    keymap[SHIFT_MODIFIER | 0x42] = 'K';
    keymap[SHIFT_MODIFIER | 0x4B] = 'L';
    keymap[SHIFT_MODIFIER | 0x3A] = 'M';
    keymap[SHIFT_MODIFIER | 0x31] = 'N';
    keymap[SHIFT_MODIFIER | 0x44] = 'O';
    keymap[SHIFT_MODIFIER | 0x4D] = 'P';
    keymap[SHIFT_MODIFIER | 0x15] = 'Q';
    keymap[SHIFT_MODIFIER | 0x2D] = 'R';
    keymap[SHIFT_MODIFIER | 0x1B] = 'S';
    keymap[SHIFT_MODIFIER | 0x2C] = 'T';
    keymap[SHIFT_MODIFIER | 0x3C] = 'U';
    keymap[SHIFT_MODIFIER | 0x2A] = 'V';
    keymap[SHIFT_MODIFIER | 0x1D] = 'W';
    keymap[SHIFT_MODIFIER | 0x22] = 'X';
    keymap[SHIFT_MODIFIER | 0x35] = 'Y';
    keymap[SHIFT_MODIFIER | 0x1A] = 'Z';
    keymap[SHIFT_MODIFIER | 0x54] = 'Å';
    keymap[SHIFT_MODIFIER | 0x52] = 'Ä';
    keymap[SHIFT_MODIFIER | 0x4C] = 'Ö';

    keymap[0x16] = '1';
    keymap[0x1E] = '2';
    keymap[0x26] = '3';
    keymap[0x25] = '4';
    keymap[0x2E] = '5';
    keymap[0x36] = '6';
    keymap[0x3D] = '7';
    keymap[0x3E] = '8';
    keymap[0x46] = '9';
    keymap[0x45] = '0';

    keymap[SHIFT_MODIFIER | 0x16] = '!';
    keymap[SHIFT_MODIFIER | 0x1E] = '"';
    keymap[SHIFT_MODIFIER | 0x26] = '#';
    keymap[SHIFT_MODIFIER | 0x25] = '¤';
    keymap[SHIFT_MODIFIER | 0x2E] = '%';
    keymap[SHIFT_MODIFIER | 0x36] = '&';
    keymap[SHIFT_MODIFIER | 0x3D] = '/';
    keymap[SHIFT_MODIFIER | 0x3E] = '(';
    keymap[SHIFT_MODIFIER | 0x46] = ')';
    keymap[SHIFT_MODIFIER | 0x45] = '=';

    keymap[ALTGR_MODIFIER | 0x1E] = '@';
    keymap[ALTGR_MODIFIER | 0x26] = '£';
    keymap[ALTGR_MODIFIER | 0x25] = '$';
    keymap[ALTGR_MODIFIER | 0x3D] = '{';
    keymap[ALTGR_MODIFIER | 0x3E] = '[';
    keymap[ALTGR_MODIFIER | 0x46] = ']';
    keymap[ALTGR_MODIFIER | 0x45] = '}';
    keymap[ALTGR_MODIFIER | 0x4E] = '\\';
    keymap[ALTGR_MODIFIER | 0x61] = '|';

    keymap[0x41] = ',';
    keymap[0x49] = '.';
    keymap[0x4A] = '-';
    keymap[SHIFT_MODIFIER | 0x41] = ';';
    keymap[SHIFT_MODIFIER | 0x49] = ':';
    keymap[SHIFT_MODIFIER | 0x4A] = '_';

    keymap[0x5D] = '\'';
    keymap[SHIFT_MODIFIER | 0x5D] = '*';
    keymap[0x5B] = '¨';
    keymap[SHIFT_MODIFIER |  0x5B] = '^';
    keymap[0x4E] = '+'; 
    keymap[SHIFT_MODIFIER | 0x4E] = '?';
    keymap[0x61] = '<';
    keymap[SHIFT_MODIFIER | 0x61] = '>';


}

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

static mut keyboard_state: KeyboardState = KeyboardState {command: State::Waiting, ctrl_pressed: false, shift_pressed: false, alt_pressed: false, altgr_pressed: false};

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {

    // Implement keyboard 2

    unsafe { if let Ok(data) = controller.read_data() {
        if data == 0xF0 && keyboard_state.command == State::Waiting {
            // Start of KeyRelease event
            keyboard_state.command = State::KeyRelease;
        } else if data == 0xE0 && keyboard_state.command == State::Waiting {
            // We got indication that a "right" key is getting specific commands
            keyboard_state.command = State::WaitingForRightCommand;
        } else if data == 0xF0 && keyboard_state.command == State::WaitingForRightCommand {
            // We got a key release event after receiving a "right" key
            keyboard_state.command = State::RightKeyRelease;
        } else if keyboard_state.command == State::KeyRelease {
            // If we are waiting for KeyRelease event then check if its modifier if so switch it back off
            if data == 0x12 {
                keyboard_state.shift_pressed = false;
            } else if data == 0x11 {
                keyboard_state.alt_pressed = false;
            }
            keyboard_state.command = State::Waiting;
        } else if keyboard_state.command == State::RightKeyRelease {
            if data == 0x11 {
                keyboard_state.altgr_pressed = false;
            }
            keyboard_state.command = State::Waiting;
        } else {
            if data == 0x11 && keyboard_state.command == State::WaitingForRightCommand {
                keyboard_state.altgr_pressed = true;
                keyboard_state.command = State::Waiting;
            } else if data == 0x11 {
                keyboard_state.alt_pressed = true;
            }
            if data == 0x12 {
                // If shift gets pressed then switch it on
                keyboard_state.shift_pressed = true;
            } else {
                let mut key: usize = data as usize;
                if keyboard_state.shift_pressed {
                    key |= SHIFT_MODIFIER;
                }
                if keyboard_state.altgr_pressed {
                    key |= ALTGR_MODIFIER;
                }
                if keyboard_state.alt_pressed {
                    key |= ALT_MODIFIER;
                }
                if keyboard_state.ctrl_pressed {
                    key |= CTRL_MODIFIER;
                }

                if keymap[key] != '\x00' {
                    debug!(&keymap[key]);
                } else {
                    debug!(&key);
                }
            }
        }
    } }
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
