// Loading-stage component: bridges real PNG assets on disk into the
// project-owned `CpuTexture` representation, caching by path so a texture
// requested twice is decoded only once.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use raylib::prelude::Image;

use crate::core::color::Color;
use crate::core::texture::{CpuTexture, TextureError, TextureId};

/// Reasons a texture load can fail. Distinguishes a decode/file failure
/// (Raylib could not produce an `Image` for the given path) from a
/// downstream rejection by `CpuTexture::new` itself.
#[derive(Debug)]
pub enum TextureLoadError {
    DecodeFailed,
    InvalidTexture(TextureError),
}

/// Owns every `CpuTexture` loaded so far and caches `path -> TextureId` so
/// requesting the same file twice returns the same id without decoding or
/// storing the image a second time. This is deliberately not a general
/// asset manager: no hot reload, no invalidation, no async loading.
pub struct TextureManager {
    textures: Vec<CpuTexture>,
    ids_by_path: HashMap<PathBuf, TextureId>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            ids_by_path: HashMap::new(),
        }
    }

    /// Returns the `TextureId` for `path`, decoding and caching it on first
    /// request. A subsequent call with the same path returns the same id
    /// without touching disk again or growing internal storage.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<TextureId, TextureLoadError> {
        let path = path.as_ref();

        if let Some(&id) = self.ids_by_path.get(path) {
            return Ok(id);
        }

        let texture = decode_png_to_cpu_texture(path)?;
        let id = TextureId::new(self.textures.len());
        self.textures.push(texture);
        self.ids_by_path.insert(path.to_path_buf(), id);
        Ok(id)
    }

    /// Looks up an already-loaded texture by id.
    pub fn get(&self, id: TextureId) -> Option<&CpuTexture> {
        self.textures.get(id.index())
    }

    /// Number of distinct textures currently stored.
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodes a PNG at `path` via Raylib into an `Image`, then copies its
/// pixels into a project-owned `CpuTexture`. Raylib's role ends the moment
/// `get_image_data` returns: the resulting `CpuTexture` holds only
/// `core::Color` values and knows nothing about `Image`, `Texture2D`, or
/// any other Raylib/GPU type.
fn decode_png_to_cpu_texture(path: &Path) -> Result<CpuTexture, TextureLoadError> {
    let path_str = path.to_str().ok_or(TextureLoadError::DecodeFailed)?;
    let image = Image::load_image(path_str).map_err(|_| TextureLoadError::DecodeFailed)?;

    let width = image.width();
    let height = image.height();
    if width <= 0 || height <= 0 {
        return Err(TextureLoadError::DecodeFailed);
    }

    let raw_pixels = image.get_image_data();
    let pixels: Vec<Color> = raw_pixels
        .iter()
        .map(|pixel| {
            Color::new(
                pixel.r as f32 / 255.0,
                pixel.g as f32 / 255.0,
                pixel.b as f32 / 255.0,
                pixel.a as f32 / 255.0,
            )
        })
        .collect();

    CpuTexture::new(width as usize, height as usize, pixels)
        .map_err(TextureLoadError::InvalidTexture)
}
