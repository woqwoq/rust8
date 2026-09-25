use minifb::{Key, Window, WindowOptions};

use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Display {
    buffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    window: Window,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayError {
    DrawPixelOutOfRange(usize),
    UpdateFrameBuffer,
}

impl Display {
    pub fn new() -> Self {
        let buffer = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        let mut window = Window::new(
            "RUST8 - CHIP8 Emulator",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions {
                scale: minifb::Scale::X16,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        window.set_target_fps(60);

        Display { buffer, window }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn is_key_down(&self, key: Key) -> Result<bool, DisplayError> {
        Ok(self.window.is_key_down(key))
    }

    pub fn draw_from_binary_buf(&mut self, buf: &[bool]) -> Result<(), DisplayError> {
        for (i, pixel) in buf.iter().enumerate() {
            if i >= self.buffer.len() {
                return Err(DisplayError::DrawPixelOutOfRange(i));
            }

            self.buffer[i] = match &pixel {
                true => 0xFFFFFFFF,
                false => 0x00000000,
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), DisplayError> {
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|_| DisplayError::UpdateFrameBuffer)
    }
}
