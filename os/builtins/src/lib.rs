#![no_std]
extern crate alloc;
mod apps;
pub use apps::*;
use fs::{FileSystemError, PathString};

pub fn install_all() -> Result<(), FileSystemError> {
    let active_path = fs::active_directory();
    let result = try_install();
    fs::set_active_directory(active_path);
    result
}
fn try_install() -> Result<(), FileSystemError> {
    fs::create_dir(PathString::from("bin")).unwrap();
    fs::create_data_file(
        PathString::from("bin/startup.for"),
        include_str!("startup.txt").as_bytes(),
    )?;
    fs::set_active_directory(PathString::from("bin"));
    fs::install_app::<Help>()?;
    fs::install_app::<Dir>()?;
    fs::install_app::<View>()?;
    fs::install_app::<ForEditorFile>()?;
    fs::install_app::<ForRunner>()?;
    fs::install_app::<ChangeDir>()?;
    fs::install_app::<ClearScreen>()?;
    Ok(())
}
