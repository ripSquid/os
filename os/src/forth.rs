use alloc::{collections::BTreeMap, format, string::String, vec::Vec};

use crate::display::STATIC_VGA_WRITER;

type ForthFunction = &'static dyn Fn(&ForthMachine, Stack);

type Stack = Vec<StackItem>;
enum StackItem {
    String(String),
    Int(isize),
}

struct ForthMachine {
    words: BTreeMap<String, Vec<String>>,
    implemented_words: BTreeMap<String, ForthFunction>,
    stack: Stack,
}

impl ForthMachine {
    fn print(s: StackItem) {
        match s {
            StackItem::String(s) => unsafe {
                STATIC_VGA_WRITER.write_str(&s);
            },
            StackItem::Int(i) => unsafe {
                STATIC_VGA_WRITER.write_str(&format!("{}", i));
            },
        }
    }
}
