use core::{fmt::Display, arch::x86_64};
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use crate::display::{DefaultVgaWriter, UniversalVgaFormatter};
pub type ForthFunction = &'static (dyn Fn(&mut ForthMachine) + Sync + Send + 'static);

fn forth_print(fm: &mut ForthMachine) {
    if let Some(x) = fm.stack.pop() {
        fm.formatter.write_str(&format!("{}", x));
    }
}

fn forth_dup(fm: &mut ForthMachine) {
    if let Some(x) = fm.stack.pop() {
        fm.stack.push(x.clone());
        fm.stack.push(x);
    }
}

// Bottom Top
// Top Bottom
fn forth_swap(fm: &mut ForthMachine) {
    if fm.stack.0.len() >= 2 {
        let top = fm.stack.pop().unwrap();
        let bottom = fm.stack.pop().unwrap();
        fm.stack.push(top);
        fm.stack.push(bottom);
    }
}

fn forth_rot(fm: &mut ForthMachine) {
    if fm.stack.0.len() >= 3 {
        let top = fm.stack.pop().unwrap();
        let middle = fm.stack.pop().unwrap();
        let bottom = fm.stack.pop().unwrap();
        fm.stack.push(middle);
        fm.stack.push(top);
        fm.stack.push(bottom);
    }
}

fn forth_drop(fm: &mut ForthMachine) {
    if fm.stack.0.len() > 0 {
        let _ = fm.stack.pop().unwrap();
    }
}

fn forth_over(fm: &mut ForthMachine) {
    if fm.stack.0.len() >= 2 {
        fm.stack.push(fm.stack.0[fm.stack.0.len() - 2].clone());
    }
}

fn forth_debug(fm: &mut ForthMachine) {
    fm.formatter.write_str(&format!("{:#?}", fm.stack.0));
}

fn forth_new_word(fm: &mut ForthMachine) {
    let mut defined_as = ForthInstructions::default();
    let instruction_name = if let Some(ForthInstruction::Word(s)) = fm.instructions.get(fm.instruction_counter) {s.clone()} else {return;};
    let mut instruction_counter = fm.instruction_counter + 1;
    let mut complete_instruction = false;
    while let Some(instruction) = fm.instructions.get(instruction_counter)  {
        match instruction.clone() {
            ForthInstruction::Word(s) => {
                if s == ":" {
                    complete_instruction = true;
                    break;
                } else {
                    defined_as.0.push(ForthInstruction::Word(s));
                }
            },
            ForthInstruction::Data(x) => {
                defined_as.0.push(ForthInstruction::Data(x));
            }
        }

        instruction_counter += 1;

    }

    if complete_instruction {
        fm.words.insert(instruction_name, defined_as);
        fm.instruction_counter = instruction_counter + 1;
    }
}

fn forth_add(fm: &mut ForthMachine) {
    if let Some((x, y)) = fm.stack.try_pop_two_ints() {
        fm.stack.push(StackItem::Int(x+y));
    }
}

fn forth_mul(fm: &mut ForthMachine) {
    if let Some((x, y)) = fm.stack.try_pop_two_ints() {
        fm.stack.push(StackItem::Int(x*y));
    }
}

fn forth_sub(fm: &mut ForthMachine) {
    if let Some((x, y)) = fm.stack.try_pop_two_ints() {
        fm.stack.push(StackItem::Int(x-y));
    }
}

fn forth_div(fm: &mut ForthMachine) {
    if let Some((x, y)) = fm.stack.try_pop_two_ints() {
        fm.stack.push(StackItem::Int(x/y));
    }
}

fn forth_mod(fm: &mut ForthMachine) {
    if let Some((x, y)) = fm.stack.try_pop_two_ints() {
        fm.stack.push(StackItem::Int(x % y));
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum StackItem {
    String(String),
    Int(isize),
}
impl Display for StackItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::String(x) => x.fmt(f),
            Self::Int(i) => i.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForthInstructions(Vec<ForthInstruction>);

impl Default for ForthInstructions {
    fn default() -> Self {
        Self(alloc::vec![])
    }
}
impl ForthInstructions {
    pub fn add_instructions_to_end(&mut self, new_data: &Vec<char>) {
        let mut parsed_instructions = ForthInstructions::default();
        let mut i = 0;
        let mut word = String::new();
        let mut string_mode = false;

        while i < new_data.len() {
            let prev_char = if i > 0 { new_data[i - 1] } else { '\0' };
            let c = new_data[i];

            if c == ' ' && string_mode == false {
                // Word ends or data ends
                // Parse word/data into ForthInstruction then
                if word.len() > 0 {
                    parsed_instructions.0.push(word.into());
                    word = String::new();
                }
            } else if i + 1 == new_data.len() && string_mode == false {
                // Last word doesnt have space after it
                word.push(c);
                parsed_instructions.0.push(word.into());
                word = String::new();
            } else {
                if c == '"' && prev_char != '\\' {
                    // String mode flips
                    string_mode = !string_mode;
                    if string_mode == false {
                        // Save it
                        parsed_instructions
                            .0
                            .push(ForthInstruction::Data(StackItem::String(word)));
                        word = String::new();
                    }
                } else if c == '"' && prev_char == '\\' {
                    word.pop();
                    word.push(c);
                } else {
                    word.push(c);
                }
            }

            // \" Hello \" "df

            i += 1;
        }
        self.0.append(&mut parsed_instructions.0);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, u: usize) -> Option<&ForthInstruction> {
        self.0.get(u)
    }

    fn iter(&self) -> core::slice::Iter<'_, ForthInstruction> {
        self.0.iter()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ForthInstruction {
    Data(StackItem),
    Word(String),
}
impl From<String> for ForthInstruction {
    fn from(word: String) -> Self {
        if let Ok(i) = isize::from_str_radix(&word, 10) {
            ForthInstruction::Data(StackItem::Int(i))
        } else {
            ForthInstruction::Word(word)
        }
    }
}
impl TryFrom<StackItem> for String {
    type Error = StackItem;

    fn try_from(value: StackItem) -> Result<Self, Self::Error> {
        match value {
            StackItem::String(string) => Ok(string),
            invalid => Err(invalid),
        }
    }
}
impl TryFrom<StackItem> for isize {
    type Error = StackItem;

    fn try_from(value: StackItem) -> Result<Self, Self::Error> {
        match value {
            StackItem::Int(i) => Ok(i),
            invalid => Err(invalid),
        }
    }
}
#[derive(Default)]
pub struct Stack(Vec<StackItem>);
impl Stack {
    pub fn pop(&mut self) -> Option<StackItem> {
        self.0.pop()
    }

    pub fn push(&mut self, s: StackItem) {
        self.0.push(s);
    }
    pub fn try_pop<T: TryFrom<StackItem, Error = StackItem>>(&mut self) -> Option<T> {
        match T::try_from(self.pop()?) {
            Ok(valid) => Some(valid),
            Err(invalid) => {
                self.push(invalid);
                None
            }
        }
    }

    pub fn try_pop_two_ints(&mut self) -> Option<(isize, isize)> {
        if self.0.len() >= 2 {
            return None;
        }
        let x: StackItem = self.pop().unwrap();
        let y: StackItem = self.pop().unwrap();

        if let StackItem::Int(x) = x {
            if let StackItem::Int(y) = y {
                return Some((x, y));
            } else {
                self.push(y);
                self.push(StackItem::Int(x));
            }
        } else {
            self.push(y);
            self.push(x);
        }
        None
    }
}

pub struct ForthMachine {
    pub instruction_counter: usize,
    pub instructions: ForthInstructions,
    pub stack: Stack,
    words: BTreeMap<String, ForthInstructions>,
    default_words: BTreeMap<&'static str, ForthFunction>,
    pub formatter: UniversalVgaFormatter,
}
impl Default for ForthMachine {
    fn default() -> Self {
        let default_words = {
            let mut map: BTreeMap<&'static str, ForthFunction> = BTreeMap::new();
            map.insert(",", &forth_print);
            map.insert("dup", &forth_dup);
            map.insert("over", &forth_over);
            map.insert("drop", &forth_drop);
            map.insert("rot", &forth_rot);
            map.insert("swap", &forth_swap);
            map.insert("debug", &forth_debug);
            map.insert("+", &forth_add);
            map.insert("-", &forth_sub);
            map.insert("/", &forth_div);
            map.insert("*", &forth_mul);
            map.insert("%", &forth_mod);
            map.insert(":", &forth_new_word);
            map
        };
        Self {
            formatter: UniversalVgaFormatter::new_unsafe(),
            instruction_counter: 0,
            instructions: ForthInstructions::default(),
            stack: Stack::default(),
            words: BTreeMap::default(),
            default_words,
        }
    }
}
impl ForthMachine {
    pub fn insert_default_word(&mut self, name: &'static str, f: ForthFunction) {
        self.default_words.insert(name, f);
    }
    pub fn add_instructions_to_end<S: AsRef<str>>(&mut self, data: &S) {
        self.instructions.add_instructions_to_end(&data.as_ref().chars().collect())
    }
    pub fn run(&mut self) {
        if self.instruction_counter >= self.instructions.len() {
            // Dont run because there are no instructions to run
            return;
        }
        let instruction_to_run = self.instructions.get(self.instruction_counter).unwrap();
        
        self.instruction_counter += 1;

        match instruction_to_run {
            ForthInstruction::Data(si) => {
                self.stack.push(si.clone());
            }
            ForthInstruction::Word(word) => {
                // Find word in default_words
                // Then in new words i guess
                if let Some(f) = self.default_words.get(word.as_str()) {
                    (*f)(self);
                } else if let Some(instructions) = self.words.get(word.as_str()) {
                    self.run_instructions_locally(instructions.clone());
                }

            }
        }
        
        
    }

    pub fn run_to_end(&mut self) {
        while self.instruction_counter < self.instructions.len() {
            self.run();
        }
    }

    fn run_instructions_locally(&mut self, fi: ForthInstructions) {
        for fi in fi.iter() {
            match fi {
                ForthInstruction::Data(si) => {
                    self.stack.push(si.clone());
                }
                ForthInstruction::Word(word) => {
                    // Find word in default_words
                    // Then in new words i guess
                    if let Some(f) = self.default_words.get(word.as_str()) {
                        (*f)(self);
                    } else if let Some(instructions) = self.words.get(word.as_str()) {
                        self.run_instructions_locally((*instructions).clone());
                    }
                }
            }
        }
    }
}
