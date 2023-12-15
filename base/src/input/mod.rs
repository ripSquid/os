use core::ops::BitOr;
use heapless::spsc::Queue;
pub mod mapping;
pub mod k_swedish;
pub const SHIFT_MODIFIER: usize = 0b0100_0000_0000;
pub const CTRL_MODIFIER: usize = 0b1000_0000_0000;
pub const ALT_MODIFIER: usize = 0b0010_0000_0000;
pub const ALTGR_MODIFIER: usize = 0b0001_0000_0000;

pub static mut KEYBOARD_QUEUE: Keyboard<KeyEvent> = Keyboard::new();
pub static mut KEYMAP: [char; 4096] = ['\x00'; 4096];

#[derive(Copy, Clone, PartialEq)]
pub struct ScanCode(pub usize);

#[derive(Clone, PartialEq)]
pub enum KeyEvent {
    KeyPressed {
        modifiers: Modifiers,
        key: ScanCode,
    },
    KeyReleased {
        key: ScanCode,
    },
    ModifiersChanged {
        modifiers: Modifiers,
    }

}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers(usize);
impl From<usize> for Modifiers {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<ScanCode> for Modifiers {
    fn from(value: ScanCode) -> Self {
        Self(value.0 & 0xF00)
    }
}

impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Modifiers {
    pub const CTRL: Self = Self(CTRL_MODIFIER);
    pub const SHIFT: Self = Self(SHIFT_MODIFIER);
    pub const ALT: Self = Self(ALT_MODIFIER);

    pub const fn combine(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub fn is_ctrl_pressed(&self) -> bool {
        (self.0 & CTRL_MODIFIER) > 0
    }
    pub fn new(u: usize) -> Self {
        Self(u)
    }

    pub fn is_shift_pressed(&self) -> bool {
        (self.0 & SHIFT_MODIFIER) > 0
    }

    pub fn is_alt_pressed(&self) -> bool {
        (self.0 & ALT_MODIFIER) > 0
    }
}

impl ScanCode {
    pub fn key_modifiers(self) -> Modifiers {
        self.into()
    }

    pub fn as_char(self) -> char {
        self.into()
    }

    pub fn new(u: usize) -> Self {
        Self(u)
    }
    pub fn resolve_text_char(self, modifiers: Modifiers) -> Option<char> {
        let char = unsafe  {if modifiers.is_shift_pressed() {
            KEYMAP.get(SHIFT_MODIFIER | self.0).cloned()
        } else {
            KEYMAP.get(self.0).cloned()
        }};
        char.map(|char| (char !='\0').then_some(char)).flatten()
    }
}

impl Into<char> for ScanCode {
    fn into(self) -> char {
        unsafe {KEYMAP[self.0]}
    }
}
/* 
impl Into<Option<char>> for ScanCode {
    fn into(self) -> Option<char> {
        let key = unsafe {keymap[self.0]};
        (key != '\0').then_some(key)
    }
}
*/
pub struct Keyboard<T> {
    queue: Queue<T, 256>,
}
impl Keyboard<KeyEvent> {
    pub fn getch_blocking(&mut self) -> char {
        loop {
            let Some(KeyEvent::KeyPressed { key, modifiers}) = self.queue.dequeue() else {continue};
            return key.resolve_text_char(modifiers).unwrap_or('\0');
        }
    }
    pub fn try_getch(&mut self) -> Option<(ScanCode, Modifiers)> {
        self.queue.dequeue().map(|v| if let KeyEvent::KeyPressed { key, modifiers } = v { Some((key, modifiers))} else {None}).flatten()
    }
    pub fn try_getch_char(&mut self) -> Option<char> {
        self.try_getch().map(|(x, y)| x.resolve_text_char(y)).flatten()
    }
}
impl<T> Keyboard<T> {
    pub fn get(&mut self) -> Option<T> {
        self.queue.dequeue()
    }
    pub fn get_blocking(&mut self) -> T {
        loop {
            if let Some(valid) = self.queue.dequeue() {
                return valid;
            }
        }
    }
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }
    pub fn insert(&mut self, data: T) {
        unsafe {
            self.queue.enqueue_unchecked(data);
        }
    }
}

pub unsafe fn setup_keymap() {
    // 0000 / 0000 0000
    // Highest 4 bits are for CTRL, SHIFT, ALT, ALTGR
    // Lowest 8 bits are for the character/keycode from keyboard

    KEYMAP[0x1E] = 'a';
    KEYMAP[0x30] = 'b';
    KEYMAP[0x2E] = 'c';
    KEYMAP[0x20] = 'd';
    KEYMAP[0x12] = 'e';
    KEYMAP[0x21] = 'f';
    KEYMAP[0x22] = 'g';
    KEYMAP[0x23] = 'h';
    KEYMAP[0x17] = 'i';
    KEYMAP[0x24] = 'j';
    KEYMAP[0x25] = 'k';
    KEYMAP[0x26] = 'l';
    KEYMAP[0x32] = 'm';
    KEYMAP[0x31] = 'n';
    KEYMAP[0x18] = 'o';
    KEYMAP[0x19] = 'p';
    KEYMAP[0x10] = 'q';
    KEYMAP[0x13] = 'r';
    KEYMAP[0x1F] = 's';
    KEYMAP[0x14] = 't';
    KEYMAP[0x16] = 'u';
    KEYMAP[0x2F] = 'v';
    KEYMAP[0x11] = 'w';
    KEYMAP[0x2D] = 'x';
    KEYMAP[0x15] = 'y';
    KEYMAP[0x2C] = 'z';

    KEYMAP[0x1A] = 'å';
    KEYMAP[0x27] = 'ö';
    KEYMAP[0x28] = 'ä';

    KEYMAP[SHIFT_MODIFIER | 0x1A] = 'Å';
    KEYMAP[SHIFT_MODIFIER | 0x27] = 'Ö';
    KEYMAP[SHIFT_MODIFIER | 0x28] = 'Ä';

    KEYMAP[SHIFT_MODIFIER | 0x1E] = 'A';
    KEYMAP[SHIFT_MODIFIER | 0x30] = 'B';
    KEYMAP[SHIFT_MODIFIER | 0x2E] = 'C';
    KEYMAP[SHIFT_MODIFIER | 0x20] = 'D';
    KEYMAP[SHIFT_MODIFIER | 0x12] = 'E';
    KEYMAP[SHIFT_MODIFIER | 0x21] = 'F';
    KEYMAP[SHIFT_MODIFIER | 0x22] = 'G';
    KEYMAP[SHIFT_MODIFIER | 0x23] = 'H';
    KEYMAP[SHIFT_MODIFIER | 0x17] = 'I';
    KEYMAP[SHIFT_MODIFIER | 0x24] = 'J';
    KEYMAP[SHIFT_MODIFIER | 0x25] = 'K';
    KEYMAP[SHIFT_MODIFIER | 0x26] = 'L';
    KEYMAP[SHIFT_MODIFIER | 0x32] = 'M';
    KEYMAP[SHIFT_MODIFIER | 0x31] = 'N';
    KEYMAP[SHIFT_MODIFIER | 0x18] = 'O';
    KEYMAP[SHIFT_MODIFIER | 0x19] = 'P';
    KEYMAP[SHIFT_MODIFIER | 0x10] = 'Q';
    KEYMAP[SHIFT_MODIFIER | 0x13] = 'R';
    KEYMAP[SHIFT_MODIFIER | 0x1F] = 'S';
    KEYMAP[SHIFT_MODIFIER | 0x14] = 'T';
    KEYMAP[SHIFT_MODIFIER | 0x16] = 'U';
    KEYMAP[SHIFT_MODIFIER | 0x2F] = 'V';
    KEYMAP[SHIFT_MODIFIER | 0x11] = 'W';
    KEYMAP[SHIFT_MODIFIER | 0x2D] = 'X';
    KEYMAP[SHIFT_MODIFIER | 0x15] = 'Y';
    KEYMAP[SHIFT_MODIFIER | 0x2C] = 'Z';

    KEYMAP[0xB] = '0';
    KEYMAP[0x2] = '1';
    KEYMAP[0x3] = '2';
    KEYMAP[0x4] = '3';
    KEYMAP[0x5] = '4';
    KEYMAP[0x6] = '5';
    KEYMAP[0x7] = '6';
    KEYMAP[0x8] = '7';
    KEYMAP[0x9] = '8';
    KEYMAP[0xA] = '9';

    KEYMAP[SHIFT_MODIFIER | 0xB] = '=';
    KEYMAP[SHIFT_MODIFIER | 0x2] = '!';
    KEYMAP[SHIFT_MODIFIER | 0x3] = '"';
    KEYMAP[SHIFT_MODIFIER | 0x4] = '#';
    KEYMAP[SHIFT_MODIFIER | 0x5] = '3';
    KEYMAP[SHIFT_MODIFIER | 0x6] = '%';
    KEYMAP[SHIFT_MODIFIER | 0x7] = '&';
    KEYMAP[SHIFT_MODIFIER | 0x8] = '/';
    KEYMAP[SHIFT_MODIFIER | 0x9] = '(';
    KEYMAP[SHIFT_MODIFIER | 0xA] = ')';

    KEYMAP[ALT_MODIFIER | 0x3] = '@';
    KEYMAP[ALT_MODIFIER | 0x5] = '$';

    KEYMAP[0x0C] = '+';
    KEYMAP[SHIFT_MODIFIER | 0x0C] = '?';
    KEYMAP[ALT_MODIFIER | 0x0C] = '\\';

    KEYMAP[0x33] = ',';
    KEYMAP[0x34] = '.';
    KEYMAP[0x35] = '-';

    KEYMAP[SHIFT_MODIFIER | 0x33] = ';';
    KEYMAP[SHIFT_MODIFIER | 0x34] = ':';
    KEYMAP[SHIFT_MODIFIER | 0x35] = '_';

    KEYMAP[0x2B] = '\'';
    KEYMAP[SHIFT_MODIFIER | 0x2B] = '*';
    KEYMAP[SHIFT_MODIFIER | 0x1B] = '^';

    KEYMAP[0x56] = '<';
    KEYMAP[SHIFT_MODIFIER | 0x56] = '>';
    KEYMAP[ALT_MODIFIER | 0x56] = '|';

    // Space
    KEYMAP[0x39] = ' ';
    KEYMAP[SHIFT_MODIFIER | 0x39] = ' ';
    // Enter
    KEYMAP[0x1C] = '\x0A';
    // Backspace
    KEYMAP[0x0E] = '\x08';
    // Escape
    KEYMAP[0x01] = '\x1B';

    // 0x39 SPACE
    // 0x1C ENTER
    // 0xE  BACKSPACE
    // 0x2A 0xAA SHIFT
    // 0x1D 0x9D CTRL
    // 0x38 0xB8 ALT
}
