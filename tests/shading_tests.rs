#[path = "."]
mod core {
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/material.rs"]
    pub mod material;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
}

use core::color::Color;
use core::material::Material;
use renderer::shading::ambient;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn zero_ambient_factor_produces_black() {
    let material = Material::matte(Color::new(0.8, 0.3, 0.1, 1.0));
    let color = ambient(&material, 0.0);

    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn full_ambient_factor_matches_albedo() {
    let albedo = Color::new(0.6, 0.4, 0.2, 1.0);
    let material = Material::matte(albedo);
    let color = ambient(&material, 1.0);

    assert!(approx_eq(color.r, albedo.r));
    assert!(approx_eq(color.g, albedo.g));
    assert!(approx_eq(color.b, albedo.b));
}

#[test]
fn partial_ambient_factor_scales_albedo() {
    let albedo = Color::new(0.8, 0.8, 0.8, 1.0);
    let material = Material::matte(albedo);
    let color = ambient(&material, 0.5);

    assert!(approx_eq(color.r, 0.4));
    assert!(approx_eq(color.g, 0.4));
    assert!(approx_eq(color.b, 0.4));
}

#[test]
fn negative_ambient_factor_is_clamped_to_zero_contribution() {
    let material = Material::matte(Color::new(0.5, 0.5, 0.5, 1.0));
    let color = ambient(&material, -1.0);

    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn ambient_output_stays_within_normalized_color_range() {
    let material = Material::matte(Color::new(0.9, 0.9, 0.9, 1.0));
    let color = ambient(&material, 5.0);

    assert!((0.0..=1.0).contains(&color.r));
    assert!((0.0..=1.0).contains(&color.g));
    assert!((0.0..=1.0).contains(&color.b));
}

#[test]
fn ambient_result_is_finite() {
    let material = Material::glossy(Color::new(0.2, 0.7, 0.4, 1.0));
    let color = ambient(&material, 0.3);

    assert!(color.r.is_finite());
    assert!(color.g.is_finite());
    assert!(color.b.is_finite());
}
