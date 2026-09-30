// `pixels` and `clear` land ahead of the raytracer loop that will drive
// per-frame recomputation of the buffer.
#![allow(dead_code)]

use crate::core::color::Color;

pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::black(); width * height],
        }
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

    /// Row-major pixel storage, for writers that own whole regions at once
    /// (the parallel renderer's bands).
    pub fn pixels_mut(&mut self) -> &mut [Color] {
        &mut self.pixels
    }

    /// Resets every pixel to `color` while reusing the existing allocation.
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }

    /// Makes the buffer `width x height`. The same size keeps the storage
    /// (and its contents) untouched; a different size reuses the
    /// allocation's capacity where possible and clears to black, since no
    /// pixel of the old size is meaningful at the new one.
    pub fn resize(&mut self, width: usize, height: usize) {
        if width == self.width && height == self.height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.pixels.resize(width * height, Color::black());
    }

    /// Writes every pixel as 8-bit RGBA (row-major, 4 bytes per pixel,
    /// exactly `Color::to_rgba8`) into `out`, reusing its allocation.
    pub fn write_rgba8(&self, out: &mut Vec<u8>) {
        out.resize(self.pixels.len() * 4, 0);
        // `(v * 255 + 0.5) as u8` is `(v * 255).round() as u8` for a value
        // clamped to `[0, 1]` (round-half-up equals round-half-away-from-zero
        // for non-negative numbers), without the per-channel call.
        let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        for (pixel, bytes) in self.pixels.iter().zip(out.chunks_exact_mut(4)) {
            bytes[0] = channel(pixel.r);
            bytes[1] = channel(pixel.g);
            bytes[2] = channel(pixel.b);
            bytes[3] = channel(pixel.a);
        }
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Writes a pixel; out-of-bounds coordinates are silently ignored.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        self.index(x, y).map(|index| self.pixels[index])
    }
}
