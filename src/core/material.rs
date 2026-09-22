// Foundation-stage primitive: lands ahead of the shading work that will
// consume it.
#![allow(dead_code)]

use crate::core::color::Color;

const MIN_SHININESS: f32 = 1.0;
const MAX_SHININESS: f32 = 512.0;

/// Minimal CPU surface material for basic local lighting: a uniform albedo
/// color and the Phong/Blinn-Phong specular parameters. Textures, material
/// identifiers, reflection, refraction, transparency, emission, and normal
/// mapping are explicitly out of scope until later categories.
pub struct Material {
    pub albedo: Color,
    pub specular: f32,
    pub shininess: f32,
}

impl Material {
    /// `specular` is clamped to non-negative, and `shininess` is clamped to
    /// a safe finite range so degenerate input cannot destabilize the
    /// specular term.
    pub fn new(albedo: Color, specular: f32, shininess: f32) -> Self {
        Self {
            albedo,
            specular: specular.max(0.0),
            shininess: shininess.clamp(MIN_SHININESS, MAX_SHININESS),
        }
    }

    /// A matte material: zero specular response.
    pub fn matte(albedo: Color) -> Self {
        Self::new(albedo, 0.0, MIN_SHININESS)
    }

    /// A glossy material: strong, concentrated specular response.
    pub fn glossy(albedo: Color) -> Self {
        Self::new(albedo, 1.0, MAX_SHININESS)
    }
}
