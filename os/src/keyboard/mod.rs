use crate::{
    display::{macros::debug, KernelDebug, KernelFormatter, STATIC_VGA_WRITER},
    interrupt::setup::{self, pics},
};
use alloc::format;
use alloc::{fmt::format, string::ToString};
use heapless::spsc::Queue;
use ps2::{error::ControllerError, flags::ControllerConfigFlags, Controller};
use x86_64::structures::idt::InterruptStackFrame;

static mut controller: Controller = unsafe { Controller::new() };

static mut keymap: [char; 4096] = ['\x00'; 4096];

const SHIFT_MODIFIER: usize = 0b0100_0000_0000;
const CTRL_MODIFIER: usize = 0b1000_0000_0000;
const ALT_MODIFIER: usize = 0b0010_0000_0000;
const ALTGR_MODIFIER: usize = 0b0001_0000_0000;

pub static mut KEYBOARD_QUEUE: Queue<char, 256> = Queue::new();

pub unsafe fn setup_keymap() {
    // 0000 / 0000 0000
    // Highest 4 bits are for CTRL, SHIFT, ALT, ALTGR
    // Lowest 8 bits are for the character/keycode from keyboard

    keymap[0x1E] = 'a';
    keymap[0x30] = 'b';
    keymap[0x2E] = 'c';
    keymap[0x20] = 'd';
    keymap[0x12] = 'e';
    keymap[0x21] = 'f';
    keymap[0x22] = 'g';
    keymap[0x23] = 'h';
    keymap[0x17] = 'i';
    keymap[0x24] = 'j';
    keymap[0x25] = 'k';
    keymap[0x26] = 'l';
    keymap[0x32] = 'm';
    keymap[0x31] = 'n';
    keymap[0x18] = 'o';
    keymap[0x19] = 'p';
    keymap[0x10] = 'q';
    keymap[0x13] = 'r';
    keymap[0x1F] = 's';
    keymap[0x14] = 't';
    keymap[0x16] = 'u';
    keymap[0x2F] = 'v';
    keymap[0x11] = 'w';
    keymap[0x2D] = 'x';
    keymap[0x15] = 'y';
    keymap[0x2C] = 'z';

    keymap[SHIFT_MODIFIER | 0x1E] = 'A';
    keymap[SHIFT_MODIFIER | 0x30] = 'B';
    keymap[SHIFT_MODIFIER | 0x2E] = 'C';
    keymap[SHIFT_MODIFIER | 0x20] = 'D';
    keymap[SHIFT_MODIFIER | 0x12] = 'E';
    keymap[SHIFT_MODIFIER | 0x21] = 'F';
    keymap[SHIFT_MODIFIER | 0x22] = 'G';
    keymap[SHIFT_MODIFIER | 0x23] = 'H';
    keymap[SHIFT_MODIFIER | 0x17] = 'I';
    keymap[SHIFT_MODIFIER | 0x24] = 'J';
    keymap[SHIFT_MODIFIER | 0x25] = 'K';
    keymap[SHIFT_MODIFIER | 0x26] = 'L';
    keymap[SHIFT_MODIFIER | 0x32] = 'M';
    keymap[SHIFT_MODIFIER | 0x31] = 'N';
    keymap[SHIFT_MODIFIER | 0x18] = 'O';
    keymap[SHIFT_MODIFIER | 0x19] = 'P';
    keymap[SHIFT_MODIFIER | 0x10] = 'Q';
    keymap[SHIFT_MODIFIER | 0x13] = 'R';
    keymap[SHIFT_MODIFIER | 0x1F] = 'S';
    keymap[SHIFT_MODIFIER | 0x14] = 'T';
    keymap[SHIFT_MODIFIER | 0x16] = 'U';
    keymap[SHIFT_MODIFIER | 0x2F] = 'V';
    keymap[SHIFT_MODIFIER | 0x11] = 'W';
    keymap[SHIFT_MODIFIER | 0x2D] = 'X';
    keymap[SHIFT_MODIFIER | 0x15] = 'Y';
    keymap[SHIFT_MODIFIER | 0x2C] = 'Z';

    keymap[0xB] = '0';
    keymap[0x2] = '1';
    keymap[0x3] = '2';
    keymap[0x4] = '3';
    keymap[0x5] = '4';
    keymap[0x6] = '5';
    keymap[0x7] = '6';
    keymap[0x8] = '7';
    keymap[0x9] = '8';
    keymap[0xA] = '9';

    keymap[SHIFT_MODIFIER | 0xB] = '=';
    keymap[SHIFT_MODIFIER | 0x2] = '!';
    keymap[SHIFT_MODIFIER | 0x3] = '"';
    keymap[SHIFT_MODIFIER | 0x4] = '#';
    keymap[SHIFT_MODIFIER | 0x5] = '3';
    keymap[SHIFT_MODIFIER | 0x6] = '%';
    keymap[SHIFT_MODIFIER | 0x7] = '&';
    keymap[SHIFT_MODIFIER | 0x8] = '/';
    keymap[SHIFT_MODIFIER | 0x9] = '(';
    keymap[SHIFT_MODIFIER | 0xA] = ')';

    keymap[ALT_MODIFIER | 0x3] = '@';
    keymap[ALT_MODIFIER | 0x5] = '$';

    keymap[0x0C] = '+';
    keymap[SHIFT_MODIFIER | 0x0C] = '?';
    keymap[ALT_MODIFIER | 0x0C] = '\\';

    keymap[0x33] = ',';
    keymap[0x34] = '.';
    keymap[0x35] = '-';

    keymap[SHIFT_MODIFIER | 0x33] = ';';
    keymap[SHIFT_MODIFIER | 0x34] = ':';
    keymap[SHIFT_MODIFIER | 0x35] = '_';

    keymap[0x2B] = '\'';
    keymap[SHIFT_MODIFIER | 0x2B] = '*';
    keymap[SHIFT_MODIFIER | 0x1B] = '^';

    keymap[0x56] = '<';
    keymap[SHIFT_MODIFIER | 0x56] = '>';
    keymap[ALT_MODIFIER | 0x56] = '|';

    // Space
    keymap[0x39] = ' ';
    // Enter
    keymap[0x1C] = '\x0A';
    // Backspace
    keymap[0x0E] = '\x08';
    // Escape
    keymap[0x01] = '\x1B';

    // 0x39 SPACE
    // 0x1C ENTER
    // 0xE  BACKSPACE
    // 0x2A 0xAA SHIFT
    // 0x1D 0x9D CTRL
    // 0x38 0xB8 ALT
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

impl KeyboardState {
    fn get_modifier_usize(&self) -> usize {
        (if self.ctrl_pressed {CTRL_MODIFIER} else {0}) | (if self.shift_pressed {SHIFT_MODIFIER} else {0}) | (if self.alt_pressed {ALT_MODIFIER} else {0})
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
                },
                0xAA => {
                    // Shift released
                    keyboard_state.shift_pressed = false;
                },
                0x1D => {
                    // CTRL pressed
                    keyboard_state.ctrl_pressed = true;
                },
                0x9D => {
                    // CTRL released
                    keyboard_state.ctrl_pressed = false;
                },
                0x38 => {
                    // Alt pressed
                    keyboard_state.alt_pressed = true;
                },
                0xB8 => {
                    // Alt released
                    keyboard_state.alt_pressed = false;
                },
                _ => {
                    let keymap_entry = keymap[keyboard_state.get_modifier_usize() | data as usize];
                    if keymap_entry != '\0' {
                        //STATIC_VGA_WRITER.write_raw_char(keymap_entry as u8);
                        KEYBOARD_QUEUE.enqueue_unchecked(keymap_entry);
                    } else {
                        //STATIC_VGA_WRITER.write_str(&format!("0x{:X}", data));
                    }
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
    controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = controller.test_keyboard().is_ok();

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
