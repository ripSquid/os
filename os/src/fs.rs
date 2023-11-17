use core::error::Error;

use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};
use hashbrown::HashMap;

use crate::apps::{LittleManApp, InstallableApp};

use spin::{RwLock, RwLockReadGuard, RwLockUpgradableGuard};
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
}

#[derive(Debug)]
pub enum FileSystemError {
    FileSystemNotInitialized,
    IncorrectFileType,
    FileNotFound,
    DirectoryNotFound,
    InvalidParentDirectory,
    Busy,
    EmptyPath,
    CritikalUnknown,
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
                    Ok(DirRead(dir.iter().map(|(k,_)| Path::from(k.as_str())).collect()))
                } else {
                    Err(FileSystemError::IncorrectFileType)
                }
            },
            None => {
                Ok(DirRead(self.filesystem.as_ref().ok_or(FileSystemError::CritikalUnknown)?.as_ref().ok_or(FileSystemError::FileSystemNotInitialized)?.iter().map(|(k,_)| Path::from(k.as_str())).collect()))
            },
        }
    }
    pub fn launch_app(&self) -> Result<Box<dyn LittleManApp>, FileSystemError> {
        match self.read_guards.last() {
            Some(handle) => {
                if let KaggFile::App(constructor) = &**handle {
                    Ok(constructor.instantiate())
                } else {
                    Err(FileSystemError::IncorrectFileType)
                }
            },
            None => {
                Err(FileSystemError::CritikalUnknown)
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
                None => return Err(FileSystemError::CritikalUnknown),
            };
            final_result
        }
    }
    fn add(mut self, component: &str) -> Result<Self, FileSystemError> {
        if component == "" {
            return Ok(self)
        }
        if self.read_guards.len() == 0 {
            self.path += component;

            let guard = if let Some(other_guard) = self.read_guards.last() {
                match &**other_guard {
                    KaggFile::Directory(dir) => {
                        dir.get(component).ok_or(FileSystemError::FileNotFound)?
                    }
                    _ => return Err(FileSystemError::FileNotFound),
                }
            } else {
                self.filesystem.as_ref().ok_or(FileSystemError::CritikalUnknown)?
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
        }
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
pub fn create_data_file<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    create_file(path, KaggFile::Data(Vec::new()))
}
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<LittleFileHandle<'static>, FileSystemError> {
    create_file(path, KaggFile::Directory(HashMap::new()))
}
pub fn install_app<A: InstallableApp>() -> Result<LittleFileHandle<'static>, FileSystemError> {
    let (path, app) = A::install();
    create_file(path, KaggFile::App(app))
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<DirRead, FileSystemError> {
    match get_file(path) {
        Ok(file_handle) => {
            file_handle.children()
        },
        Err(err) => {
            //Err(FileSystemError::Busy)
            Ok(DirRead(FILE_SYSTEM.0.read().as_ref().ok_or(FileSystemError::FileSystemNotInitialized)?.iter().map(|(k,_)| Path::from(k.as_str())).collect()))
        },
    }
}
pub struct DirRead(Vec<Path>);
impl DirRead {
    pub fn items(self) -> impl Iterator<Item = Path> {
        self.0.into_iter()
    }
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
    unsafe {ACTIVE_DIRECTORY = Some(p)}; 
}