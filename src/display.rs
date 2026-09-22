use minifb::{Key, Window, WindowOptions};

pub struct Display {
    width: usize,
    height: usize,

    buffer: Vec<u32>,
    window: Window,
}

pub enum DisplayError {
    DrawPixelOutOfRange { x: usize, y: usize },
    UpdateFrameBuffer,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![0; width * height];
        let mut window = Window::new(
            "RUST8 - CHIP8 Emulator",
            width,
            height,
            WindowOptions {
                scale: minifb::Scale::X16,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        window.set_target_fps(60);

        Display {
            width,
            height,
            buffer,
            window,
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn draw_pixel(&mut self, value: u32, x: usize, y: usize) -> Result<(), DisplayError> {
        if !(x < self.width && y < self.height) {
            return Err(DisplayError::DrawPixelOutOfRange { x, y });
        }

        self.buffer[x + y * self.width] = value;

        Ok(())
    }

    pub fn draw_from_binary_buf(&mut self, buf: &[bool]) -> Result<(), DisplayError> {
        for (i, pixel) in buf.iter().enumerate() {
            self.buffer[i] = match &pixel {
                true => 0xFFFFFFFF,
                false => 0x00000000,
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), DisplayError> {
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .map_err(|_| DisplayError::UpdateFrameBuffer)
    }
}
