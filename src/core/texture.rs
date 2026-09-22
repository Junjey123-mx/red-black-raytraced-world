// Foundation-stage primitive: lands ahead of the UV/sampling and asset
// loading work that will consume it.
#![allow(dead_code)]

use crate::core::color::Color;

/// Reasons a `CpuTexture` cannot be constructed from the given dimensions
/// and pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    ZeroDimension,
    PixelCountMismatch,
    DimensionOverflow,
}

/// Project-owned, fully CPU representation of an already-decoded image: a
/// row-major grid of `Color` texels with the logical origin at the top-left
/// corner. This type knows nothing about files, paths, Raylib images,
/// materials, or UV mapping; it only owns already-decoded pixel data.
pub struct CpuTexture {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl CpuTexture {
    /// Builds a texture from explicit dimensions and row-major pixel data.
    /// Rejects zero dimensions, a pixel count that does not exactly match
    /// `width * height`, and a `width * height` product that would overflow
    /// `usize`.
    pub fn new(width: usize, height: usize, pixels: Vec<Color>) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::ZeroDimension);
        }

        let expected_len = width
            .checked_mul(height)
            .ok_or(TextureError::DimensionOverflow)?;

        if pixels.len() != expected_len {
            return Err(TextureError::PixelCountMismatch);
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    /// Row-major index of texel `(x, y)`, centralizing the single formula
    /// used by every bounded texel lookup.
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Returns the texel at discrete image coordinates `(x, y)`, or `None`
    /// when either coordinate is out of bounds. `y = 0` is the top row.
    pub fn texel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[self.index(x, y)])
    }
}
