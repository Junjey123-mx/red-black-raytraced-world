// Foundation-stage primitive: lands ahead of the raytracer/shading
// integration that will call this sampler per hit.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::Vec2;
use crate::core::texture::CpuTexture;

/// Replaces a non-finite (`NaN` or `+-Infinity`) UV component with `0.0` so
/// it can never reach the texel index computation.
fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

/// Project-owned nearest-neighbor sampler: converts a normalized UV
/// coordinate into a bounded `CpuTexture` texel lookup. `u`/`v` are first
/// stabilized (non-finite inputs become `0.0`) and clamped to `[0, 1]`, then
/// mapped to a texel index via `floor(coordinate * dimension)`, clamped to
/// the last valid index so `u = 1` / `v = 1` select the final texel instead
/// of stepping out of bounds. This is deliberately not bilinear/trilinear
/// filtering, has no mipmaps, and never delegates to Raylib or GPU sampling.
pub fn sample_nearest(texture: &CpuTexture, uv: Vec2) -> Color {
    let u = finite_or_zero(uv.x).clamp(0.0, 1.0);
    let v = finite_or_zero(uv.y).clamp(0.0, 1.0);

    let x = ((u * texture.width() as f32).floor() as usize).min(texture.width() - 1);
    let y = ((v * texture.height() as f32).floor() as usize).min(texture.height() - 1);

    texture
        .texel(x, y)
        .expect("clamped coordinates are always in bounds")
}
