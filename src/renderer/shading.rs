// Foundation-stage primitive: lands ahead of the raytracer integration
// that will call these shading functions per hit.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::math::Vec3;
use crate::scene::light::{DirectionalLight, PointLight};

pub const DEFAULT_AMBIENT_FACTOR: f32 = 0.1;

/// First local shading term: a deterministic ambient contribution equal to
/// the material albedo scaled by `ambient_factor`. Intentionally
/// independent from surface orientation and light position/direction, and
/// clamped to the project's normalized `Color` range.
pub fn ambient(material: &Material, ambient_factor: f32) -> Color {
    (material.albedo * ambient_factor.max(0.0)).clamp()
}

/// Shared Lambertian diffuse term, independent of how `light_direction`
/// (surface-to-light, already unit length) was derived.
fn diffuse_from(
    material: &Material,
    normal: Vec3,
    light_direction: Vec3,
    light_color: Color,
    intensity: f32,
) -> Color {
    let n_dot_l = normal.dot(light_direction).max(0.0);
    (material.albedo * light_color * (intensity * n_dot_l)).clamp()
}

/// Lambertian diffuse contribution from a directional light:
/// `max(dot(normal, light.direction), 0)`, scaled by the light's color and
/// intensity and modulated by the material albedo. `light.direction`
/// already points from the surface toward the light (see the convention
/// documented on `DirectionalLight`), so no sign flip is needed here.
pub fn diffuse_directional(material: &Material, normal: Vec3, light: &DirectionalLight) -> Color {
    diffuse_from(
        material,
        normal,
        light.direction,
        light.color,
        light.intensity,
    )
}

/// Lambertian diffuse contribution from a point light. The surface-to-light
/// direction is derived per hit as `normalize(light.position - hit_point)`,
/// then shared with the same underlying diffuse math as the directional
/// path. A hit point (near-)coincident with the light position has no
/// well-defined direction and safely contributes nothing rather than
/// producing NaN.
pub fn diffuse_point(
    material: &Material,
    normal: Vec3,
    hit_point: Vec3,
    light: &PointLight,
) -> Color {
    let to_light = light.position - hit_point;
    let distance = to_light.length();
    if distance <= f32::EPSILON {
        return Color::black();
    }

    diffuse_from(
        material,
        normal,
        to_light / distance,
        light.color,
        light.intensity,
    )
}

/// Reflects `to_light` (the direction from the surface toward the light)
/// around `normal`, producing the direction along which specular energy is
/// concentrated: `R = 2 * dot(N, L) * N - L`. This is a local helper for
/// the Phong specular term only; it is not the recursive reflection-ray
/// system reserved for a later category.
fn reflect_toward_light(normal: Vec3, to_light: Vec3) -> Vec3 {
    normal * (2.0 * normal.dot(to_light)) - to_light
}

/// Shared Phong specular term, independent of how `light_direction`
/// (surface-to-light, already unit length) was derived.
fn specular_from(
    material: &Material,
    normal: Vec3,
    light_direction: Vec3,
    view_direction: Vec3,
    light_color: Color,
    intensity: f32,
) -> Color {
    if material.specular <= 0.0 {
        return Color::black();
    }

    let n_dot_l = normal.dot(light_direction);
    if n_dot_l <= 0.0 {
        return Color::black();
    }

    let reflected = reflect_toward_light(normal, light_direction);
    let r_dot_v = reflected.dot(view_direction).max(0.0);
    let factor = r_dot_v.powf(material.shininess) * material.specular * intensity;

    (light_color * factor).clamp()
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
    specular_from(
        material,
        normal,
        light.direction,
        view_direction,
        light.color,
        light.intensity,
    )
}

/// Phong specular contribution from a point light, sharing the same
/// underlying math as the directional path via the per-hit surface-to-light
/// direction. A hit point (near-)coincident with the light position safely
/// contributes nothing.
pub fn specular_point(
    material: &Material,
    normal: Vec3,
    hit_point: Vec3,
    view_direction: Vec3,
    light: &PointLight,
) -> Color {
    let to_light = light.position - hit_point;
    let distance = to_light.length();
    if distance <= f32::EPSILON {
        return Color::black();
    }

    specular_from(
        material,
        normal,
        to_light / distance,
        view_direction,
        light.color,
        light.intensity,
    )
}
