//! examples/simple.rs
use font::{get_default_font, Framebuffer};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let font = get_default_font().unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut fb = Framebuffer::new(WIDTH, HEIGHT, &mut buffer);

    fb.draw_text("Hello, world!", 10, 10, &font);

    let mut window = Window::new(
        "Font Example - Press ESC to exit",
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
        // We don't need to redraw every frame if the text doesn't change.
        // We just do it here for simplicity.
        window.update_with_buffer(&fb.buffer, WIDTH, HEIGHT).unwrap();
    }
}
