#![no_std]
#![feature(const_mut_refs)]

use x86_64::instructions::port::PortWriteOnly;
pub mod display;
pub use display::macros::*;

