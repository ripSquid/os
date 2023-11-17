use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use alloc::vec;

use fs::{LittleManApp, InstallableApp, Path, AppConstructor, StartError, OsHandle};
use crate::display::{STATIC_VGA_WRITER, BitmapVgaWriter, VgaPalette, VgaPaletteColor};

impl InstallableApp for ForthCon {
    fn install() -> (Path, Box<dyn AppConstructor>) {
        (Path::from("forth"), Box::new(ForthCon)) 
    }
}
pub struct ForthCon;
impl AppConstructor for ForthCon {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ForthApp(Vec::new()))
    }
}

pub struct ForthApp(Vec<String>);

impl LittleManApp for ForthApp {
    fn start(&mut self, args: &[&str]) -> Result<(), StartError> {
        self.0 = args.iter().map(|arg| arg.to_string()).collect();
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        if let Ok(_) = handle.text_mode_formatter() {
            let mut fm = ForthMachine::default();
            let segments: Vec<&str> = self.0.iter().map(|s| s.as_str()).collect();
            fm.run(&segments)
        }
        handle.call_exit();
    }
}

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

fn wp(sm: &mut ForthMachine) {
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
        tmp.implemented_words.insert("+", &add);
        tmp.implemented_words.insert(",", &print);
        tmp.implemented_words.insert("WP", &wp);
        tmp
    }

    pub fn run(&mut self, s:&[&str]) {
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
