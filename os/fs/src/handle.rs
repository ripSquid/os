use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};
use base::LittleManApp;

use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{DirRead, Directory, File, FileSystemError, KaggFile, Path, FILE_SYSTEM};

/// Definitions for the different FileHandle priviliges
pub enum WritePriviliges {}
pub enum ReadPriviliges {}
pub trait FileHandlePriviliges {}

impl FileHandlePriviliges for WritePriviliges {}
impl FileHandlePriviliges for ReadPriviliges {}

/// A file handle to a file on the operating system
///
/// This uses recursive locking in order to ensure safety in case of multiple handles coexisting
///
/// Depending on the handles priviliges you will be given access to appropriate methods
pub struct LittleFileHandle<'a, T: FileHandlePriviliges> {
    _filesystem: RwLockReadGuard<'a, Option<RwLock<Directory>>>,
    locks: FileHandleLocks<'a>,
    path: Path,
    _phantom: PhantomData<T>,
}

/// Methods which all file handles may use
impl<'a, T: FileHandlePriviliges> LittleFileHandle<'a, T> {
    /// The absolute path to this file
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_directory(&self) -> bool {
        self.locks.attempt_dir(|_| ()).is_ok()
    }

    /// Attempt to launch this file as an app
    pub fn launch_app(&self) -> Result<Box<dyn LittleManApp>, FileSystemError> {
        self.locks
            .attempt(|file| match file {
                KaggFile::App(app) => Ok(app.instantiate()),
                KaggFile::Data(_) => Err(FileSystemError::IncorrectFileType(
                    "trying to open data file",
                )),
                KaggFile::Directory(_) => Err(FileSystemError::IncorrectFileType(
                    "trying to open directory",
                )),
                KaggFile::Deleted => Err(FileSystemError::IncorrectFileType(
                    "trying to open deleted file",
                )),
            })
            .map_err(|_err| FileSystemError::Busy)
            .flatten()
    }

    /// Attempt to read this file as a directory
    pub fn read_dir(&self) -> Result<DirRead, FileSystemError> {
        self.locks
            .attempt_dir(|dir| dir.read_all())
            .map_err(|_err| FileSystemError::IncorrectFileType("file is not a directory"))
    }

    /// Attempt to read this file as a series of bytes
    pub fn read_file(&self) -> Result<Vec<u8>, FileSystemError> {
        self.locks
            .attempt(|file| match file {
                KaggFile::Data(data) => Ok(data.to_vec()),
                KaggFile::App(_) => Err(FileSystemError::IncorrectFileType("trying to app file")),
                KaggFile::Directory(_) => Err(FileSystemError::IncorrectFileType(
                    "trying to open directory",
                )),
                KaggFile::Deleted => Err(FileSystemError::IncorrectFileType(
                    "trying to open deleted file",
                )),
            })
            .map_err(|_err| FileSystemError::Busy)
            .flatten()
    }
}

/// Methods which only file handles with writing privliges may want to use
impl<'a> LittleFileHandle<'a, WritePriviliges> {
    pub fn new(path: Path) -> Result<Self, FileSystemError> {
        let path = path.clean();
        let filesystem = FILE_SYSTEM.0.read();
        let locks = if path.components().count() == 1 {
            unsafe {
                FileHandleLocks::write_root(
                    filesystem.as_ref().ok_or(FileSystemError::PointerError)? as *const _,
                )?
            }
        } else {
            unsafe {
                FileHandleLocks::write(
                    filesystem.as_ref().ok_or(FileSystemError::PointerError)? as *const _,
                    &path,
                )?
            }
        };
        Ok(Self {
            _filesystem: filesystem,
            locks,
            path,
            _phantom: PhantomData,
        })
    }

    /// Attempt to write to this file
    ///
    /// This may be unsuccessful if the file isn't a data file or is unavailable in some other way
    pub fn write_file(&mut self, insert: &[u8]) -> Result<(), FileSystemError> {
        self.locks
            .attempt_mut(|file| match file {
                KaggFile::Data(data) => {
                    *data = Vec::with_capacity(insert.len());
                    data.extend_from_slice(insert);
                    Ok(())
                }
                _ => Err(()),
            })
            .map_err(|_err| ())
            .flatten()
            .map_err(|_err| FileSystemError::PointerError)
    }

    /// Add a new file inside this one if it's a directory
    pub fn add_child(&mut self, file: File) -> Result<(), FileSystemError> {
        self.locks
            .attempt_dir_mut(|dir| dir.add_file(file))
            .map_err(|_err| FileSystemError::Busy)
    }
}

/// Methods which file handles with read priviliges might want to use
impl<'a> LittleFileHandle<'a, ReadPriviliges> {
    pub fn new(
        path: Path,
        filesystem: RwLockReadGuard<'a, Option<RwLock<Directory>>>,
    ) -> Result<Self, FileSystemError> {
        let path = path.clean();
        let locks = unsafe {
            FileHandleLocks::read(
                filesystem.as_ref().ok_or(FileSystemError::PointerError)? as *const _,
                &path,
            )?
        };
        Ok(Self {
            _filesystem: filesystem,
            locks,
            path,
            _phantom: PhantomData,
        })
    }
}

/// The inner locks of a file handle
///
/// This is also where accesses and modifications of the handled file
/// can be made in an abstracted fasion
pub enum FileHandleLocks<'a> {
    Reading {
        root_directory: RwLockReadGuard<'a, Directory>,
        further_locks: Vec<RwLockReadGuard<'a, KaggFile>>,
    },
    Writing {
        root_directory: RwLockReadGuard<'a, Directory>,
        further_locks: Vec<RwLockReadGuard<'a, KaggFile>>,
        write_lock: RwLockWriteGuard<'a, KaggFile>,
    },
    WritingRoot {
        root_directory: RwLockWriteGuard<'a, Directory>,
    },
}
pub enum ManiupulationError {
    IncorrectFileType,
    DoesntExist,
    TryingToWriteOnRead,
}
impl<'a> FileHandleLocks<'a> {
    fn attempt<R>(&self, op: impl FnOnce(&KaggFile) -> R) -> Result<R, ManiupulationError> {
        match self {
            FileHandleLocks::Reading { further_locks, .. } => Ok(op(&**further_locks
                .last()
                .ok_or(ManiupulationError::DoesntExist)?)),
            FileHandleLocks::Writing { write_lock, .. } => Ok(op(&**write_lock)),
            FileHandleLocks::WritingRoot { .. } => Err(ManiupulationError::IncorrectFileType),
        }
    }
    fn attempt_mut<R>(
        &mut self,
        op: impl FnOnce(&mut KaggFile) -> R,
    ) -> Result<R, ManiupulationError> {
        match self {
            FileHandleLocks::Reading { .. } => Err(ManiupulationError::TryingToWriteOnRead),
            FileHandleLocks::Writing { write_lock, .. } => Ok(op(&mut **write_lock)),
            FileHandleLocks::WritingRoot { .. } => Err(ManiupulationError::IncorrectFileType),
        }
    }
    fn attempt_dir_mut<R>(
        &mut self,
        op: impl FnOnce(&mut Directory) -> R,
    ) -> Result<R, ManiupulationError> {
        match self {
            FileHandleLocks::Reading { .. } => Err(ManiupulationError::TryingToWriteOnRead),
            FileHandleLocks::Writing { write_lock, .. } => match &mut **write_lock {
                KaggFile::Directory(dir) => Ok(op(dir)),
                _ => Err(ManiupulationError::IncorrectFileType),
            },
            FileHandleLocks::WritingRoot { root_directory } => Ok(op(&mut **root_directory)),
        }
    }
    fn attempt_dir<R>(&self, op: impl FnOnce(&Directory) -> R) -> Result<R, ManiupulationError> {
        match self {
            FileHandleLocks::Reading {
                further_locks,
                root_directory,
                ..
            } => match further_locks.last().map(|i| &**i) {
                Some(KaggFile::Directory(dir)) => Ok(op(dir)),
                None => Ok(op(&root_directory)),
                _ => Err(ManiupulationError::IncorrectFileType),
            },
            FileHandleLocks::Writing { write_lock, .. } => match &**write_lock {
                KaggFile::Directory(dir) => Ok(op(dir)),
                _ => Err(ManiupulationError::IncorrectFileType),
            },
            FileHandleLocks::WritingRoot { root_directory } => Ok(op(&**root_directory)),
        }
    }

    unsafe fn read(
        filesystem: *const RwLock<Directory>,
        path: &Path,
    ) -> Result<Self, FileSystemError> {
        let segments = path.components();
        let root_directory = filesystem.as_ref().unwrap().read();
        let mut further_locks: Vec<RwLockReadGuard<'_, KaggFile>> = Vec::new();
        for section in segments {
            if section == "" {
                continue;
            }
            let new_lock = if let Some(lock) = further_locks.last() {
                let guard = match &**lock {
                    KaggFile::Directory(directory) => (directory as *const Directory)
                        .as_ref()
                        .ok_or(FileSystemError::PointerError)?
                        .fetch(section),
                    _ => None,
                };
                guard
            } else {
                (&*root_directory as *const Directory)
                    .as_ref()
                    .ok_or(FileSystemError::PointerError)?
                    .fetch(section)
            };
            further_locks.push(new_lock.ok_or(FileSystemError::FileNotFound("case 1"))?);
        }
        Ok(Self::Reading {
            root_directory,
            further_locks,
        })
    }
    unsafe fn write_root(filesystem: *const RwLock<Directory>) -> Result<Self, FileSystemError> {
        Ok(Self::WritingRoot {
            root_directory: filesystem
                .as_ref()
                .ok_or(FileSystemError::PointerError)?
                .write(),
        })
    }
    unsafe fn write(
        filesystem: *const RwLock<Directory>,
        path: &Path,
    ) -> Result<Self, FileSystemError> {
        let segments: Vec<_> = path.components().collect();
        let root_directory = filesystem
            .as_ref()
            .ok_or(FileSystemError::PointerError)?
            .read();
        let mut further_locks: Vec<RwLockReadGuard<'_, KaggFile>> = Vec::new();
        for section in &segments[0..segments.len() - 1] {
            if section == &"" {
                continue;
            }
            let new_lock = if let Some(lock) = further_locks.last() {
                let guard = match &**lock {
                    KaggFile::Directory(directory) => (directory as *const Directory)
                        .as_ref()
                        .ok_or(FileSystemError::PointerError)?
                        .fetch(section),
                    _ => None,
                };
                guard
            } else {
                (&*root_directory as *const Directory)
                    .as_ref()
                    .ok_or(FileSystemError::PointerError)?
                    .fetch(section)
            };
            further_locks.push(new_lock.ok_or(FileSystemError::FileNotFound("case 2"))?);
        }
        let file_name = segments.last().unwrap();
        let write_lock = if let Some(lock) = further_locks.last() {
            let guard = match &**lock {
                KaggFile::Directory(directory) => (directory as *const Directory)
                    .as_ref()
                    .ok_or(FileSystemError::PointerError)?
                    .fetch_write(file_name),
                _ => None,
            };
            guard
        } else {
            (&*root_directory as *const Directory)
                .as_ref()
                .ok_or(FileSystemError::PointerError)?
                .fetch_write(file_name)
        }
        .ok_or(FileSystemError::FileNotFound("case 3"))?;
        Ok(Self::Writing {
            root_directory,
            further_locks,
            write_lock,
        })
    }
}
