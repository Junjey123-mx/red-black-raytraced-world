#[path = "."]
mod core {
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

use core::color::Color;
use core::texture::{CpuTexture, TextureError};

fn c(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

#[test]
fn one_by_one_texture_is_valid() {
    let texture = CpuTexture::new(1, 1, vec![c(1.0, 0.0, 0.0)]).unwrap();
    assert_eq!(texture.width(), 1);
    assert_eq!(texture.height(), 1);
}

#[test]
fn rectangular_texture_is_valid() {
    let pixels = vec![c(0.0, 0.0, 0.0); 3 * 2];
    let texture = CpuTexture::new(3, 2, pixels).unwrap();
    assert_eq!(texture.width(), 3);
    assert_eq!(texture.height(), 2);
}

#[test]
fn texel_count_matches_dimensions() {
    let pixels = vec![c(0.0, 0.0, 0.0); 4 * 5];
    let texture = CpuTexture::new(4, 5, pixels).unwrap();
    assert_eq!(texture.pixels().len(), 4 * 5);
}

#[test]
fn zero_width_is_rejected() {
    let result = CpuTexture::new(0, 4, vec![]);
    assert!(matches!(result, Err(TextureError::ZeroDimension)));
}

#[test]
fn zero_height_is_rejected() {
    let result = CpuTexture::new(4, 0, vec![]);
    assert!(matches!(result, Err(TextureError::ZeroDimension)));
}

#[test]
fn mismatched_pixel_count_is_rejected() {
    let pixels = vec![c(0.0, 0.0, 0.0); 3];
    let result = CpuTexture::new(2, 2, pixels);
    assert!(matches!(result, Err(TextureError::PixelCountMismatch)));
}

#[test]
fn dimensions_are_reported_correctly() {
    let pixels = vec![c(0.0, 0.0, 0.0); 7 * 9];
    let texture = CpuTexture::new(7, 9, pixels).unwrap();
    assert_eq!(texture.width(), 7);
    assert_eq!(texture.height(), 9);
}

#[test]
fn stored_colors_are_preserved_exactly() {
    let pixels = vec![
        c(0.25, 0.5, 0.75),
        c(1.0, 1.0, 1.0),
        c(0.0, 0.0, 0.0),
        c(0.1, 0.2, 0.3),
    ];
    let texture = CpuTexture::new(2, 2, pixels.clone()).unwrap();
    for (stored, expected) in texture.pixels().iter().zip(pixels.iter()) {
        assert_eq!(stored, expected);
    }
}

#[test]
fn dimension_overflow_is_rejected() {
    let result = CpuTexture::new(usize::MAX, 2, vec![]);
    assert!(matches!(result, Err(TextureError::DimensionOverflow)));
}
