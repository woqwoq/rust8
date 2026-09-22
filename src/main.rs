use crate::display::Display;

mod display;
mod processor;

fn main() {
    let mut display = Display::new();
    let _ = display.draw_pixel(255, 3, 3);
}
