// Foundation-stage primitive: parts of its API (e.g. `white`, `clamp`) land
// ahead of the lighting/shading work that will consume them.
#![allow(dead_code)]

use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn clamp(self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
    }

    /// Converts a clamped, normalized color into display-ready 8-bit RGBA.
    pub fn to_rgba8(self) -> [u8; 4] {
        let clamped = self.clamp();
        [
            (clamped.r * 255.0).round() as u8,
            (clamped.g * 255.0).round() as u8,
            (clamped.b * 255.0).round() as u8,
            (clamped.a * 255.0).round() as u8,
        ]
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

/// Component-wise multiplication, used to tint/filter one color by another.
impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(
            self.r * scalar,
            self.g * scalar,
            self.b * scalar,
            self.a * scalar,
        )
    }
}
