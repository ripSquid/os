#![no_std]
extern crate alloc;
mod apps;
pub use apps::*;
use fs::{FileSystemError, Path};

pub fn install_all() -> Result<(), FileSystemError> {
    let active_path = fs::active_directory();
    let result = try_install();
    fs::set_active_directory(active_path);
    result
}
fn try_install() -> Result<(), FileSystemError> {
    fs::create_dir(Path::from("bin"))?;
    fs::set_active_directory(Path::from("bin"));
    fs::install_app::<Help>()?;
    fs::install_app::<Dir>()?;
    fs::install_app::<ChangeDir>()?;
    fs::install_app::<ClearScreen>()?;
    Ok(())

}