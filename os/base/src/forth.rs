
extern crate alloc;

use alloc::{collections::BTreeMap, string::String, format, vec::Vec};
use crate::display::{DefaultVgaWriter, VgaPalette, VgaPaletteColor, BitmapVgaWriter};

pub type ForthFunction = &'static (dyn Fn(&mut ForthMachine, &mut DefaultVgaWriter) + Sync + Send + 'static);

pub struct Stack(Vec<StackItem>);

impl Stack {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    
    /// a count of how many items are on the stack
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// pop an item onto the stack
    pub fn pop(&mut self) -> Option<StackItem> {
        self.0.pop()
    }
    
    /// pop a series of items off the stack
    /// 
    /// This method returns `None` when (len < N)
    pub fn pop_series<const N: usize>(&mut self) -> Option<[StackItem; N]> {
        if self.0.len() >= N {
            Some(core::array::from_fn(|_|self.pop().unwrap()))
        } else {
            None
        }
    }
    
    /// Push a value onto the stack.
    pub fn push<T: Into<StackItem>>(&mut self, item: T) {
        self.0.push(item.into())
    }

    /// Push a series of items onto the stack.
    pub fn push_series<const N: usize, T: Into<StackItem>>(&mut self, item: [T; N]) {
        for item in item {
            self.0.push(item.into())
        }
    }
}
impl Into<StackItem> for String {
    fn into(self) -> StackItem {
        StackItem::String(self)
    }
}
impl Into<StackItem> for isize {
    fn into(self) -> StackItem {
        StackItem::Int(self)
    }
}
impl Into<StackItem> for i64 {
    fn into(self) -> StackItem {
        StackItem::Int(self as isize)
    }
}


pub enum StackItem {
    String(String),
    Int(isize),
}

pub struct ForthMachine {
    words: BTreeMap<String, Vec<String>>,
    implemented_words: BTreeMap<&'static str, ForthFunction>,
    stack: Stack,
}

fn add(sm: &mut ForthMachine, _formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(x), StackItem::Int(y)) = (x, y) {
        sm.stack.push(StackItem::Int(x+y));
    } 
}


fn sub(sm: &mut ForthMachine, _formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(y), StackItem::Int(x)) = (x, y) {
        sm.stack.push(StackItem::Int(x-y));
    }
}

fn div(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(y), StackItem::Int(x)) = (x, y) {
        if y == 0 {
            formatter.write_str("Tried dividing by zero");
            return;
        }
        sm.stack.push(StackItem::Int(x/y));
    }
}

fn mul(sm: &mut ForthMachine, _formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(y), StackItem::Int(x)) = (x, y) {
        sm.stack.push(StackItem::Int(x*y));
    }
}



fn print(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 1 {
        return;
    }

    if let Some(s) = sm.stack.pop() {
        sm.print(s, formatter);
    }
}

fn wp(sm: &mut ForthMachine, _formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() >= 4 {
        let tmp = (sm.stack.pop().unwrap(), sm.stack.pop().unwrap(), sm.stack.pop().unwrap(), sm.stack.pop().unwrap());
        if let (StackItem::Int(b), StackItem::Int(g), StackItem::Int(r), StackItem::Int(x)) = tmp {
            let mut g_formatter =  unsafe { BitmapVgaWriter::new_unsafe() };
            let palette = VgaPalette::from_array_offset(
                [VgaPaletteColor::from_rgb(r.try_into().unwrap(), g.try_into().unwrap(), b.try_into().unwrap())],
                x.try_into().unwrap(),
            );
            g_formatter.set_palette(palette);
        }
    }
}

impl Default for ForthMachine {
    fn default() -> Self {
        let mut tmp = Self {
            words: BTreeMap::new(),
            implemented_words: BTreeMap::new(),
            stack: Stack::new(),
        };
        tmp.implemented_words.insert("+", &add);
        tmp.implemented_words.insert("-", &sub);
        tmp.implemented_words.insert("/", &div);
        tmp.implemented_words.insert("*", &mul);
        tmp.implemented_words.insert(",", &print);
        tmp.implemented_words.insert("WP", &wp);

        tmp
    }
}

#[derive(PartialEq, Clone, Copy)]
enum WordType {
    Word,
    String,
}
#[derive(PartialEq, Clone, Copy)]
struct ForthWord<'a> {
    str: &'a str,
    kind: WordType,
}
impl<'a> ForthWord<'a> {
    fn with_kind(str: &'a str, kind: WordType) -> Self {
        Self { str, kind }
    }
}



impl ForthMachine {
    pub fn insert_word(&mut self, op: ForthFunction, name: &'static str) {
        self.implemented_words.insert(name, op);
    }
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
    pub fn print(&self, s: StackItem, formatter: &mut DefaultVgaWriter) {
        match s {
            StackItem::String(s) => {
                formatter.write_str(&s);},
            StackItem::Int(i) => 
                {formatter.write_str(&format!("{}", i));},
        }
    }

    pub fn run(&mut self, s: String, formatter: &mut DefaultVgaWriter) {
        let s = {
            s.split('\"').enumerate().flat_map(|(i, e)| {
                let (ty, split) = if i % 2 == 0 {
                    (WordType::Word, e.split(' '))
                } else {
                    (WordType::String, e.split('\"') )
                };
                split.map(move |e| ForthWord::with_kind(e, ty))
            }).collect::<Vec<_>>()
        };
        let mut i = 0;
        let mut new_i = 0;
        while new_i < s.len() {
            i = new_i;
            new_i += 1;

            let word = &s[i];
            let ForthWord { str, kind } = *word;

            if !(str.len() > 0) {
                continue;
            }
            
            if self.implemented_words.contains_key(str) {
                // Run it

                let f = *self.implemented_words.get(str).unwrap();
                f(self, formatter);
            } else if let Ok(x) = isize::from_str_radix(str, 10) {
                self.stack.push(StackItem::Int(x));
            } else if kind == WordType::String { 
                self.stack.push(StackItem::String(String::from(str)));
            }
        }
    }
}


//kolla om en &str har fnuttar runt sig
fn is_quotated_str(word: &str) -> bool {
    let mut chars = word.chars();
    chars.nth(0) == Some('\"') && chars.nth(word.len()-2) == Some('\"')
}
