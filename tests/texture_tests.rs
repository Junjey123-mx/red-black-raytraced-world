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

fn checkerboard_2x2() -> CpuTexture {
    let red = c(1.0, 0.0, 0.0);
    let green = c(0.0, 1.0, 0.0);
    let blue = c(0.0, 0.0, 1.0);
    let white = c(1.0, 1.0, 1.0);
    CpuTexture::new(2, 2, vec![red, green, blue, white]).unwrap()
}

#[test]
fn texel_0_0_is_top_left() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(0, 0), Some(c(1.0, 0.0, 0.0)));
}

#[test]
fn texel_width_minus_one_0_is_top_right() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(1, 0), Some(c(0.0, 1.0, 0.0)));
}

#[test]
fn texel_0_height_minus_one_is_bottom_left() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(0, 1), Some(c(0.0, 0.0, 1.0)));
}

#[test]
fn texel_bottom_right_corner() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(1, 1), Some(c(1.0, 1.0, 1.0)));
}

#[test]
fn texel_x_equal_width_is_out_of_bounds() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(2, 0), None);
}

#[test]
fn texel_y_equal_height_is_out_of_bounds() {
    let texture = checkerboard_2x2();
    assert_eq!(texture.texel(0, 2), None);
}

#[test]
fn rectangular_texture_preserves_row_major_order() {
    let pixels: Vec<Color> = (0..(3 * 2)).map(|i| c(i as f32, 0.0, 0.0)).collect();
    let texture = CpuTexture::new(3, 2, pixels).unwrap();

    assert_eq!(texture.texel(0, 0), Some(c(0.0, 0.0, 0.0)));
    assert_eq!(texture.texel(1, 0), Some(c(1.0, 0.0, 0.0)));
    assert_eq!(texture.texel(2, 0), Some(c(2.0, 0.0, 0.0)));
    assert_eq!(texture.texel(0, 1), Some(c(3.0, 0.0, 0.0)));
    assert_eq!(texture.texel(1, 1), Some(c(4.0, 0.0, 0.0)));
    assert_eq!(texture.texel(2, 1), Some(c(5.0, 0.0, 0.0)));
}

#[test]
fn repeated_reads_are_deterministic() {
    let texture = checkerboard_2x2();
    let first = texture.texel(1, 1);
    let second = texture.texel(1, 1);
    assert_eq!(first, second);
}
