use std::{fs::File, io::Read, path::PathBuf};

use minifb::Key;

use crate::{
    display::{Display, DisplayError},
    processor::{Processor, RuntimeError},
};
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const SCREEN_FPS: usize = 60;
const CPU_CLOCK_SPEED_HZ: usize = 600;

pub struct Emulator {
    processor: Processor,
    display: Display,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EmulatorError {
    Display(DisplayError),
    Processor(RuntimeError),
    RomOpen,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            processor: Processor::new(),
            display: Display::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        }
    }

    pub fn from_rom_file(path: PathBuf) -> Result<Self, EmulatorError> {
        let mut emu = Self::new();
        match std::fs::read(path) {
            Ok(rom) => emu
                .processor
                .load_rom(rom)
                .map_err(EmulatorError::Processor)?,
            Err(_) => return Err(EmulatorError::RomOpen),
        }

        Ok(emu)
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        // 60 FPS Loop for screen updates
        while self.display.is_open()
            && !self
                .display
                .is_key_down(Key::Escape)
                .map_err(EmulatorError::Display)?
        {
            // 600 FPS Loop to emulate 600hz CPU execution speed
            for _ in 0..(CPU_CLOCK_SPEED_HZ / SCREEN_FPS) {
                self.processor.step().map_err(EmulatorError::Processor)?;
            }

            // Tick timer at 60hz/FPS
            self.processor.tick_timers();

            // Redraw only when pixels are dirty
            if self.processor.is_dirty {
                self.display
                    .draw_from_binary_buf(&self.processor.display)
                    .map_err(EmulatorError::Display)?;

                self.processor.is_dirty = false;
            }

            // Refresh anyway capture key events
            self.display.update().map_err(EmulatorError::Display)?;
        }

        Ok(())
    }
}
