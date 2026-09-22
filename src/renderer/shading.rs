// Foundation-stage primitive: lands ahead of the raytracer integration
// that will call these shading functions per hit.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::math::Vec3;
use crate::scene::light::DirectionalLight;

pub const DEFAULT_AMBIENT_FACTOR: f32 = 0.1;

/// First local shading term: a deterministic ambient contribution equal to
/// the material albedo scaled by `ambient_factor`. Intentionally
/// independent from surface orientation and light position/direction, and
/// clamped to the project's normalized `Color` range.
pub fn ambient(material: &Material, ambient_factor: f32) -> Color {
    (material.albedo * ambient_factor.max(0.0)).clamp()
}

/// Lambertian diffuse contribution from a directional light:
/// `max(dot(normal, light.direction), 0)`, scaled by the light's color and
/// intensity and modulated by the material albedo. `light.direction`
/// already points from the surface toward the light (see the convention
/// documented on `DirectionalLight`), so no sign flip is needed here.
pub fn diffuse_directional(material: &Material, normal: Vec3, light: &DirectionalLight) -> Color {
    let n_dot_l = normal.dot(light.direction).max(0.0);
    (material.albedo * light.color * (light.intensity * n_dot_l)).clamp()
}
