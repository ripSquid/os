
use core::ops::{Deref, DerefMut};

use alloc::{boxed::Box, vec::Vec};
use crate::{Path, AppConstructor};


pub trait DefaultInstall: AppConstructor {
    fn path() -> Path;
}
pub trait InstallableApp: AppConstructor {
    fn install() -> (Path, Box<dyn AppConstructor>);
}
impl<T> InstallableApp for T where T: Default + DefaultInstall + AppConstructor {
    fn install() -> (Path, Box<dyn AppConstructor>) {
        (T::path(), Box::new(T::default()))
    }
} 
