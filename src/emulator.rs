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

pub enum EmulatorError {
    Display(DisplayError),
    Processor(RuntimeError),
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            processor: Processor::new(),
            display: Display::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        }
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        // 60 FPS Loop for screen updates
        while self.display.is_open() {
            // 600 FPS Loop to emulate 600hz CPU execution speed
            for _ in 0..(CPU_CLOCK_SPEED_HZ / SCREEN_FPS) {
                self.processor.step().map_err(EmulatorError::Processor)?;
            }

            self.processor.tick_timers();

            if self.processor.is_dirty {
                self.display
                    .draw_from_binary_buf(&self.processor.display)
                    .map_err(EmulatorError::Display)?;

                self.processor.is_dirty = false;
            }

            self.display.update().map_err(EmulatorError::Display)?;
        }

        Ok(())
    }
}
