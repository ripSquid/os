use crate::{AppConstructor, PathString};
use alloc::boxed::Box;

pub trait DefaultInstall: AppConstructor {
    fn path() -> PathString;
}
pub trait InstallableApp: AppConstructor {
    fn install() -> (PathString, Box<dyn AppConstructor>);
}
impl<T> InstallableApp for T
where
    T: Default + DefaultInstall + AppConstructor,
{
    fn install() -> (PathString, Box<dyn AppConstructor>) {
        (T::path(), Box::new(T::default()))
    }
}
