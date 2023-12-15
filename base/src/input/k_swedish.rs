use super::{mapping::{KeyMapEntry, KeyMapper}, ScanCode, Modifiers};






pub struct SwedishLayoutKeyboardMapper;

impl KeyMapper for SwedishLayoutKeyboardMapper {
    type AdditionalParams = ();

    type CompleteMapTarget = KeyMapEntry;

    fn map_combined(scan_code: ScanCode, modifers: Modifiers, any: Self::AdditionalParams) -> Self::CompleteMapTarget {
        todo!()
    }
}

