// Material-stage component: perturbs the *shading* normal from a
// tangent-space normal texture. The geometric normal that intersections,
// silhouettes, shadow-ray offsets and secondary rays rely on is never
// modified here.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::hit::Face;
use crate::core::material::Material;
use crate::core::math::{Vec2, Vec3};
use crate::renderer::texture_sampling::sample_nearest;
use crate::scene::texture_manager::TextureManager;

/// Deterministic tangent frame of an axis-aligned face, matching the UV
/// convention frozen by `Cube::intersect`:
///
/// - `tangent` (`+x` of the normal map) points toward **increasing `u`**;
/// - `bitangent` (`+y` of the normal map, "up" in the image, OpenGL style)
///   points toward **decreasing `v`**;
/// - `normal` is the face's outward normal (`+z` of the normal map).
///
/// Every frame is right-handed (`tangent x bitangent = normal`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TangentBasis {
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub normal: Vec3,
}

pub fn tangent_basis(face: Face) -> TangentBasis {
    let (tangent, bitangent) = match face {
        // uv = (lx, 1 - ly)
        Face::PositiveZ => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        // uv = (1 - lx, 1 - ly)
        Face::NegativeZ => (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        // uv = (1 - lz, 1 - ly)
        Face::PositiveX => (Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0)),
        // uv = (lz, 1 - ly)
        Face::NegativeX => (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
        // uv = (lx, lz)
        Face::PositiveY => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        // uv = (lx, 1 - lz)
        Face::NegativeY => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
    };

    TangentBasis {
        tangent,
        bitangent,
        normal: face.normal(),
    }
}

/// Decodes a normal-map texel (`rgb` in `[0, 1]`) into a unit tangent-space
/// normal: `n = 2 * rgb - 1`. The outward component is kept non-negative so
/// a perturbed normal can never point into the surface, and a degenerate
/// (zero-length) texel decodes to the flat normal `(0, 0, 1)`.
pub fn decode_normal(texel: Color) -> Vec3 {
    let n = Vec3::new(
        2.0 * texel.r - 1.0,
        2.0 * texel.g - 1.0,
        (2.0 * texel.b - 1.0).max(0.0),
    );
    let unit = n.normalize();

    if unit.x.is_finite() && unit.y.is_finite() && unit.z.is_finite() && unit.length_squared() > 0.0
    {
        unit
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    }
}

/// Rotates the tangent-space `tangent_normal` into world space using the
/// face's frame around the (possibly view-flipped) geometric `normal`:
/// `shading = normalize(T*x + B*y + N*z)`. Always finite and unit length; a
/// flat tangent normal returns `normal` itself.
pub fn perturb_normal(normal: Vec3, face: Face, tangent_normal: Vec3) -> Vec3 {
    let basis = tangent_basis(face);
    let perturbed = (basis.tangent * tangent_normal.x
        + basis.bitangent * tangent_normal.y
        + normal * tangent_normal.z)
        .normalize();

    if perturbed.x.is_finite()
        && perturbed.y.is_finite()
        && perturbed.z.is_finite()
        && perturbed.length_squared() > 0.0
    {
        perturbed
    } else {
        normal
    }
}

/// The normal used for diffuse and specular shading at a hit. A material
/// without a normal texture (or whose texture id is not loaded) shades with
/// the geometric `normal` unchanged; otherwise the normal texture is sampled
/// with the same UV and nearest-neighbor filter as the albedo and the result
/// perturbs `normal` through the face's tangent frame.
pub fn sample_shading_normal(
    material: &Material,
    face: Face,
    uv: Vec2,
    normal: Vec3,
    texture_manager: &TextureManager,
) -> Vec3 {
    let Some(texture_id) = material.normal_texture else {
        return normal;
    };
    let Some(texture) = texture_manager.get(texture_id) else {
        return normal;
    };

    let tangent_normal = decode_normal(sample_nearest(texture, uv));
    perturb_normal(normal, face, tangent_normal)
}
