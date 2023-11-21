use core::ops::Range;
use alloc::vec;
use alloc::{string::String, boxed::Box, vec::{Vec}};
use base::display::{VgaColorCombo, VgaColor};
use base::input::Key;
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
    path: Result<PathString, String>,
    state: EditorState
}
impl Default for ForEditor {
    fn default() -> Self {
        Self { work: Default::default(), cursor_position: Default::default(), x_offset: Default::default(), y_offset: Default::default(), path: Err(String::default()), line_cache: vec![0..0], state: EditorState::Writing }
    }
}
#[derive(Clone, Copy)]
enum EditorState {
    Writing,
    HoveringSave,
    HoveringExit,
    WritingSavePath,
}

impl ForEditor {
    pub fn redraw(&mut self, new_char: Option<Key>, formatter: &mut DefaultVgaWriter) -> bool {
        if let Some(new_char) = new_char {
            
            match self.state {
                EditorState::Writing => {
                    let Some(new_char) = Into::<Option<char>>::into(new_char) else {return false};
                    if new_char != '\x08' {
                        self.work.insert(self.cursor_position, new_char);
                        self.cursor_position += new_char.len_utf8();
                    } else {
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
                },
                EditorState::HoveringSave => {
                    
                },
                EditorState::HoveringExit => {
                    
                },
                EditorState::WritingSavePath => {
                    
                },
            }
        }
       
        let line_range = self.y_offset..self.y_offset+25;
        
        formatter.clear_screen(VgaColor::Blue).set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Blue)).set_position((0,0)).disable_cursor();
        for (i, index) in line_range.into_iter().enumerate() {
            if let Some(line) = self.line_cache.get(index) {
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
       
        

        //if new char is escape immediately quit
        new_char.map(|v| Into::<Option<char>>::into(v)).flatten() == Some('\x1B')
    }
    fn load_file(&mut self, path: String) -> Result<(), ProgramError> {
        let file = fs::get_file(&path).map_err(|_| ProgramError::FileSystemError)?.read_file().map_err(|_| ProgramError::Custom("could not read file!"))?; 
        self.work = String::from_utf8(file).map_err(|_| ProgramError::Custom("Invalid file contents"))?;
        Ok(())
    }
}
impl LittleManApp for ForEditor {
    fn run(&mut self, machine: &mut ForthMachine) -> Result<(), ProgramError> {
        if let Some(path) = machine.stack.try_pop::<String>() {
            self.load_file(path);
        }
        let formatter = machine.formatter.switch_to_text_mode();
        self.redraw(None, formatter);
        loop {
            let char = unsafe { KEYBOARD_QUEUE.getch_blocking() };
            if self.redraw(Some(char), formatter) {
                break;
            }
        }
        formatter.clear_screen(VgaColor::Black).set_default_colors(VgaColorCombo::on_black(VgaColor::White)).enable_cursor().set_position((0,0));

        Ok(())
    }
}