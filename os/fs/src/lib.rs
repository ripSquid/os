#![no_std]
#![feature(error_in_core)]
#![feature(result_flattening)]

extern crate alloc;
pub mod apps;
mod directory;
mod file;
mod path;
pub use apps::{DefaultInstall, InstallableApp};
use base::debug;
pub use directory::*;
pub use file::*;
use handle::{LittleFileHandle, ReadPriviliges, WritePriviliges};
pub use path::*;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

pub mod handle;

use spin::RwLock;
pub struct RamFileSystem(RwLock<Option<RwLock<Directory>>>);

#[derive(Debug)]
pub enum FileSystemError {
    FileSystemNotInitialized,
    IncorrectFileType(&'static str),
    FileNotFound,
    DirectoryNotFound,
    InvalidParentDirectory,
    Busy,
    EmptyPath,
    CUPE,
    NameAlreadyExists,
}

static FILE_SYSTEM: RamFileSystem = RamFileSystem(RwLock::new(None));
static mut ACTIVE_DIRECTORY: Option<Path> = None;

impl RamFileSystem {
    fn init(&self) {
        *self.0.write() = Some(RwLock::new(Directory::default()));
    }

    fn get_file<'b, P: AsRef<Path>>(
        &'b self,
        path: P,
    ) -> Result<LittleFileHandle<'b, ReadPriviliges>, FileSystemError> {
        LittleFileHandle::<ReadPriviliges>::new(path.as_ref().clone().clean())
    }
    fn get_file_write<'b, P: AsRef<Path>>(
        &'b self,
        path: P,
    ) -> Result<LittleFileHandle<'b, WritePriviliges>, FileSystemError> {
        LittleFileHandle::<WritePriviliges>::new(path.as_ref().clone().clean())
    }
}
pub struct File {
    data: KaggFile,
    name: String,
}

pub fn start() {
    FILE_SYSTEM.init()
}

pub fn get_file<P: AsRef<Path>>(
    path: P,
) -> Result<LittleFileHandle<'static, ReadPriviliges>, FileSystemError> {
    FILE_SYSTEM.get_file(path)
}
pub fn get_file_relative<P: AsRef<Path>>(
    path: P,
) -> Result<LittleFileHandle<'static, ReadPriviliges>, FileSystemError> {
    FILE_SYSTEM.get_file(active_directory().append(path.as_ref()))
}
pub fn create_data_file<P: AsRef<Path>>(
    path: P,
) -> Result<LittleFileHandle<'static, WritePriviliges>, FileSystemError> {
    create_file(path, KaggFile::Data(Vec::new()))
}
pub fn create_dir<P: AsRef<Path>>(
    path: P,
) -> Result<LittleFileHandle<'static, WritePriviliges>, FileSystemError> {
    create_file(path, KaggFile::Directory(Directory::default()))
}
pub fn install_app<A: InstallableApp>() -> Result<(), FileSystemError> {
    let (path, app) = A::install();
    let path = active_directory().append(&path);
    create_file(path, KaggFile::App(app))?;
    Ok(())
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<DirRead, FileSystemError> {
    match get_file(path) {
        Ok(file_handle) => file_handle.read_dir(),
        Err(err) => Err(err),
    }
}

pub struct FileMetadata {
    pub path: Path,
    pub filetype: FileType,
}
pub enum FileType {
    Directory,
    Data,
    App,
}

impl File {
    pub fn empty<S: ToString>(name: S) -> Self {
        Self {
            data: KaggFile::Data(Vec::new()),
            name: name.to_string(),
        }
    }
    pub fn from_app<S: ToString>(app: Box<dyn AppConstructor>, name: S) -> Self {
        Self {
            data: KaggFile::App(app),
            name: name.to_string(),
        }
    }
}
pub fn active_directory() -> Path {
    unsafe {
        ACTIVE_DIRECTORY
            .as_ref()
            .unwrap_or(&Path(String::from("")))
            .clone()
    }
}
pub fn set_active_directory(p: Path) {
    unsafe { ACTIVE_DIRECTORY = Some(p.clean()) };
}

pub(crate) fn create_file<P: AsRef<Path>>(
    path: P,
    file: KaggFile,
) -> Result<LittleFileHandle<'static, WritePriviliges>, FileSystemError> {
    let mut parent = path.as_ref().clone();
    let file_name = parent.pop().unwrap();
    debug!(&(parent.0).as_str());
    match FILE_SYSTEM.get_file_write(parent) {
        Ok(mut exists) => {
            exists.add_child(File {
                data: file,
                name: file_name.0,
            })?;
        }
        Err(_) => return Err(FileSystemError::Busy),
    }

    FILE_SYSTEM.get_file_write(path)
}
