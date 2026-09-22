// Foundation-stage primitive: lands ahead of the shading work that will
// consume it.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::Vec3;

/// Directional light: models a distant source (e.g. the sun) whose rays are
/// treated as parallel across the whole scene.
///
/// **Direction convention (fixed for the whole renderer):** `direction` is
/// the normalized direction FROM any surface point TOWARD the light — i.e.
/// it is already usable directly as the `L` vector in Lambertian diffuse:
/// `max(dot(N, light.direction), 0)`. This is the opposite of "the
/// direction the light travels"; every later shading computation must
/// treat `direction` this way without renegotiating the sign.
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    /// `direction` is normalized safely (a near-zero input normalizes to
    /// the zero vector rather than NaN, matching `Vec3::normalize`).
    /// `intensity` is clamped to non-negative.
    pub fn new(direction: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity: intensity.max(0.0),
        }
    }
}

/// A scene light source. Only the directional variant exists so far; the
/// point-light variant is added in a later commit.
pub enum Light {
    Directional(DirectionalLight),
}
