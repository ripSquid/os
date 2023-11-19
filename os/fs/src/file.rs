use alloc::{boxed::Box, vec::Vec};
use base::LittleManApp;

use crate::{Directory, FileType};

pub trait AppConstructor: Send + Sync + 'static {
    fn instantiate(&self) -> Box<dyn LittleManApp>;
}
pub enum KaggFile {
    Directory(Directory),
    Data(Vec<u8>),
    App(Box<dyn AppConstructor>),
    Deleted,
}
impl KaggFile {
    pub fn is_directory(&self) -> bool {
        match self {
            KaggFile::Directory(_) => true,
            _ => false,
        }
    }
    pub fn file_type(&self) -> Option<FileType> {
        match self {
            KaggFile::Directory(_) => Some(FileType::Directory),
            KaggFile::Data(_) => Some(FileType::Data),
            KaggFile::App(_) => Some(FileType::App),
            KaggFile::Deleted => None,
        }
    }
}
