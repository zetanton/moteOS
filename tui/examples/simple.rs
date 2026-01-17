//! examples/simple.rs
use minifb::{Key, Window, WindowOptions};
use tui::{draw_text, get_default_font, Color, Framebuffer};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let font = get_default_font().expect("Failed to load default font");
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut fb = Framebuffer::new(WIDTH, HEIGHT, &mut buffer);

    draw_text(&mut fb, &font, 10, 10, "Hello, moteOS TUI!", Color::white());
    draw_text(
        &mut fb,
        &font,
        10,
        30,
        "Colored text rendering test.",
        Color::new(255, 0, 255, 255),
    ); // Magenta

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
        window
            .update_with_buffer(&fb.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
