use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use alloc::vec;

use crate::display::STATIC_VGA_WRITER;


type ForthFunction = &'static dyn Fn(&mut ForthMachine);

type Stack = Vec<StackItem>;
pub enum StackItem {
    String(String),
    Int(isize),
}

pub struct ForthMachine {
    words: BTreeMap<String, Vec<String>>,
    implemented_words: BTreeMap<&'static str, ForthFunction>,
    stack: Stack,
}

fn add(sm: &mut ForthMachine) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(x), StackItem::Int(y)) = (x, y) {
        sm.stack.push(StackItem::Int(x+y));
    } 
}

fn print(sm: &mut ForthMachine) {
    if sm.stack.len() < 1 {
        return;
    }

    if let Some(s) = sm.stack.pop() {
        sm.print(s);
    }
}

impl ForthMachine {
    pub fn print(&self, s: StackItem) {
        match s {
            StackItem::String(s) => unsafe {
                STATIC_VGA_WRITER.write_str(&s);
            },
            StackItem::Int(i) => unsafe {
                STATIC_VGA_WRITER.write_str(&format!("{}", i));
            },
        }
    }

    pub fn default() -> Self {
        let mut tmp = Self {
            words: BTreeMap::new(),
            implemented_words: BTreeMap::new(),
            stack: vec![],
        };
        tmp.implemented_words.insert("+".into(), &add);
        tmp.implemented_words.insert(",".into(), &print);
        tmp
    }

    pub fn run(&mut self, s:&Vec<&str>) {
        for word in s {
            //self.print(StackItem::String(String::from("z")));
            if let Ok(x) = isize::from_str_radix(word, 10) {
                self.stack.push(StackItem::Int(x));
                continue;
            }
            //self.print(StackItem::String(String::from("y")));
            if self.implemented_words.contains_key(word) {
                // Run it
                //self.print(StackItem::String(String::from("x")));
                let f = *self.implemented_words.get(word).unwrap();
                f(self);
            }
        }
    }
}
