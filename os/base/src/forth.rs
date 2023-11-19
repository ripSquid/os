use core::fmt::Display;

use alloc::{collections::BTreeMap, format, string::String, vec::Vec};

use crate::display::{DefaultVgaWriter, UniversalVgaFormatter};

pub type ForthFunction = &'static (dyn Fn(&mut ForthMachine) + Sync + Send + 'static);

fn forth_print(fm: &mut ForthMachine) {
    if let Some(x) = fm.stack.pop() {
        fm.formatter.write_str(&format!("{}", x));
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
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

#[derive(Clone)]
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
            } else if i + 1 == new_data.len() {
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
        if self.len() > u {
            return Some(&self.0[u]);
        }
        None
    }

    fn iter(&self) -> core::slice::Iter<'_, ForthInstruction> {
        self.0.iter()
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
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
    pub fn run(&mut self) {
        if self.instruction_counter >= self.instructions.len() {
            // Dont run because there are no instructions to run
            return;
        }

        let instruction_to_run = self.instructions.get(self.instruction_counter).unwrap();
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

        self.instruction_counter += 1;
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
