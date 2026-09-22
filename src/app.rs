use raylib::prelude::*;

use crate::config;
use crate::core::color::Color as CpuColor;
use crate::renderer::framebuffer::Framebuffer;

const CHECKER_CELL_SIZE: usize = 32;

/// Fills the framebuffer with a deterministic checkerboard computed on the CPU.
fn fill_checkerboard(framebuffer: &mut Framebuffer) {
    let bright = CpuColor::new(0.9, 0.4, 0.1, 1.0);
    let dark = CpuColor::new(0.1, 0.1, 0.15, 1.0);

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let is_bright = ((x / CHECKER_CELL_SIZE) + (y / CHECKER_CELL_SIZE)) % 2 == 0;
            framebuffer.set_pixel(x, y, if is_bright { bright } else { dark });
        }
    }
}

/// Copies already-computed framebuffer pixels into a Raylib Image, the only
/// place where `core::Color` is converted to a Raylib-facing type.
fn framebuffer_to_image(framebuffer: &Framebuffer) -> Image {
    let mut image = Image::gen_image_color(
        framebuffer.width() as i32,
        framebuffer.height() as i32,
        Color::BLACK,
    );

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            if let Some(pixel) = framebuffer.get_pixel(x, y) {
                let [r, g, b, a] = pixel.to_rgba8();
                image.draw_pixel(x as i32, y as i32, Color::new(r, g, b, a));
            }
        }
    }

    image
}

pub fn run() {
    let (mut rl, thread) = raylib::init()
        .size(config::WINDOW_WIDTH, config::WINDOW_HEIGHT)
        .title(config::WINDOW_TITLE)
        .build();

    let mut framebuffer = Framebuffer::new(
        config::WINDOW_WIDTH as usize,
        config::WINDOW_HEIGHT as usize,
    );
    fill_checkerboard(&mut framebuffer);

    let image = framebuffer_to_image(&framebuffer);
    let texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("failed to upload the CPU framebuffer to a Raylib texture");

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
