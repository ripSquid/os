use super::KaggApp;

pub struct Help(Language);

enum Language {
    Swedish,
    English,
}

impl KaggApp for Help {
    fn start(&mut self, args: &[&str]) -> Result<(), super::StartError> {
        match args.get(0) {
            Some(&"eng") => self.0 = Language::English,
            Some(&"swe") => self.0 = Language::Swedish,
            Some(_) |
            None => (),
        }
        Ok(())
    }
    fn update(&mut self, handle: &mut super::KaggHandle) {
        match &self.0 {
            Language::Swedish => todo!(),
            Language::English => todo!(),
        }
        handle.call_exit();
    }
}