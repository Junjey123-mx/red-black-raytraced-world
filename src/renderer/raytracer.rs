// Foundation-stage primitive: lands ahead of the full framebuffer render
// loop that will call `cast_ray` once per pixel.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::cube::Cube;
use crate::core::math::Vec3;
use crate::core::ray::Ray;

/// Resolves a primary ray against a single diagnostic cube. This is
/// deliberately unlit: a hit produces a diagnostic color derived purely
/// from the surface normal, and a miss produces `background`. There is no
/// ambient/diffuse/specular shading, material, or light source involved.
pub fn cast_ray(cube: &Cube, ray: &Ray, background: Color) -> Color {
    match cube.intersect(ray, 0.0, f32::INFINITY) {
        Some(hit) => normal_to_diagnostic_color(hit.normal),
        None => background,
    }
}

/// Remaps each unit-normal component from `[-1, 1]` to `[0, 1]` so the six
/// cube faces render as six visually distinct, deterministic colors.
fn normal_to_diagnostic_color(normal: Vec3) -> Color {
    Color::new(
        (normal.x + 1.0) * 0.5,
        (normal.y + 1.0) * 0.5,
        (normal.z + 1.0) * 0.5,
        1.0,
    )
}
