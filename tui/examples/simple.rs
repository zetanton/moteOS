//! examples/simple.rs
use tui::{get_default_font, Color, Framebuffer};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let font = get_default_font().expect("Failed to load default font");
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut fb = Framebuffer::new(WIDTH, HEIGHT, &mut buffer);

    fb.draw_text("Hello, moteOS TUI!", 10, 10, &font, Color::WHITE);
    fb.draw_text("Colored text rendering test.", 10, 30, &font, Color(0x00FF00FF)); // Purple-ish

    let mut window = Window::new(
        "TUI Font Example - Press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&fb.buffer, WIDTH, HEIGHT).unwrap();
    }
}
