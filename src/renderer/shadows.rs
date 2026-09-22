// Foundation-stage primitive: lands ahead of the occlusion-integration work
// that will call these shadow-ray builders per light per hit.
#![allow(dead_code)]

use crate::core::cube::Cube;
use crate::core::math::Vec3;
use crate::core::ray::Ray;
use crate::scene::light::{DirectionalLight, PointLight};

/// Centralized offset applied to shadow-ray origins along the surface
/// normal. Avoids immediate floating-point self-intersection against the
/// originating surface ("shadow acne") while staying small enough not to
/// visibly detach real shadows from their casters. Every shadow-ray origin
/// in the renderer must go through this single constant rather than
/// repeating ad hoc magic numbers.
pub const SHADOW_EPSILON: f32 = 1e-4;

/// Large-but-finite reach used as the effective "infinite" range of a
/// directional light's shadow ray.
pub const DIRECTIONAL_SHADOW_RANGE: f32 = 1_000_000.0;

/// Builds the epsilon-offset shadow ray from a surface hit toward a
/// directional light. The origin is displaced along `normal` by
/// `SHADOW_EPSILON` so the ray does not immediately re-intersect the
/// surface it originated from. Callers should test occlusion within
/// `[0, DIRECTIONAL_SHADOW_RANGE]`.
pub fn shadow_ray_to_directional_light(
    hit_point: Vec3,
    normal: Vec3,
    light: &DirectionalLight,
) -> Ray {
    let origin = hit_point + normal * SHADOW_EPSILON;
    Ray::new(origin, light.direction)
}

/// Builds the epsilon-offset shadow ray from a surface hit toward a point
/// light, plus the maximum valid occlusion distance (the distance from the
/// offset origin to the light itself). A blocker beyond this distance lies
/// behind the light and must not count as an occluder. When the light is
/// (near-)coincident with the offset origin, the direction is undefined;
/// `normal` is used as a safe fallback and the returned distance is `0.0`,
/// which excludes any occluder from the valid range.
pub fn shadow_ray_to_point_light(hit_point: Vec3, normal: Vec3, light: &PointLight) -> (Ray, f32) {
    let origin = hit_point + normal * SHADOW_EPSILON;
    let to_light = light.position - origin;
    let distance = to_light.length();

    if distance <= f32::EPSILON {
        return (Ray::new(origin, normal), 0.0);
    }

    (Ray::new(origin, to_light), distance)
}

/// Binary hard-shadow occlusion test: `true` if any cube in `objects`
/// blocks `ray` within `(0, t_max)`. Geometry only — materials do not
/// affect occlusion, matching the hard-shadow contract (no transparency).
pub fn is_occluded<'a>(objects: impl IntoIterator<Item = &'a Cube>, ray: &Ray, t_max: f32) -> bool {
    objects
        .into_iter()
        .any(|cube| cube.intersect(ray, 0.0, t_max).is_some())
}
