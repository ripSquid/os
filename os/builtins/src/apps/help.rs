use alloc::boxed::Box;
use base::{
    forth::{Stack, StackItem},
    LittleManApp, OsHandle, StartError,
};
use fs::{AppConstructor, DefaultInstall, Path};

pub struct HelpApp(Language);
#[derive(Default)]
pub struct Help;
enum Language {
    Swedish,
    English,
}
impl AppConstructor for Help {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(HelpApp(Language::Swedish))
    }
}
impl DefaultInstall for Help {
    fn path() -> Path {
        Path::from("help.run")
    }
}
impl LittleManApp for HelpApp {
    fn start(&mut self, args: &mut Stack) -> Result<(), StartError> {
        match args.pop() {
            Some(StackItem::String(string)) => match string.as_str() {
                "eng" => self.0 = Language::English,
                "swe" => self.0 = Language::Swedish,
                _ => args.push(StackItem::String(string)),
            },
            Some(item) => {
                args.push(item);
            }
            _ => (),
        }
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle) {
        let text = match &self.0 {
            Language::Swedish => {
                "Välkommen till ett gymnasiearbete gjort av två elever på Lars Kagg Skolan.
            Detta operativsystem kommer med olika demon 
            och verktyg som visar vad det är kapabelt av.
            För att lista alla program och filer, skriv [\"dir\" run]
            
            For english help, write [\"eng\" \"help\" run]"
            }
            Language::English => {
                "Welcome to a Thesis work created by two students at the school of Lars Kagg.
            This operating system comes with demos and 
            tools which displays its full capabilities.
            To list all programs and files, write [\"dir\" run]"
            }
        };
        if let Ok(formatter) = handle.text_mode_formatter() {
            formatter.next_line().write_str(text);
        }
        handle.call_exit();
    }
}
