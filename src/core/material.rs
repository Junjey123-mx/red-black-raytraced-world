// Foundation-stage primitive: lands ahead of the shading work that will
// consume it.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;

const MIN_SHININESS: f32 = 1.0;
const MAX_SHININESS: f32 = 512.0;

/// Strongly-typed, lightweight reference to a material. Blocks store this
/// instead of a full `Material`, so material ownership stays centralized
/// and a bare integer is never passed around as an implicit material key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(u32);

impl MaterialId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

/// Minimal CPU surface material for basic local lighting: a uniform albedo
/// color and the Phong/Blinn-Phong specular parameters. `face_textures`, if
/// present, overrides `albedo` as the ambient/diffuse albedo source per
/// hit face (see `renderer::raytracer::cast_ray_lit`); it never affects
/// `specular`/`shininess`. Material identifiers, material libraries,
/// reflection, refraction, transparency, emission, and normal mapping are
/// explicitly out of scope until later categories.
pub struct Material {
    pub albedo: Color,
    pub specular: f32,
    pub shininess: f32,
    pub face_textures: Option<FaceTextures>,
}

impl Material {
    /// `specular` is clamped to non-negative, and `shininess` is clamped to
    /// a safe finite range so degenerate input cannot destabilize the
    /// specular term. `face_textures` starts unset (uniform `albedo`).
    pub fn new(albedo: Color, specular: f32, shininess: f32) -> Self {
        Self {
            albedo,
            specular: specular.max(0.0),
            shininess: shininess.clamp(MIN_SHININESS, MAX_SHININESS),
            face_textures: None,
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

    /// Attaches a per-face texture albedo source to this material. `albedo`
    /// remains as the fallback/base color; once set, `face_textures`
    /// overrides the ambient/diffuse albedo used per hit face.
    pub fn with_face_textures(mut self, face_textures: FaceTextures) -> Self {
        self.face_textures = Some(face_textures);
        self
    }
}
