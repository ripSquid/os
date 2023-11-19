//this program can't use std since it's on bare metal
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#![feature(error_in_core)]
#![feature(result_flattening)]
#[macro_use]
extern crate bitflags;
extern crate alloc;

use alloc::boxed::Box;
use base::display::{DefaultVgaWriter, VgaColorCombo, VgaPalette, UniversalVgaFormatter};
use base::forth::{ForthMachine, StackItem};
use base::input::KEYBOARD_QUEUE;
use forth::Stack;

use base::*;
use fs::Path;
use interrupt::setup::global_os_time;

use crate::interrupt::pitinit;

use crate::memory::populate_global_allocator;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use memory::paging::PageTableMaster;
use x86_64::instructions::port::PortWriteOnly;

pub mod cpuid;

mod easter_eggs;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;

mod input;
mod interrupt;
mod memory;
mod multiboot_info;

//no mangle tells the compiler to keep the name of this symbol
//this is later used in long_mode.asm, at which point the cpu is prepared to run rust code
#[no_mangle]
pub extern "C" fn rust_start(info: u64) -> ! {
    disable_cursor();

    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");

    let multiboot_info = unsafe {
        multiboot_info::MultibootInfoUnparsed::from_pointer(info as *const MultibootInfoHeader)
    }
    .unwrap();
    let mut active_table = unsafe { PageTableMaster::new() };
    let mut allocator = memory::remap_everything(multiboot_info, &mut active_table);
    unsafe {
        populate_global_allocator(&mut active_table, &mut allocator);
    }
    unsafe { interrupt::setup::setup_interrupts() }
    let cpu_info = cpuid::ProcessorIdentification::gather();

    fs::start();

    builtins::install_all().unwrap();

    let mut forth_machine = ForthMachine::default();

    unsafe {
        pitinit(2400);
    }
    // Start forth application
    let skip = easter_eggs::show_lars();

    let author_text: String = authors.replace(":", " and ");
    let mut formatter = unsafe { DefaultVgaWriter::new_unsafe() };

    formatter
        .clear_screen(display::VgaColor::Black)
        .set_default_colors(VgaColorCombo::on_black(display::VgaColor::Green))
        .write_str("I've succesfully booted, hello world!")
        .next_line();

    formatter
        .next_line()
        .set_default_colors(VgaColorCombo::on_black(display::VgaColor::White))
        .write_str("Version: ")
        .write_str(version)
        .next_line()
        .write_str("Code by: ")
        .write_str(&author_text.as_str())
        .next_line()
        .write_str("CPU vendor: ")
        .write_str(cpu_info.vendor())
        .next_line()
        .next_line()
        .write_str("Skriv [ \"help\" run ] för en introduktion till OperativSystemet")
        .next_line();

    //forth_machine.insert_word(&run, "run");

    if !skip {
        let timestamp = unsafe { global_os_time };
        let duration = 500;

        let mut fade = 0;
        while (unsafe { global_os_time } < timestamp + duration) {
            let time = unsafe { global_os_time } - timestamp;
            let old_fade = fade;
            fade = ((time * u8::MAX as u64) / duration) as u8;
            if fade != old_fade {
                formatter.set_palette(VgaPalette::<32>::DEFAULT_TEXTMODE.fade_factor(fade));
            }
            unsafe { DefaultVgaWriter::new_unsafe().set_position((4, 4)) };
        }
    }
    formatter.set_palette(VgaPalette::<32>::DEFAULT_TEXTMODE);

    unsafe {
        let mut string = String::new();
        formatter.enable_cursor().set_position((0, 8));
        loop {
            formatter
                .write_str(fs::active_directory().as_str())
                .write_str(" > ");
            loop {
                let c = KEYBOARD_QUEUE.getch_blocking();
                match c {
                    '\x08' => {
                        formatter
                            .back_up(string.len())
                            .write_str(&" ".repeat(string.len()))
                            .back_up(string.len());
                        string.pop();
                    }
                    '\n' => {
                        formatter.next_line();
                        let mut new_string = String::new();
                        core::mem::swap(&mut new_string, &mut string);
                        forth_machine.run(new_string, &mut formatter);
                        formatter.next_line();

                        break;
                    }
                    _ => {
                        formatter.back_up(string.len());
                        string.push(c);
                    }
                }
                formatter.write_str(&string);
            }
        }
    };
}

unsafe fn tmp_write(s: String) {
    for char in s.chars() {
        while (x86_64::instructions::port::PortReadOnly::<u8>::new(0x3F8 + 5).read() & 0x20) == 0 {}
        PortWriteOnly::new(0x3f8).write(char as u8);
    }
}

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
        PortWriteOnly::new(0x03D5_u16).write(0x20_u8);
    }
}

#[no_mangle]
pub extern "C" fn keyboard_handler() {
    panic!();
}

/* 
fn run(sm: &mut ForthMachine, formatter: &mut DefaultVgaWriter, _: &mut usize) {
    let path = match sm.stack_mut().pop() {
        Some(StackItem::String(str)) => Ok(Path::from(str)),
        Some(other) => {
            sm.stack_mut().push(other);
            Err("argument was not a path")
        }
        None => Err("No argument passed"),
    };
    match path {
        Ok(path) => {
            let Some(app) = get_app(path, formatter) else {
                formatter.next_line();
                return;
            };
            run_inner(app, formatter, sm);
        }
        Err(msg) => {
            formatter.write_str("RUN: ").write_str(msg);
        }
    }
    formatter.next_line();
}
fn get_app(path: Path, formatter: &mut DefaultVgaWriter) -> Option<Box<dyn LittleManApp>> {
    let path = path.add_extension("run");
    let file = fs::get_file_relative(&path).or(fs::get_file(Path::from("bin").append(&path)));
    match file {
        Ok(file_handle) => file_handle
            .launch_app()
            .map_err(|e| {
                formatter.write_str(&format!("FILESYSTEM: {e:?}, wrong file type"));
                ()
            })
            .ok(),
        Err(error) => {
            formatter.write_str(&format!("FILESYSTEM: {error:?}"));
            None
        }
    }
}
fn run_inner(
    mut app: Box<dyn LittleManApp>,
    formatter: &mut DefaultVgaWriter,
    fm: &mut ForthMachine,
) {
    match app.start(fm.stack_mut()) {
        Ok(_) => {
            let graphics = GraphicsHandle::from_universal(UniversalVgaFormatter::new(default));
            let mut handle = unsafe { OsHandle::new_complicated(graphics, fm) };
            while handle.running() {
                app.update(&mut handle);
            }
            app.shutdown();
        }
        Err(error) => {
            formatter.write_str(&format!("APP START ERROR: {error:?}"));
        }
    }
}
*/
