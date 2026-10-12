// Environment profiles of the World's two realms: the bright celeste day of
// the Overworld and the dark cornelian sky of the inverted Red-Black world.
// A profile is plain data (sky colors and an ambient scale); the free-fly
// camera's portal transition blends between them, and the sky is hung
// along whichever vertical the camera currently experiences.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::Vec3;
use crate::renderer::skybox::{
    Background, CUBEMAP_FACE_NAMES, CubemapFaces, SkyCubemap, SkyGradient,
};
use crate::scene::texture_manager::{TextureLoadError, TextureManager};

/// The two realms' skyboxes: the Overworld's clouded day with its sun, and
/// the Red-Black world's starred cornelian night with its eclipsed moon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldSkyboxes {
    pub day: CubemapFaces,
    pub red_black: CubemapFaces,
}

impl WorldSkyboxes {
    /// Loads `<dir>/overworld/*.png` and `<dir>/red_black/*.png`, six faces
    /// each, as CPU textures.
    pub fn load(manager: &mut TextureManager, dir: &str) -> Result<Self, TextureLoadError> {
        let mut faces = |realm: &str| -> Result<CubemapFaces, TextureLoadError> {
            let mut ids = Vec::with_capacity(6);
            for name in CUBEMAP_FACE_NAMES {
                ids.push(manager.load(format!("{dir}/{realm}/{name}.png"))?);
            }
            Ok(CubemapFaces([
                ids[0], ids[1], ids[2], ids[3], ids[4], ids[5],
            ]))
        };
        Ok(Self {
            day: faces("overworld")?,
            red_black: faces("red_black")?,
        })
    }

    /// The skybox hung along `up`, cross-faded from one realm's cube to
    /// another's; `fallback` is the blended gradient of the same moment.
    pub fn background(
        &self,
        from: CubemapFaces,
        to: CubemapFaces,
        blend: f32,
        up: Vec3,
        fallback: SkyGradient,
    ) -> Background {
        Background::Cubemap(SkyCubemap {
            from,
            to,
            blend,
            up,
            fallback,
        })
    }
}

/// Sky colors and ambient scale of one realm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldEnvironmentProfile {
    pub sky_zenith: Color,
    pub sky_horizon: Color,
    pub sky_haze: Color,
    /// Multiplier of the World's base ambient factor.
    pub ambient_scale: f32,
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    (a * (1.0 - t) + b * t).clamp()
}

impl WorldEnvironmentProfile {
    /// The Overworld: the Gate 12/13 daytime sky, unchanged.
    pub const DAY: WorldEnvironmentProfile = WorldEnvironmentProfile {
        sky_zenith: Color {
            r: 0.33,
            g: 0.56,
            b: 0.92,
            a: 1.0,
        },
        sky_horizon: Color {
            r: 0.72,
            g: 0.85,
            b: 0.97,
            a: 1.0,
        },
        sky_haze: Color {
            r: 0.80,
            g: 0.88,
            b: 0.95,
            a: 1.0,
        },
        ambient_scale: 1.0,
    };

    /// The inverted Red-Black world: a very dark cornelian zenith, a wine
    /// horizon and a muted red-purple haze; dark and ominous, never flat
    /// red, neon magenta or pure black. Ambient is kept a touch higher so
    /// the underside stays readable under the darker sky.
    pub const RED_BLACK: WorldEnvironmentProfile = WorldEnvironmentProfile {
        sky_zenith: Color {
            r: 0.13,
            g: 0.02,
            b: 0.06,
            a: 1.0,
        },
        sky_horizon: Color {
            r: 0.34,
            g: 0.05,
            b: 0.14,
            a: 1.0,
        },
        sky_haze: Color {
            r: 0.26,
            g: 0.06,
            b: 0.15,
            a: 1.0,
        },
        ambient_scale: 1.15,
    };

    /// Linear blend of two profiles (`t = 0` gives `self`).
    pub fn blend(&self, other: &WorldEnvironmentProfile, t: f32) -> WorldEnvironmentProfile {
        let t = if t.is_finite() {
            t.clamp(0.0, 1.0)
        } else {
            0.0
        };
        WorldEnvironmentProfile {
            sky_zenith: lerp_color(self.sky_zenith, other.sky_zenith, t),
            sky_horizon: lerp_color(self.sky_horizon, other.sky_horizon, t),
            sky_haze: lerp_color(self.sky_haze, other.sky_haze, t),
            ambient_scale: self.ambient_scale * (1.0 - t) + other.ambient_scale * t,
        }
    }

    /// The profile's sky hung along `up`.
    pub fn sky(&self, up: Vec3) -> SkyGradient {
        SkyGradient {
            zenith: self.sky_zenith,
            horizon: self.sky_horizon,
            ground: self.sky_haze,
            up,
        }
    }

    /// The profile's sky as a scene background.
    pub fn background(&self, up: Vec3) -> Background {
        Background::Sky(self.sky(up))
    }
}
