use raylib::prelude::*;

use crate::camera::camera::Camera;
use crate::camera::projection::primary_ray;
use crate::config;
use crate::core::color::Color as CpuColor;
use crate::core::cube::Cube;
use crate::core::math::Vec3;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::raytracer::cast_ray;

/// Casts one primary ray per pixel against `cube` and writes the resulting
/// diagnostic color (or `background` on a miss) into the framebuffer. This
/// is the first fully CPU-computed 3D image in the project.
fn render(framebuffer: &mut Framebuffer, camera: &Camera, cube: &Cube, background: CpuColor) {
    let width = framebuffer.width();
    let height = framebuffer.height();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(camera, x, y, width, height);
            let color = cast_ray(cube, &ray, background);
            framebuffer.set_pixel(x, y, color);
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

    let aspect_ratio = config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32;
    let camera = Camera::new(
        Vec3::new(2.5, 2.0, 4.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    );
    let cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    let background = CpuColor::new(0.05, 0.05, 0.08, 1.0);

    render(&mut framebuffer, &camera, &cube, background);

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
