#[path = "."]
mod core {
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;

        pub use vec2::Vec2;
    }
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
}

use core::color::Color;
use core::math::Vec2;
use core::texture::CpuTexture;
use renderer::texture_sampling::sample_nearest;

fn c(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

fn checkerboard_2x2() -> CpuTexture {
    let red = c(1.0, 0.0, 0.0);
    let green = c(0.0, 1.0, 0.0);
    let blue = c(0.0, 0.0, 1.0);
    let white = c(1.0, 1.0, 1.0);
    CpuTexture::new(2, 2, vec![red, green, blue, white]).unwrap()
}

fn distinct_4x4() -> CpuTexture {
    let pixels: Vec<Color> = (0..16).map(|i| c(i as f32, 0.0, 0.0)).collect();
    CpuTexture::new(4, 4, pixels).unwrap()
}

#[test]
fn uv_0_0_samples_top_left() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 0.0)),
        c(1.0, 0.0, 0.0)
    );
}

#[test]
fn uv_1_0_samples_top_right() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(1.0, 0.0)),
        c(0.0, 1.0, 0.0)
    );
}

#[test]
fn uv_0_1_samples_bottom_left() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 1.0)),
        c(0.0, 0.0, 1.0)
    );
}

#[test]
fn uv_1_1_samples_bottom_right() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(1.0, 1.0)),
        c(1.0, 1.0, 1.0)
    );
}

#[test]
fn uv_within_first_texel() {
    let texture = distinct_4x4();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.1, 0.1)),
        c(0.0, 0.0, 0.0)
    );
}

#[test]
fn uv_within_another_known_texel() {
    let texture = distinct_4x4();
    // x = floor(0.6*4) = 2, y = floor(0.6*4) = 2 -> index 2*4+2 = 10
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.6, 0.6)),
        c(10.0, 0.0, 0.0)
    );
}

#[test]
fn exact_texel_boundary_follows_floor_policy() {
    let texture = distinct_4x4();
    // u = 0.5 -> x = floor(0.5*4) = 2 (belongs to the texel to the right of the boundary)
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.5, 0.0)),
        c(2.0, 0.0, 0.0)
    );
}

#[test]
fn negative_u_clamps_to_zero() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(-5.0, 0.0)),
        c(1.0, 0.0, 0.0)
    );
}

#[test]
fn negative_v_clamps_to_zero() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, -5.0)),
        c(1.0, 0.0, 0.0)
    );
}

#[test]
fn u_greater_than_one_clamps_to_one() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(5.0, 0.0)),
        c(0.0, 1.0, 0.0)
    );
}

#[test]
fn v_greater_than_one_clamps_to_one() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 5.0)),
        c(0.0, 0.0, 1.0)
    );
}

#[test]
fn non_finite_uv_never_panics_and_follows_zero_policy() {
    let texture = checkerboard_2x2();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(f32::NAN, f32::NAN)),
        c(1.0, 0.0, 0.0)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(f32::INFINITY, f32::NEG_INFINITY)),
        c(1.0, 0.0, 0.0)
    );
}

#[test]
fn one_by_one_texture_always_returns_its_only_color() {
    let texture = CpuTexture::new(1, 1, vec![c(0.3, 0.4, 0.5)]).unwrap();
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 0.0)),
        c(0.3, 0.4, 0.5)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(1.0, 1.0)),
        c(0.3, 0.4, 0.5)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.5, 0.5)),
        c(0.3, 0.4, 0.5)
    );
}

#[test]
fn non_square_texture_samples_correctly() {
    // 4 wide, 2 tall.
    let pixels: Vec<Color> = (0..8).map(|i| c(i as f32, 0.0, 0.0)).collect();
    let texture = CpuTexture::new(4, 2, pixels).unwrap();

    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 0.0)),
        c(0.0, 0.0, 0.0)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(1.0, 0.0)),
        c(3.0, 0.0, 0.0)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(0.0, 1.0)),
        c(4.0, 0.0, 0.0)
    );
    assert_eq!(
        sample_nearest(&texture, Vec2::new(1.0, 1.0)),
        c(7.0, 0.0, 0.0)
    );
}
