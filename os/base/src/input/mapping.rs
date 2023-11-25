use super::{ScanCode, Modifiers};

pub trait KeyMapper {
    type AdditionalParams;
    type CompleteMapTarget;
    fn map_combined(scan_code: ScanCode, modifers: Modifiers, any: Self::AdditionalParams) -> Self::CompleteMapTarget;
}

pub enum KeyMapEntry {
    Char(char),
    Escape,
    Delete,
    ArrowUp,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
    Tab,  
    Null, 
}
