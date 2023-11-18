#![no_std]
#![feature(error_in_core)]

extern crate alloc;
pub mod apps;
pub use apps::{DefaultInstall, InstallableApp};
use base::LittleManApp;
use core::error::Error;

use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};
use hashbrown::HashMap;



use spin::{RwLock, RwLockUpgradableGuard};
pub struct RamFileSystem(RwLock<Option<HashMap<String, RwLock<KaggFile>>>>);

pub trait AppConstructor: Send + Sync + 'static {
    fn instantiate(&self) -> Box<dyn LittleManApp>;
}

pub enum KaggFile {
    Directory(HashMap<String, RwLock<KaggFile>>),
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

#[derive(Debug)]
pub enum FileSystemError {
    FileSystemNotInitialized,
    IncorrectFileType(&'static str),
    FileNotFound,
    DirectoryNotFound,
    InvalidParentDirectory,
    Busy,
    EmptyPath,
    CriticalUnknown,
    NameAlreadyExists,
}

static FILE_SYSTEM: RamFileSystem = RamFileSystem(RwLock::new(None));
static mut ACTIVE_DIRECTORY: Option<Path> = None;

impl RamFileSystem {
    fn init(&self) {
        *self.0.write() = Some(HashMap::new());
    }

    fn get_file<'b, P: AsRef<Path>>(
        &'b self,
        path: P,
    ) -> Result<LittleFileHandle<'b>, FileSystemError> {
        let components = path.as_ref().components();
        let mut handle = LittleFileHandle::new(self.0.upgradeable_read());
        for component in components {
            handle = handle.add(component)?;
        }
        Ok(handle)
    }
}
pub struct File {
    data: KaggFile,
    name: String,
}
pub struct LittleFileHandle<'a> {
    filesystem: Option<RwLockUpgradableGuard<'a, Option<HashMap<String, RwLock<KaggFile>>>>>,
    read_guards: Vec<RwLockUpgradableGuard<'a, KaggFile>>,
    path: String,
}
impl<'a> LittleFileHandle<'a> {
    fn children(&self) -> Result<DirRead, FileSystemError> {
        match self.read_guards.last() {
            Some(handle) => {
                if let KaggFile::Directory(dir) = &**handle {
                    Ok(DirRead(dir.iter().filter_map(|(k,v)| Some(FileMetadata { path: Path::from(k.as_str()), filetype: v.read().file_type()? })).collect()))
                } else {
                    Err(FileSystemError::IncorrectFileType("Trying to open file as directory"))
                }
            },
            None => {
                Ok(DirRead(self.filesystem.as_ref().ok_or(FileSystemError::CriticalUnknown)?.as_ref().ok_or(FileSystemError::FileSystemNotInitialized)?.iter().filter_map(|(k,v)| Some(FileMetadata { path: Path::from(k.as_str()), filetype: v.read().file_type()? })).collect()))
            },
        }
    }
    pub fn launch_app(&self) -> Result<Box<dyn LittleManApp>, FileSystemError> {
        match self.read_guards.last() {
            Some(handle) => {
                match &**handle {
                    KaggFile::App(constructor) => Ok(constructor.instantiate()),
                    KaggFile::Directory(_) => Err(FileSystemError::IncorrectFileType("Trying to launch directory as an app")),
                    KaggFile::Data(_) => Err(FileSystemError::IncorrectFileType("Trying to data file as an app")),
                    KaggFile::Deleted => Err(FileSystemError::IncorrectFileType("Trying to launch deleted file as an app")),
                    
                }
            },
            None => {
                Err(FileSystemError::CriticalUnknown)
            },
        }
    }
    
    fn new_unchecked() -> Self {
        Self::new(FILE_SYSTEM.0.upgradeable_read())
    }
    fn new(
        filesystem: RwLockUpgradableGuard<'a, Option<HashMap<String, RwLock<KaggFile>>>>,
    ) -> Self {
        Self {
            filesystem: Some(filesystem),
            read_guards: Vec::new(),
            path: String::from("./"),
        }
    }
    pub fn is_directory(&self) -> bool {
        self.read_guards.is_empty() || match self.read_guards.last().map(|lock| &**lock) {
            Some(KaggFile::Directory(_)) => true,
            None => true,
            _ => false,
        } 
    }
    fn create_file(&mut self, file: File) -> Result<(), FileSystemError> {
        if let Some(guard) = self.read_guards.pop() {
            match guard.try_upgrade() {
                Ok(mut write_guard) => {
                    let result = if let KaggFile::Directory(dir) = &mut *write_guard {
                        let File { data, name } = file;
                        dir.insert(name, RwLock::new(data));
                        Ok(())
                    } else {
                        Err(FileSystemError::DirectoryNotFound)
                    };
                    self.read_guards.push(write_guard.downgrade_to_upgradeable());
                    result
                },
                Err(nope) => {
                    self.read_guards.push(nope); 
                    Err(FileSystemError::Busy)
                },
            }
        } else {
            let final_result;
            (self.filesystem, final_result) = match self.filesystem.take() {
                Some(filesystem) => {
                    let result = match filesystem.try_upgrade() {
                        Ok(mut success) => {
                            let File { data, name } = file;
                            let final_result = match success.as_mut().ok_or(FileSystemError::FileSystemNotInitialized) {
                                Ok(worked) => {worked.insert(name, RwLock::new(data)); Ok(())},
                                Err(err) => Err(err),
                            };
                            (Some(success.downgrade_to_upgradeable()), final_result)
                        },
                        Err(fail) => (Some(fail), Err(FileSystemError::Busy)),
                    };
                    result
                },
                None => return Err(FileSystemError::CriticalUnknown),
            };
            final_result
        }
    }
    fn path(&self) -> Path {
        Path::from(self.path.as_str()).clean()
    }
    fn file_type(&self) -> Option<FileType> {
        match self.read_guards.last().map(|s| &**s) {
            Some(KaggFile::App(_)) => Some(FileType::App),
            Some(KaggFile::Data(_)) => Some(FileType::Data),
            Some(KaggFile::Directory(_)) => Some(FileType::Directory),
            Some(KaggFile::Deleted) => None,
            None => Some(FileType::Directory),
        }
    }
    fn add(mut self, component: &str) -> Result<Self, FileSystemError> {
        match component {
            "" => {
                return Ok(self)
            },
            ".." => {
                self.read_guards.pop();
                return Ok(self)
            },
            _ => (),
        }
        self.path += component;

        let guard = if let Some(other_guard) = self.read_guards.last() {
            match &**other_guard {
                KaggFile::Directory(dir) => {
                    dir.get(component).ok_or(FileSystemError::FileNotFound)?
                }
                _ => return Err(FileSystemError::FileNotFound),
            }
        } else {
            self.filesystem.as_ref().ok_or(FileSystemError::CriticalUnknown)?
                .as_ref()
                .ok_or(FileSystemError::FileSystemNotInitialized)?
                .get(component)
                .ok_or(FileSystemError::FileNotFound)?
        };
        let unsafe_guard = unsafe {
            (guard as *const RwLock<KaggFile>)
                .as_ref()
                .unwrap()
                .upgradeable_read()
        };
        self.read_guards.push(unsafe_guard);

        Ok(self)
    }
}
#[derive(Clone)]
pub struct Path(String);
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Path {
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split("/")
    }
    pub fn new() -> Self {
        Self(String::new())
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn append<A: AppendsPath>(mut self, path: &A) -> Self {
        self.0.push_str("/");
        self.0.push_str(path.to_str());
        self
    }
    pub fn clean(mut self) -> Self {
        let mut new = Vec::new();
        let sections = self.0.split('/');
        for section in sections {
            match section {
                "" => continue,
                "." => continue,
                ".." => {new.pop(); continue}
                _ => (),
            }
            new.push(section);
        }
        let mut finale = String::new();
        for part in new.into_iter() {
            finale.push('/');
            finale.push_str(part);
        }
        Self(finale)
    }
    pub fn add_extension(mut self, extension: &str) -> Self {
        self.0.push('.');
        self.0.push_str(extension);
        self
    }
}
pub trait AppendsPath {
    fn to_str(&self) -> &str; 
}
impl<T: AsRef<str>> AppendsPath for T {
    fn to_str(&self) -> &str {
        self.as_ref()
    }
}
impl AppendsPath for Path {
    fn to_str(&self) -> &str {
        self.as_ref().as_str()
    }
}
impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
pub fn start() {
    FILE_SYSTEM.init()
}
fn create_file<P: AsRef<Path>>(path: P, file: KaggFile) -> Result<LittleFileHandle<'static>, FileSystemError> {
    let mut base_handle = LittleFileHandle::new_unchecked();
    let components: Vec<_> = path.as_ref().components().collect();
    for component in &components[0..components.len()-1] {
        base_handle = base_handle.add(component)?;
    }
    let file_name = components.last().ok_or(FileSystemError::EmptyPath)?;
    base_handle.create_file(File { data: file, name: file_name.to_string() })?;
    Ok(base_handle.add(file_name)?)
} 
pub fn get_file<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    FILE_SYSTEM.get_file(path)
}
pub fn get_file_relative<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    FILE_SYSTEM.get_file(active_directory().append(path.as_ref()))
}
pub fn create_data_file<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    create_file(path, KaggFile::Data(Vec::new()))
}
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    match get_file(path.as_ref()) {
        Ok(exists) => {
            match exists.is_directory() {
                true => Ok(exists),
                false => Err(FileSystemError::NameAlreadyExists),
            }
        },
        Err(_) => create_file(path, KaggFile::Directory(HashMap::new())),
    }
}
pub fn install_app<A: InstallableApp>() -> Result<LittleFileHandle<'static>, FileSystemError> {
    let (path, app) = A::install();
    let path = active_directory().append(&path);
    create_file(path, KaggFile::App(app))
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<DirRead, FileSystemError> {
    match get_file(path) {
        Ok(file_handle) => {
            file_handle.children()
        },
        Err(err) => {
            Err(err)
            //Ok(DirRead(FILE_SYSTEM.0.read().as_ref().ok_or(FileSystemError::FileSystemNotInitialized)?.iter().map(|(k,_)| Path::from(k.as_str())).collect()))
        },
    }
}
pub struct DirRead(Vec<FileMetadata>);
impl DirRead {
    pub fn items(self) -> impl Iterator<Item = FileMetadata> {
        self.0.into_iter()
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
        Self { data: KaggFile::Data(Vec::new()), name: name.to_string() }
    }
    pub fn from_app<S: ToString>(app: Box<dyn AppConstructor>, name: S) -> Self {
        Self { data: KaggFile::App(app), name: name.to_string() }
    }
}
pub fn active_directory() -> Path {
    unsafe {ACTIVE_DIRECTORY.as_ref().unwrap_or(&Path(String::from(""))).clone()}
}
pub fn set_active_directory(p: Path) {
    unsafe {ACTIVE_DIRECTORY = Some(p.clean())}; 
}

pub fn fs_ref() -> &'static RamFileSystem {
    &FILE_SYSTEM
}