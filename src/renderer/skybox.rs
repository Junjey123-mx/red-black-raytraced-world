// What a ray sees when it hits nothing: one flat color (the catalog's dark
// studio backdrop), a procedural direction-dependent gradient, or the
// World's skybox: a six-image cubemap sampled on the CPU by ray direction.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::{Vec2, Vec3};
use crate::core::texture::TextureId;
use crate::renderer::texture_sampling::sample_nearest;
use crate::scene::texture_manager::TextureManager;

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
    /// The sky's vertical: the zenith lies along it. +Y for the Overworld;
    /// the inverted realm hangs its sky along -Y.
    pub up: Vec3,
}

impl SkyGradient {
    /// Sky color along `direction` (any non-zero length; a zero vector reads
    /// as the horizon).
    pub fn color(&self, direction: Vec3) -> Color {
        let d = direction.normalize();
        let up = self.up.normalize();
        let elevation = d.dot(up).clamp(-1.0, 1.0);
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

/// File stems of a cubemap's six faces, in `CubemapFaces` order.
pub const CUBEMAP_FACE_NAMES: [&str; 6] = ["px", "nx", "py", "ny", "pz", "nz"];

/// The six images of a sky cube: +X, -X, +Y (zenith), -Y, +Z, -Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubemapFaces(pub [TextureId; 6]);

/// Which face of the cube a direction looks at and where on it: the face
/// index (in `CubemapFaces` order) and its UV, `v` running down the image.
/// A zero vector reads as the middle of the +Z face.
pub fn cubemap_face_uv(direction: Vec3) -> (usize, Vec2) {
    let (x, y, z) = (direction.x, direction.y, direction.z);
    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
    let (face, a, b) = if ax >= ay && ax >= az && ax > 0.0 {
        if x > 0.0 {
            (0, -z / ax, -y / ax)
        } else {
            (1, z / ax, -y / ax)
        }
    } else if ay >= az && ay > 0.0 {
        if y > 0.0 {
            (2, x / ay, z / ay)
        } else {
            (3, x / ay, -z / ay)
        }
    } else if az > 0.0 {
        if z > 0.0 {
            (4, x / az, -y / az)
        } else {
            (5, -x / az, -y / az)
        }
    } else {
        (4, 0.0, 0.0)
    };
    (face, Vec2::new((a + 1.0) * 0.5, (b + 1.0) * 0.5))
}

/// The World's skybox: a cubemap hung along `up` (its +Y face is the
/// zenith), cross-faded toward a second cubemap while the camera crosses
/// the portal. Every miss ray samples it on the CPU, nearest-neighbor like
/// every other texture; `fallback` answers if a face is not loaded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyCubemap {
    pub from: CubemapFaces,
    pub to: CubemapFaces,
    /// `0` shows `from`, `1` shows `to`.
    pub blend: f32,
    pub up: Vec3,
    pub fallback: SkyGradient,
}

impl SkyCubemap {
    /// `direction` in the cube's own frame, whose +Y is `up`: the shortest
    /// rotation taking +Y onto `up` (a half turn about X when they oppose).
    fn local(&self, direction: Vec3) -> Vec3 {
        let up = self.up.normalize();
        let y = Vec3::new(0.0, 1.0, 0.0);
        let axis = y.cross(up);
        let (sin, cos) = (axis.length(), y.dot(up));
        if sin < 1e-5 {
            return if cos >= 0.0 {
                direction
            } else {
                Vec3::new(direction.x, -direction.y, -direction.z)
            };
        }
        // Rodrigues by the inverse angle about the same axis.
        let k = axis * (1.0 / sin);
        direction * cos - k.cross(direction) * sin + k * (k.dot(direction) * (1.0 - cos))
    }

    pub fn color(&self, direction: Vec3, textures: &TextureManager) -> Color {
        let (face, uv) = cubemap_face_uv(self.local(direction.normalize()));
        let texel = |faces: &CubemapFaces| {
            textures
                .get(faces.0[face])
                .map(|texture| sample_nearest(texture, uv))
        };
        let Some(from) = texel(&self.from) else {
            return self.fallback.color(direction);
        };
        if self.blend <= 0.0 {
            return from;
        }
        match texel(&self.to) {
            Some(to) => mix(from, to, self.blend.min(1.0)),
            None => from,
        }
    }
}

/// The scene background: a flat color, a directional gradient or a skybox.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Solid(Color),
    Sky(SkyGradient),
    Cubemap(SkyCubemap),
}

impl Background {
    /// Background color for a ray travelling along `direction`.
    pub fn color(&self, direction: Vec3) -> Color {
        match self {
            Background::Solid(color) => *color,
            Background::Sky(sky) => sky.color(direction),
            Background::Cubemap(sky) => sky.fallback.color(direction),
        }
    }

    /// What a miss ray sees: like `color`, with the skybox's images.
    pub fn sample(&self, direction: Vec3, textures: &TextureManager) -> Color {
        match self {
            Background::Cubemap(sky) => sky.color(direction, textures),
            other => other.color(direction),
        }
    }

    /// `true` for a directional sky.
    pub fn is_sky(&self) -> bool {
        matches!(self, Background::Sky(_) | Background::Cubemap(_))
    }

    /// `true` for the six-image skybox.
    pub fn is_cubemap(&self) -> bool {
        matches!(self, Background::Cubemap(_))
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Solid(color)
    }
}
