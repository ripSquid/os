use core::fmt::Display;
use core::ops::Range;
use alloc::{vec, format};
use alloc::{string::String, boxed::Box, vec::{Vec}};
use base::display::{VgaColorCombo, VgaColor};
use base::input::{ScanCode, CTRL_MODIFIER, KeyEvent, Modifiers};
use base::{LittleManApp, forth::ForthMachine, ProgramError, display::DefaultVgaWriter, input::KEYBOARD_QUEUE};
use fs::{PathString, AppConstructor, DefaultInstall};


type EditorMessage = Option<(VgaColorCombo, &'static str)>;

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
    state: EditorState,
    message: Option<(VgaColorCombo, &'static str, usize)>
}
impl Default for ForEditor {
    fn default() -> Self {
        Self {message: None, work: Default::default(), cursor_position: Default::default(), x_offset: Default::default(), y_offset: Default::default(), path: None, line_cache: vec![0..0], state: EditorState::Writing }
    }
}
#[derive(Clone)]
enum EditorState {
    Writing,
    InMenu(MenuOptions),
    WritingSavePath(String),
}
#[derive(Clone, PartialEq)]
enum MenuOptions {
    Save,
    SaveAs,
    Open,
    Quit,
}
impl Display for MenuOptions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MenuOptions::Save => "Save".fmt(f),
            MenuOptions::SaveAs => "Save As".fmt(f),
            MenuOptions::Open => "Open".fmt(f),
            MenuOptions::Quit => "Quit".fmt(f),
        }
    }
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
    fn save_file(&mut self, message: &mut Option<(VgaColorCombo, &str)>) {
        match &self.path {
            Some(exists) => {
                match fs::get_file_write(exists) {
                    Ok(mut data_file) => {
                        match data_file.write_file(self.work.as_bytes()) {
                            Ok(_) => *message = Some((VgaColorCombo::new(VgaColor::Black, VgaColor::Green), "File Saved.")),
                            Err(_) => *message = Some((VgaColorCombo::new(VgaColor::White, VgaColor::Red), "File system error.")),
                        }
                    },
                    Err(_) => *message = Some((VgaColorCombo::new(VgaColor::White, VgaColor::Red), "File not found.")),
                }
            },
            None => {self.state = EditorState::WritingSavePath(String::new());},
        }
    }
    fn draw_work(&mut self, formatter: &mut DefaultVgaWriter) {
        let mut cursor_pos = None;
        formatter.set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Blue)).disable_cursor();
        let line_range = self.y_offset..self.y_offset+24;
        for (i, index) in line_range.into_iter().enumerate() {
            let substr = if let Some(line) = self.line_cache.get(index) {
                if line.contains(&self.cursor_position) {
                    cursor_pos = Some((self.cursor_position-line.start, i));
                } else if self.cursor_position == line.end {
                    cursor_pos = Some((line.end-line.start, i));
                }
                let str = &self.work[line.clone()];
                let len = str.len();
                //unless len is bigger than the x_offset we're not drawing anything anyway
                &str[self.x_offset..len.min(self.x_offset+80)]
            } else {
                ""
            };
            formatter.set_position((0,i)).write_str(&format!("{substr:<80}"));  
        }
        if let Some((x, y)) = cursor_pos {
            formatter.update_cursor(x as u8, y as u8).enable_cursor();
        }
    } 
    fn draw_dialog(formatter: &mut DefaultVgaWriter, title: &str, text: &str) {
        let width = 40;
        let start = (80-width)/2;
        formatter.set_position((start, 10)).set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Red)).write_str(format!("O{:=^1$}O", title, width-2));
        for i in 11..14 {
            formatter.set_position((start, i)).set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Red)).write_str(format!("|{:1$}|", "",width-2));
        }
        formatter.set_position((start, 14)).set_default_colors(VgaColorCombo::new(VgaColor::White, VgaColor::Red)).write_str(format!("O{:=^1$}O", "", width-2));
        formatter.set_position((start+2, 12)).set_default_colors(VgaColorCombo::on_black(VgaColor::White)).enable_cursor().write_str(text);
    }
    fn handle_writing_key(&mut self, modifiers: Modifiers, key: ScanCode, message: &mut EditorMessage) {
        let first_part = key.0;
        match (first_part, modifiers) {
            (0xE048, _mods) => {
                let Some((index, range)) = self.line_cache.iter().enumerate().find(|(_, r)| r.contains(&self.cursor_position) || self.cursor_position == r.end) else {return};
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
                let Some((index, range)) = self.line_cache.iter().enumerate().find(|(_, r)| r.contains(&self.cursor_position) || self.cursor_position == r.end) else {return};
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
            (0x1F, Modifiers::CTRL) => {
                self.save_file(message);
            },
            _ => {
                let Some(new_char) = key.resolve_text_char(modifiers) else {return};
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
    }
    fn redraw(&mut self, new_event: Option<KeyEvent>, formatter: &mut DefaultVgaWriter) -> bool {
        let mut message = None;
        let mut shutdown = false;
        const CTRLNALT: Modifiers = Modifiers::CTRL.combine(Modifiers::ALT);

        match new_event {
            Some(KeyEvent::ModifiersChanged { modifiers: CTRLNALT }) => {
                self.state = EditorState::InMenu(MenuOptions::Save);
            }
            Some(KeyEvent::KeyPressed { modifiers, key }) => match &mut self.state {
                EditorState::Writing => {
                    self.handle_writing_key(modifiers, key, &mut message);
                },
                EditorState::InMenu(option) => {
                    let first_part = key.0;
                    match first_part {
                        0x01 => {
                            self.state = EditorState::Writing;
                        },
                        0x1C => {
                            match option {
                                MenuOptions::Save => {self.save_file(&mut message);},
                                MenuOptions::SaveAs => (),
                                MenuOptions::Open => (),
                                MenuOptions::Quit => shutdown = true,
                            }
                        }
                        0xE048 => {
                            *option = match option {
                                MenuOptions::Save => MenuOptions::SaveAs,
                                MenuOptions::SaveAs => MenuOptions::Open,
                                MenuOptions::Open => MenuOptions::Quit,
                                MenuOptions::Quit => MenuOptions::Save,
                            }
                        }
                        0xE050 => {
                            *option = match option {
                                MenuOptions::Save => MenuOptions::Quit,
                                MenuOptions::SaveAs => MenuOptions::Save,
                                MenuOptions::Open => MenuOptions::SaveAs,
                                MenuOptions::Quit => MenuOptions::Open,
                            }
                        }
                        _ => {}
                    }
                },
                EditorState::WritingSavePath(string) => {
                    match key.resolve_text_char(modifiers) {
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
                                message = Some((VgaColorCombo::new(VgaColor::White, VgaColor::Red), "Failed to create file."));
                            }
                           
                        }
                        Some(char) => {
                            string.push(char);
                            
                        },
                        None => (),
                    }
                },
            },
            _ => (),
        }
       
        if let Some((c, t)) = message {
            self.message = Some((c, t, 10))
        }
        self.draw_bottom(formatter);

        match &self.state {
            EditorState::Writing => {
                self.draw_work(formatter);
            },
            EditorState::InMenu(option) => {
                Self::draw_menu(formatter, option);
            },
            EditorState::WritingSavePath(string) => {
                Self::draw_dialog(formatter, "| Enter Save Path: |", &string)
            },
        }

        shutdown
    }
    fn draw_menu(formatter: &mut DefaultVgaWriter, option: &MenuOptions) {
        formatter.disable_cursor();
        for (i, kind) in [MenuOptions::Open, MenuOptions::Quit, MenuOptions::Save, MenuOptions::SaveAs].iter().enumerate() {
            formatter.set_position((2, 23-i));
            let (combo, format) = match kind == option {
                true => (VgaColorCombo::new(VgaColor::Black, VgaColor::Yellow), format!("[{kind:<14}]")),
                false => (VgaColorCombo::new(VgaColor::Black, VgaColor::White), format!(" {kind:<14} ")),
            };
            formatter.set_default_colors(combo).write_str(format);
        }
    }
    fn draw_start_frame(&mut self, formatter: &mut DefaultVgaWriter) {
        self.draw_work(formatter);
        self.draw_bottom(formatter);
    }
    fn draw_bottom(&mut self, formatter: &mut DefaultVgaWriter) {
        let (combo, text) = match &mut self.message {
            Some((_,_,0)) => {
                self.message = None;
                None
            }
            Some((c,t,n)) => {
                *n = n.saturating_sub(1);
                Some((*c,*t))
            },
            None => None,
        }.unwrap_or((VgaColorCombo::new(VgaColor::Black, VgaColor::White), "Press Ctrl+Alt to bring up menu"));
        formatter.set_position((0,24)).set_default_colors(combo).write_str(format!("{text:<80}"));
        
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
        self.draw_start_frame(formatter);
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
