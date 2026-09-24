// Material-stage sampler: lands ahead of the shading integration that will
// add the emitted color to a hit's local color.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::material::Material;
use crate::core::math::Vec2;
use crate::renderer::texture_sampling::sample_nearest;
use crate::scene::texture_manager::TextureManager;

/// Radiance a material emits at texture coordinate `uv`: the emissive mask
/// texel (same UV as the albedo, nearest-neighbor) scaled by the material's
/// `emission_strength`.
///
/// The mask is a separate texture from the albedo, so only the texels the
/// artist painted (e.g. the lit panels of a redstone lamp) emit; a black
/// texel emits nothing. A material without an emissive texture, with zero
/// strength, or whose texture id is not loaded emits black. The result is
/// not clamped: strengths above `1` are intentional (HDR-style over-bright
/// panels) and the shading stage clamps the final pixel.
pub fn sample_emissive(material: &Material, uv: Vec2, texture_manager: &TextureManager) -> Color {
    let Some(texture_id) = material.emissive_texture else {
        return Color::black();
    };
    if material.emission_strength <= 0.0 {
        return Color::black();
    }
    let Some(texture) = texture_manager.get(texture_id) else {
        return Color::black();
    };

    let texel = sample_nearest(texture, uv);
    Color::new(
        texel.r * material.emission_strength,
        texel.g * material.emission_strength,
        texel.b * material.emission_strength,
        1.0,
    )
}
