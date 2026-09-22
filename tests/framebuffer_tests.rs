#[path = "."]
mod core {
    #[path = "../src/core/color.rs"]
    pub mod color;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
}

use core::color::Color;
use renderer::framebuffer::Framebuffer;

#[test]
fn pixel_count_matches_dimensions() {
    let fb = Framebuffer::new(4, 3);
    assert_eq!(fb.pixels().len(), 4 * 3);
}

#[test]
fn clear_fills_every_pixel() {
    let mut fb = Framebuffer::new(2, 2);
    fb.clear(Color::white());

    for pixel in fb.pixels() {
        assert_eq!(*pixel, Color::white());
    }
}

#[test]
fn corner_writes_are_isolated() {
    let mut fb = Framebuffer::new(4, 3);
    fb.clear(Color::black());

    fb.set_pixel(0, 0, Color::white());
    fb.set_pixel(3, 2, Color::white());

    assert_eq!(fb.get_pixel(0, 0), Some(Color::white()));
    assert_eq!(fb.get_pixel(3, 2), Some(Color::white()));
    assert_eq!(fb.get_pixel(1, 1), Some(Color::black()));
}

#[test]
fn out_of_bounds_access_does_not_corrupt_memory() {
    let mut fb = Framebuffer::new(2, 2);
    fb.set_pixel(99, 99, Color::white());

    assert_eq!(fb.get_pixel(99, 99), None);
    for pixel in fb.pixels() {
        assert_eq!(*pixel, Color::black());
    }
}

#[test]
fn buffer_is_reused_across_successive_clears() {
    let mut fb = Framebuffer::new(3, 3);
    let pixels_ptr_before = fb.pixels().as_ptr();

    fb.clear(Color::white());
    fb.clear(Color::black());

    let pixels_ptr_after = fb.pixels().as_ptr();
    assert_eq!(pixels_ptr_before, pixels_ptr_after);
    assert_eq!(fb.width(), 3);
    assert_eq!(fb.height(), 3);
}
