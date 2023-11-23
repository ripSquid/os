use heapless::spsc::Queue;
pub mod mapping;
pub mod k_swedish;
pub const SHIFT_MODIFIER: usize = 0b0100_0000_0000;
pub const CTRL_MODIFIER: usize = 0b1000_0000_0000;
pub const ALT_MODIFIER: usize = 0b0010_0000_0000;
pub const ALTGR_MODIFIER: usize = 0b0001_0000_0000;

pub static mut KEYBOARD_QUEUE: Keyboard<KeyEvent> = Keyboard::new();
pub static mut keymap: [char; 4096] = ['\x00'; 4096];

#[derive(Copy, Clone)]
pub struct ScanCode(pub usize);

pub enum KeyEvent {
    KeyPressed {
        modifiers: KeyModifier,
        key: ScanCode,
    },
    KeyReleased {
        key: ScanCode,
    },
    ModifiersChanged {
        modifiers: KeyModifier,
    }

}

pub struct KeyModifier(usize);
impl From<usize> for KeyModifier {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<ScanCode> for KeyModifier {
    fn from(value: ScanCode) -> Self {
        Self(value.0 & 0xF00)
    }
}

impl KeyModifier {
    pub fn is_ctrl_pressed(&self) -> bool {
        (self.0 & CTRL_MODIFIER) > 0
    }

    pub fn is_shift_pressed(&self) -> bool {
        (self.0 & SHIFT_MODIFIER) > 0
    }

    pub fn is_alt_pressed(&self) -> bool {
        (self.0 & ALT_MODIFIER) > 0
    }
}

impl ScanCode {
    pub fn key_modifiers(self) -> KeyModifier {
        self.into()
    }

    pub fn as_char(self) -> char {
        self.into()
    }

    pub fn new(u: usize) -> Self {
        Self(u)
    }
    pub fn resolve_text_char(self, modifiers: KeyModifier) -> Option<char> {
        let char = unsafe  {if modifiers.is_shift_pressed() {
             keymap[SHIFT_MODIFIER | self.0]
            
        } else {
            keymap[self.0]
        }};
        (char != '\0').then_some(char)
    }
}

impl Into<char> for ScanCode {
    fn into(self) -> char {
        unsafe {keymap[self.0]}
    }
}

impl Into<Option<char>> for ScanCode {
    fn into(self) -> Option<char> {
        let key = unsafe {keymap[self.0]};
        (key != '\0').then_some(key)
    }
}

pub struct Keyboard<T> {
    queue: Queue<T, 256>,
}
impl Keyboard<KeyEvent> {
    pub fn getch_blocking(&mut self) -> ScanCode {
        loop {
            if let Some(KeyEvent::KeyPressed { key, ..}) = self.queue.dequeue() {
                return key
            }
        }
    }
    pub fn try_getch(&mut self) -> Option<ScanCode> {
        self.queue.dequeue().map(|v| if let KeyEvent::KeyPressed { key, .. } = v { Some(key)} else {None}).flatten().into()
    }
    pub fn try_getch_char(&mut self) -> Option<char> {
        self.try_getch().map(|x| x.into()).flatten()
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

    keymap[0x1A] = 'å';
    keymap[0x27] = 'ö';
    keymap[0x28] = 'ä';

    keymap[SHIFT_MODIFIER | 0x1A] = 'Å';
    keymap[SHIFT_MODIFIER | 0x27] = 'Ö';
    keymap[SHIFT_MODIFIER | 0x28] = 'Ä';

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
