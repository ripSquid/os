use alloc::boxed::Box;
use alloc::fmt::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use alloc::vec;

use base::display::DefaultVgaWriter;
use fs::{LittleManApp, InstallableApp, Path, AppConstructor, StartError, OsHandle};
use spin::rwlock::RwLock;
use crate::display::{STATIC_VGA_WRITER, BitmapVgaWriter, VgaPalette, VgaPaletteColor};

#[derive(Default)]
pub struct ForthCon {
    fm: Arc<RwLock<ForthMachine>>
}

impl InstallableApp for ForthCon {
    fn install() -> (Path, Box<dyn AppConstructor>) {
        (Path::from("forth"), Box::new(ForthCon::default())) 
    }
}
impl AppConstructor for ForthCon {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ForthApp(Vec::new(),self.fm.clone()))
    }
}

pub struct ForthApp(Vec<String>, Arc<RwLock<ForthMachine>>);

impl LittleManApp for ForthApp {
    fn start(&mut self, args: &[&str]) -> Result<(), StartError> {
        self.0 = args.iter().map(|arg| arg.to_string()).collect();
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        if let Ok(g) = handle.text_mode_formatter() {
            let segments: Vec<&str> = self.0.iter().map(|s| s.as_str()).collect();
            self.1.write().run(&segments, g);
        }
        handle.call_exit();
    }
}

type ForthFunction = &'static (dyn Fn(&mut ForthMachine, &mut DefaultVgaWriter) + Sync + Send + 'static);

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

fn add(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
    if sm.stack.len() < 2 {
        return;
    }
    let x = sm.stack.pop().unwrap();
    let y = sm.stack.pop().unwrap();
    if let (StackItem::Int(x), StackItem::Int(y)) = (x, y) {
        sm.stack.push(StackItem::Int(x+y));
    } 
}


fn sub(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
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

fn mul(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
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

fn wp(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter) {
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
            stack: vec![],
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

impl ForthMachine {
    pub fn print(&self, s: StackItem, formatter: &mut DefaultVgaWriter) {
        match s {
            StackItem::String(s) => {
                formatter.write_str(&s);},
            StackItem::Int(i) => 
                {formatter.write_str(&format!("{}", i));},
        }
    }

    pub fn run(&mut self, s: &[&str], formatter: &mut DefaultVgaWriter) {
        let mut i = 0;
        let mut new_i = 0;
        while new_i < s.len() {
            i = new_i;
            new_i += 1;

            let word = s[i];

            if let Ok(x) = isize::from_str_radix(word, 10) {
                self.stack.push(StackItem::Int(x));
                continue;
            }

            if self.implemented_words.contains_key(word) {
                // Run it

                let f = *self.implemented_words.get(word).unwrap();
                f(self, formatter);
            }
        }
    }
}
