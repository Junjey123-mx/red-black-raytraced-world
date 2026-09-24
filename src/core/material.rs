// Foundation-stage primitive: lands ahead of the shading work that will
// consume it.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::texture::TextureId;

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

/// CPU surface material: the central optical contract of the renderer.
///
/// `albedo` is the uniform base color; `face_textures` (the albedo texture
/// source) overrides it per hit face when present (see
/// `renderer::raytracer`), and never affects `specular`/`shininess`.
/// `emissive_texture` and `normal_texture` are optional texture references
/// sharing the albedo UV. The optical scalars are always kept in a valid
/// range by the constructors/builders:
///
/// - `reflectivity` in `[0, 1]`
/// - `transparency` in `[0, 1]`
/// - `refractive_index` > 0 (finite)
/// - `emission_strength` >= 0
///
/// A `Material` never owns pixel data: textures are `TextureId`s resolved
/// through the `TextureManager`. Blocks reference a material by
/// `MaterialId`, resolved through the scene-owned `MaterialLibrary`.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub albedo: Color,
    pub specular: f32,
    pub shininess: f32,
    pub face_textures: Option<FaceTextures>,
    pub emissive_texture: Option<TextureId>,
    pub normal_texture: Option<TextureId>,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
    pub emission_strength: f32,
}

/// Vacuum/air index, the neutral default for materials that do not refract.
pub const DEFAULT_REFRACTIVE_INDEX: f32 = 1.0;

/// Returns `value` when finite, otherwise `fallback`, so NaN/infinity can
/// never enter the optical parameters.
fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

impl Material {
    /// `specular` is clamped to non-negative, and `shininess` is clamped to
    /// a safe finite range so degenerate input cannot destabilize the
    /// specular term. Every advanced optical property starts neutral: an
    /// opaque, non-reflective, non-emissive material with no extra textures.
    pub fn new(albedo: Color, specular: f32, shininess: f32) -> Self {
        Self {
            albedo,
            specular: specular.max(0.0),
            shininess: shininess.clamp(MIN_SHININESS, MAX_SHININESS),
            face_textures: None,
            emissive_texture: None,
            normal_texture: None,
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: DEFAULT_REFRACTIVE_INDEX,
            emission_strength: 0.0,
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

    /// Attaches an emissive mask texture sharing the albedo UV.
    pub fn with_emissive_texture(mut self, texture: TextureId) -> Self {
        self.emissive_texture = Some(texture);
        self
    }

    /// Attaches a tangent-space normal texture sharing the albedo UV.
    pub fn with_normal_texture(mut self, texture: TextureId) -> Self {
        self.normal_texture = Some(texture);
        self
    }

    /// Mirror-reflection weight, clamped to `[0, 1]`.
    pub fn with_reflectivity(mut self, reflectivity: f32) -> Self {
        self.reflectivity = finite_or(reflectivity, 0.0).clamp(0.0, 1.0);
        self
    }

    /// Fraction of light transmitted through the surface, clamped to `[0, 1]`.
    pub fn with_transparency(mut self, transparency: f32) -> Self {
        self.transparency = finite_or(transparency, 0.0).clamp(0.0, 1.0);
        self
    }

    /// Index of refraction. A non-finite or non-positive value falls back to
    /// the neutral index so Snell's law never sees an invalid medium.
    pub fn with_refractive_index(mut self, refractive_index: f32) -> Self {
        self.refractive_index = if refractive_index.is_finite() && refractive_index > 0.0 {
            refractive_index
        } else {
            DEFAULT_REFRACTIVE_INDEX
        };
        self
    }

    /// Emission scale, clamped to non-negative.
    pub fn with_emission_strength(mut self, emission_strength: f32) -> Self {
        self.emission_strength = finite_or(emission_strength, 0.0).max(0.0);
        self
    }
}
