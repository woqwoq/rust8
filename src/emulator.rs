use std::{fmt, path::PathBuf};

use minifb::Key;

use crate::{
    constants::{CPU_CLOCK_SPEED_HZ, SCREEN_FPS},
    display::{Display, DisplayError},
    processor::{Processor, RuntimeError},
    sound::Speaker,
};
pub struct Emulator {
    processor: Processor,
    display: Display,
    speaker: Speaker,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EmulatorError {
    Display(DisplayError),
    Processor(RuntimeError),
    RomOpen,
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorError::Display(e) => write!(f, "display error: {e}"),
            EmulatorError::Processor(e) => write!(f, "processor error: {e}"),
            EmulatorError::RomOpen => write!(f, "failed to open or read ROM file"),
        }
    }
}

impl std::error::Error for EmulatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EmulatorError::Display(e) => Some(e),
            EmulatorError::Processor(e) => Some(e),
            EmulatorError::RomOpen => None,
        }
    }
}

impl From<RuntimeError> for EmulatorError {
    fn from(value: RuntimeError) -> Self {
        EmulatorError::Processor(value)
    }
}

impl From<DisplayError> for EmulatorError {
    fn from(value: DisplayError) -> Self {
        EmulatorError::Display(value)
    }
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            processor: Processor::new(),
            display: Display::new(),
            speaker: Speaker::new(),
        }
    }

    pub fn from_rom_file(path: PathBuf) -> Result<Self, EmulatorError> {
        let mut emu = Self::new();
        match std::fs::read(path) {
            Ok(rom) => emu.processor.load_rom(&rom)?,
            Err(_) => return Err(EmulatorError::RomOpen),
        }

        Ok(emu)
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        // 60 FPS Loop for screen updates
        while self.display.is_open() && !self.display.is_key_down(Key::Escape) {
            self.processor.keypad = self.display.poll_tracked_keys()?;

            // 600 FPS Loop to emulate 600hz CPU execution speed
            for _ in 0..(CPU_CLOCK_SPEED_HZ / SCREEN_FPS) {
                self.processor.step()?;
            }

            if self.processor.should_beep() {
                self.speaker.play();
            } else {
                self.speaker.pause();
            }

            // Tick timer at 60hz/FPS
            self.processor.tick_timers();

            // Redraw only when pixels are dirty
            if self.processor.is_dirty {
                self.display.draw_from_binary_buf(&self.processor.display)?;

                self.processor.is_dirty = false;
            }

            // Refresh anyway capture key events
            self.display.update()?;
        }

        Ok(())
    }
}
