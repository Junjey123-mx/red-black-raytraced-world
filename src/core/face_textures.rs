// Foundation-stage component: a pure Face -> TextureId lookup table. It
// owns no `CpuTexture` and does not touch `TextureManager`; resolving a
// `TextureId` into pixel data remains `TextureManager`'s responsibility.
#![allow(dead_code)]

use crate::core::hit::Face;
use crate::core::texture::TextureId;

/// Explicit, exhaustive assignment of a `TextureId` to each of the six
/// axis-aligned cube faces. A plain six-field struct rather than a
/// `HashMap<Face, TextureId>`: `Face` already has exactly six known
/// variants, so this stays allocation-free and exhaustive at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceTextures {
    positive_x: TextureId,
    negative_x: TextureId,
    positive_y: TextureId,
    negative_y: TextureId,
    positive_z: TextureId,
    negative_z: TextureId,
}

impl FaceTextures {
    pub fn new(
        positive_x: TextureId,
        negative_x: TextureId,
        positive_y: TextureId,
        negative_y: TextureId,
        positive_z: TextureId,
        negative_z: TextureId,
    ) -> Self {
        Self {
            positive_x,
            negative_x,
            positive_y,
            negative_y,
            positive_z,
            negative_z,
        }
    }

    /// Assigns the same `TextureId` to all six faces.
    pub fn uniform(texture_id: TextureId) -> Self {
        Self::new(
            texture_id, texture_id, texture_id, texture_id, texture_id, texture_id,
        )
    }

    /// Resolves the `TextureId` assigned to `face`. Exhaustive over `Face`;
    /// a new `Face` variant would fail to compile here rather than silently
    /// falling back to a default.
    pub fn texture_for_face(&self, face: Face) -> TextureId {
        match face {
            Face::PositiveX => self.positive_x,
            Face::NegativeX => self.negative_x,
            Face::PositiveY => self.positive_y,
            Face::NegativeY => self.negative_y,
            Face::PositiveZ => self.positive_z,
            Face::NegativeZ => self.negative_z,
        }
    }
}
