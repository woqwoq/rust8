use minifb::{Key, Window, WindowOptions};

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

pub struct Display {
    buffer: Vec<u32>,
    window: Window,
}

pub enum DisplayError {
    DrawPixelOutOfRange { x: usize, y: usize },
    UpdateFrameBuffer,
}

impl Display {
    pub fn new() -> Self {
        let buffer = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
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

    pub fn draw_pixel(&mut self, value: u32, x: usize, y: usize) -> Result<(), DisplayError> {
        if !(x < SCREEN_WIDTH && y < SCREEN_HEIGHT) {
            return Err(DisplayError::DrawPixelOutOfRange { x, y });
        }

        self.buffer[x + y * SCREEN_WIDTH] = value;

        Ok(())
    }

    pub fn draw_from_buf(
        &mut self,
        buf: &[u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    ) -> Result<(), DisplayError> {
        self.buffer.extend_from_slice(buf);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), DisplayError> {
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|_| DisplayError::UpdateFrameBuffer)
    }
}
