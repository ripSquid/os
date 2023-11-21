use alloc::{string::String, boxed::Box, vec::Vec};
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
    cursor_position: usize,
    x_offset: usize,
    y_offset: usize,
    path: Result<PathString, String>,
}
impl Default for ForEditor {
    fn default() -> Self {
        Self { work: Default::default(), cursor_position: Default::default(), x_offset: Default::default(), y_offset: Default::default(), path: Err(String::default()) }
    }
}

enum EditorState {
    Writing,
    HoveringSave,
    HoveringExit,
    WritingSavePath,
}

impl ForEditor {
    pub fn redraw(&mut self, new_char: char, formatter: &mut DefaultVgaWriter) -> bool {
        let line_indices = {
            let mut iter = self.work.match_indices('\n');
            let mut finale = Vec::new();
            let mut previous = None;
            for (index, _) in iter {
                if let Some(previous) = previous.replace(index) {
                    finale.push(previous..index);
                }
            } 
            if let Some(previous) = previous {
                finale.push(previous..self.work.chars().count());
            }
            if finale.is_empty() {
                finale.push(0..self.work.chars().count());
            }
            finale
        };
        let mut line_indices = self.work.split("\n");
        let line_range = self.y_offset..self.y_offset+25;
        let column_range = self.x_offset..;
        if self.y_offset > 0 {
            let _ = line_indices.nth(self.y_offset-1);
        }
        true
    }
}
impl LittleManApp for ForEditor {
    fn run(&mut self, machine: &mut ForthMachine) -> Result<(), ProgramError> {
        let path = machine.stack.try_pop::<String>().ok_or(ProgramError::InvalidStartParameter)?;
        let file = fs::get_file(&path).map_err(|_| ProgramError::FileSystemError)?.read_file().map_err(|_| ProgramError::Custom("could not read file!"))?; 
        self.work = String::from_utf8(file).map_err(|_| ProgramError::Custom("Invalid file contents"))?;

        let formatter = machine.formatter.switch_to_text_mode();
        loop {
            let char = unsafe { KEYBOARD_QUEUE.getch_blocking() };
            if self.redraw(char.into(), formatter) {
                break;
            }
        }
        formatter.clear_screen(base::display::VgaColor::Black).enable_cursor().set_position((0,2));

        Ok(())
    }
}