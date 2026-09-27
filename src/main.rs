use std::path::PathBuf;

use crate::emulator::Emulator;

mod constants;
mod display;
mod emulator;
mod processor;
mod sound;

fn main() {
    let mut emu = Emulator::from_rom_file(PathBuf::from("src/example/Space_Invaders.ch8")).unwrap();

    if let Err(e) = emu.run() {
        println!("{e:?}");
    }
}
