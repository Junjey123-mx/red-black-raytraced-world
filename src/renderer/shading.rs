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

/// Reflects `to_light` (the direction from the surface toward the light)
/// around `normal`, producing the direction along which specular energy is
/// concentrated: `R = 2 * dot(N, L) * N - L`. This is a local helper for
/// the Phong specular term only; it is not the recursive reflection-ray
/// system reserved for a later category.
fn reflect_toward_light(normal: Vec3, to_light: Vec3) -> Vec3 {
    normal * (2.0 * normal.dot(to_light)) - to_light
}

/// Phong specular contribution from a directional light: concentrates
/// brightness around the mirror-reflection direction of `light.direction`,
/// scaled by `material.specular`, sharpened by `material.shininess`, and
/// modulated by the light's color and intensity. Zero on back-facing
/// surfaces (`dot(normal, light.direction) <= 0`) regardless of view angle.
pub fn specular_directional(
    material: &Material,
    normal: Vec3,
    view_direction: Vec3,
    light: &DirectionalLight,
) -> Color {
    if material.specular <= 0.0 {
        return Color::black();
    }

    let n_dot_l = normal.dot(light.direction);
    if n_dot_l <= 0.0 {
        return Color::black();
    }

    let reflected = reflect_toward_light(normal, light.direction);
    let r_dot_v = reflected.dot(view_direction).max(0.0);
    let factor = r_dot_v.powf(material.shininess) * material.specular * light.intensity;

    (light.color * factor).clamp()
}
