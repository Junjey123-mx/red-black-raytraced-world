// What a ray sees when it hits nothing: either one flat color (the catalog's
// dark studio backdrop) or the Overworld's procedural daytime sky, a
// direction-dependent gradient. The final six-image cubemap of the
// presentation phase will slot in here as another variant.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::Vec3;

/// Linear blend `a * (1 - t) + b * t`, clamped to the color range.
fn mix(a: Color, b: Color, t: f32) -> Color {
    (a * (1.0 - t) + b * t).clamp()
}

/// A clear daytime sky evaluated per ray direction:
///
/// ```text
/// straight up              -> zenith (the richest blue)
/// toward the horizon       -> horizon (pale, luminous celeste)
/// below the horizon (miss) -> ground haze (very pale blue)
/// ```
///
/// The upward blend is eased so the saturated zenith tone does not reach
/// down to the horizon band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyGradient {
    pub zenith: Color,
    pub horizon: Color,
    pub ground: Color,
}

impl SkyGradient {
    /// Sky color along `direction` (any non-zero length; a zero vector reads
    /// as the horizon).
    pub fn color(&self, direction: Vec3) -> Color {
        let d = direction.normalize();
        let elevation = d.y.clamp(-1.0, 1.0);
        if elevation >= 0.0 {
            // Ease-out: most of the sky dome keeps the horizon's luminosity
            // and only the top deepens toward the zenith.
            let t = elevation.sqrt();
            mix(self.horizon, self.zenith, t)
        } else {
            mix(self.horizon, self.ground, (-elevation).sqrt())
        }
    }
}

/// The scene background: a flat color or a directional sky.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Solid(Color),
    Sky(SkyGradient),
}

impl Background {
    /// Background color for a ray travelling along `direction`.
    pub fn color(&self, direction: Vec3) -> Color {
        match self {
            Background::Solid(color) => *color,
            Background::Sky(sky) => sky.color(direction),
        }
    }

    /// `true` for a directional sky.
    pub fn is_sky(&self) -> bool {
        matches!(self, Background::Sky(_))
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Solid(color)
    }
}
