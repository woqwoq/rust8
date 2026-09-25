use std::path::PathBuf;

use crate::emulator::Emulator;

mod constants;
mod display;
mod emulator;
mod processor;

fn main() {
    let mut emu =
        Emulator::from_rom_file(PathBuf::from(String::from("src/example/IBM_Logo.ch8"))).unwrap();

    if let Err(e) = emu.run() {
        println!("{e:?}");
    }
}
