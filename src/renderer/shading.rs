// Foundation-stage primitive: lands ahead of the raytracer integration
// that will call these shading functions per hit.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::material::Material;

pub const DEFAULT_AMBIENT_FACTOR: f32 = 0.1;

/// First local shading term: a deterministic ambient contribution equal to
/// the material albedo scaled by `ambient_factor`. Intentionally
/// independent from surface orientation and light position/direction, and
/// clamped to the project's normalized `Color` range.
pub fn ambient(material: &Material, ambient_factor: f32) -> Color {
    (material.albedo * ambient_factor.max(0.0)).clamp()
}
