use core::ops::Range;
use alloc::vec;
use alloc::{string::String, boxed::Box, vec::{Vec}};
use base::display::{VgaColorCombo, VgaColor};
use base::input::{ScanCode, CTRL_MODIFIER, KeyEvent, KeyModifier};
use base::{LittleManApp, forth::ForthMachine, ProgramError, display::DefaultVgaWriter, input::KEYBOARD_QUEUE};
use fs::{PathString, AppConstructor, DefaultInstall};




impl DefaultInstall for ForEditorFile {
    fn path() -> PathString {
        PathString::from("texteditor.run")
    }
}

#[derive(Debug, Default)]
pub struct ForEditorFile;

impl AppConstructor for ForEditorFile {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(ForEditor::default())
    }
}


struct ForEditor {
    work: String,
    line_cache: Vec<Range<usize>>,
    cursor_position: usize,
    x_offset: usize,
    y_offset: usize,
    path: Option<PathString>,
    state: EditorState
}
impl Default for ForEditor {
    fn default() -> Self {
        Self { work: Default::default(), cursor_position: Default::default(), x_offset: Default::default(), y_offset: Default::default(), path: None, line_cache: vec![0..0], state: EditorState::Writing }
    }
}
#[derive(Clone)]
enum EditorState {
    Writing,
    HoveringSave,
    HoveringExit,
    WritingSavePath(String),
}

impl ForEditor {
    fn refresh_line_cache(&mut self) {
        let line_indices = {
            let iter = self.work.match_indices('\n');
            let mut finale = Vec::new();
            let mut previous = 0;
            for (index, _) in iter {
                finale.push(previous..index);
                previous = index+1;
            }
            finale.push(previous..self.work.len());
            finale
        };
        self.line_cache = line_indices;
    }
    fn remove_char(&mut self) {
        if self.work.len() > 0 {
            let char = if self.cursor_position == self.work.len() {
                self.work.pop().unwrap()
            } else {
                let offset = {
                    let mut result = 0;
                    for i in 1..=4 {
                        if self.work.is_char_boundary(self.cursor_position-i) {
                            result = i;
                            break;
                        }
                    }
                    result
                };
                self.work.remove(self.cursor_position-offset)
                
            };
            self.cursor_position -= char.len_utf8();
        }
    }
    fn add_char(&mut self, char: char) {
        self.work.insert(self.cursor_position, char);
        self.cursor_position += char.len_utf8();
    } 
    fn save_file(&mut self, message: &mut (VgaColorCombo, &str)) {
        match &self.path {
            Some(exists) => {
                match fs::get_file_write(exists) {
                    Ok(mut data_file) => {
                        match data_file.write_file(self.work.as_bytes()) {
                            Ok(_) => *message = (VgaColorCombo::new(VgaColor::White, VgaColor::Green), "File Saved."),
                            Err(_) => *message = (VgaColorCombo::new(VgaColor::White, VgaColor::Red), "File system error."),
                        }
                    },
                    Err(_) => *message = (VgaColorCombo::new(VgaColor::White, VgaColor::Red), "File not found."),
                }
            },
            None => {self.state = EditorState::WritingSavePath(String::new()); *message = (VgaColorCombo::new(VgaColor::White, VgaColor::Black), "Oops...")},
        }
    }
    pub fn redraw(&mut self, new_event: Option<KeyEvent>, formatter: &mut DefaultVgaWriter) -> bool {
        let mut message = (VgaColorCombo::new(VgaColor::Black, VgaColor::White), "");
        let mut cursor_pos = None;
        let temp_line_range = self.y_offset..self.y_offset+25;

        if let Some(KeyEvent::KeyPressed { modifiers, key: new_key }) = new_event {
            
            match &mut self.state {
                EditorState::Writing => {
                    let first_part = new_key.0;
                    match (first_part, modifiers) {
                        (0xE048, _mods) => {
                            let Some((index, range)) = self.line_cache.iter().enumerate().min_by_key(|(_, e)| (e.start < self.cursor_position)) else {return false};
                            if index > 0 {
                                let offset = self.cursor_position - range.start;
                                let other_range =  &self.line_cache[index-1];
                                let new_offset = other_range.start + offset;
                                self.cursor_position = new_offset.min(other_range.end);
                            }
                            
                        }
                        (0xE04B, _mods) => {
                            self.cursor_position = self.cursor_position.saturating_sub(1);

                        }
                        (0xE050, _mods) => {
                            let Some((index, range)) = self.line_cache.iter().enumerate().min_by_key(|(_, e)| (e.start < self.cursor_position)) else {return false};
                            if index < self.line_cache.len()-1 {
                                let offset = self.cursor_position - range.start;
                                let other_range =  &self.line_cache[index+1];
                                let new_offset = other_range.start + offset;
                                self.cursor_position = new_offset.min(other_range.end);
                            }
                        },
                        (0xE04D, _mods) => {
                            self.cursor_position = self.cursor_position.saturating_add(1);
                            while self.cursor_position < self.work.len() && !self.work.is_char_boundary(self.cursor_position) {
                                self.cursor_position = self.cursor_position.saturating_add(1);
                            }
                        },
                        (0x1F, (KeyModifier::CTRL | KeyModifier::SHIFT)) => {
                            self.save_file(&mut message);
                        },
                        _ => {
                            let Some(new_char) = new_key.resolve_text_char(modifiers) else {return false};
                            if new_char != '\x08' {
                                self.add_char(new_char);
                            } else {
                                self.remove_char();
                            }
                            self.refresh_line_cache();
                        }
                    }
                    while self.cursor_position > 0 && !self.work.is_char_boundary(self.cursor_position) {
                        self.cursor_position = self.cursor_position.saturating_sub(1);
                    }
                },
                EditorState::HoveringSave => {
                    
                },
                EditorState::HoveringExit => {
                    
                },
                EditorState::WritingSavePath(string) => {
                    match new_key.resolve_text_char(modifiers) {
                        Some('\x08') => {
                            string.pop();
                        }
                        Some('\n') => {
                            let mut new_state = EditorState::Writing;
                            core::mem::swap(&mut new_state, &mut self.state);
                            let EditorState::WritingSavePath(path) = new_state else {unreachable!()};
                            if fs::create_data_file(path.clone(), [].as_slice()).is_ok() {
                                self.path = Some(PathString::from(path));
                                self.save_file(&mut message);
                            } else {
                                message = (VgaColorCombo::new(VgaColor::White, VgaColor::Red), "Failed to create file.");
                            }
                           
                        }
                        Some(char) => {
                            string.push(char);
                            
                        },
                        None => (),
                    }
                },
            }
        }
       
        let line_range = self.y_offset..self.y_offset+25;
        
        formatter.clear_screen(VgaColor::Blue).set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Blue)).set_position((0,0)).disable_cursor();
        for (i, index) in line_range.into_iter().enumerate() {
            if let Some(line) = self.line_cache.get(index) {
                if line.contains(&self.cursor_position) {
                    cursor_pos = Some((self.cursor_position-line.start, i));
                } else if self.cursor_position == line.end {
                    cursor_pos = Some((line.end-line.start, i));
                }
                let str = &self.work[line.clone()];
                let len = str.len();
                //unless len is bigger than the x_offset we're not drawing anything anyway
                if !(len > self.x_offset) {
                    continue;
                }
                let substr = &str[self.x_offset..len.min(self.x_offset+80)];
                formatter.set_position((0,i)).write_str(substr);  
            }
        }
        formatter.set_position((0,24)).set_default_colors(message.0).write_str(message.1);
        if let Some((x, y)) = cursor_pos {
            formatter.update_cursor(x as u8, y as u8).enable_cursor();
        }
        if let EditorState::WritingSavePath(string) = &self.state {
            formatter.set_position((0, 24)).set_default_colors(VgaColorCombo::on_black(VgaColor::White)).enable_cursor().write_str(&string);
        }

        //if new char is escape immediately quit
        new_event.map(|v| v == KeyEvent::KeyPressed { modifiers: KeyModifier::default(), key: ScanCode::new(1) }) == Some(true)
    }
    fn load_file(&mut self, path: String) -> Result<(), ProgramError> {
        let file = fs::get_file(&path).map_err(|_| ProgramError::FileSystemError)?.read_file().map_err(|_| ProgramError::Custom("could not read file!"))?; 
        self.work = String::from_utf8(file).map_err(|_| ProgramError::Custom("Invalid file contents"))?;
        self.path = Some(PathString::from(path));
        Ok(())
    }
}
impl LittleManApp for ForEditor {
    fn run(&mut self, machine: &mut ForthMachine) -> Result<(), ProgramError> {
        if let Some(path) = machine.stack.try_pop::<String>() {
            self.load_file(path);
        }
        let formatter = machine.formatter.switch_to_text_mode();
        self.refresh_line_cache();
        self.redraw(None, formatter);
        loop {
            let key = unsafe { KEYBOARD_QUEUE.get_blocking() };
            if self.redraw(Some(key), formatter) {
                break;
            }
        }
        formatter.clear_screen(VgaColor::Black).set_default_colors(VgaColorCombo::on_black(VgaColor::White)).enable_cursor().set_position((0,0));

        Ok(())
    }
}