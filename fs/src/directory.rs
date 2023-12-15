use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{File, FileMetadata, FileType, KaggFile, PathString};

#[derive(Default)]
pub struct Directory(HashMap<String, RwLock<KaggFile>>);
impl Directory {
    pub fn fetch(&self, file: &str) -> Option<RwLockReadGuard<'_, KaggFile>> {
        self.0.get(file).map(|s| s.try_read()).flatten()
    }
    pub fn fetch_write(&self, file: &str) -> Option<RwLockWriteGuard<'_, KaggFile>> {
        self.0.get(file).map(|s| s.try_write()).flatten()
    }
    pub fn read_all(&self) -> DirRead {
        DirRead(
            self.0
                .iter()
                .filter_map(|(str, i)| {
                    i.try_read()
                        .map(|succ| match &*succ {
                            KaggFile::Directory(_) => Some(FileMetadata {
                                path: PathString(str.to_string()),
                                filetype: FileType::Directory,
                            }),
                            KaggFile::Data(_) => Some(FileMetadata {
                                path: PathString(str.to_string()),
                                filetype: FileType::Data,
                            }),
                            KaggFile::App(_) => Some(FileMetadata {
                                path: PathString(str.to_string()),
                                filetype: FileType::App,
                            }),
                            _ => None,
                        })
                        .flatten()
                })
                .collect(),
        )
    }
    pub fn add_file(&mut self, file: File) {
        let File { data, name } = file;
        self.0.insert(name, RwLock::new(data));
    }
}

pub struct DirRead(Vec<FileMetadata>);
impl DirRead {
    pub fn items(self) -> impl Iterator<Item = FileMetadata> {
        self.0.into_iter()
    }
}
