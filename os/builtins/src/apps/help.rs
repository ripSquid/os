use alloc::boxed::Box;
use base::{
    forth::{ForthMachine, Stack, StackItem},
    LittleManApp, OsHandle, ProgramError,
};
use fs::{AppConstructor, DefaultInstall, Path};

#[derive(Default)]
pub struct Help;
enum Language {
    Swedish,
    English,
}
impl AppConstructor for Help {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(Help)
    }
}
impl DefaultInstall for Help {
    fn path() -> Path {
        Path::from("help.run")
    }
}
impl LittleManApp for Help {
    fn run(&mut self, args: &mut ForthMachine) -> Result<(), ProgramError> {
        let mut language = Language::Swedish;
        match args.stack.pop() {
            Some(StackItem::String(string)) => match string.as_str() {
                "eng" => language = Language::English,
                "swe" => language = Language::Swedish,
                _ => args.stack.push(StackItem::String(string)),
            },
            Some(item) => {
                args.stack.push(item);
            }
            _ => (),
        }
        let text = match language {
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
        args.formatter.next_line().write_str(text);
        Ok(())
    }
}
