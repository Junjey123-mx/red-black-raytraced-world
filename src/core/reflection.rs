// Optics primitive: lands ahead of the recursive reflected-ray tracing that
// will call it from the renderer.
#![allow(dead_code)]

use crate::core::math::Vec3;

/// Mirror-reflects the incident `direction` about the surface `normal`:
/// `R = D - 2(D.N)N`.
///
/// This is the *secondary ray* direction (incident direction pointing toward
/// the surface), distinct from the renderer's local Phong helper, which
/// reflects the surface-to-light vector for the highlight term only.
///
/// `normal` is normalized internally, so it need not be exactly unit length,
/// and the sign of the normal does not change the result (`N` and `-N`
/// describe the same mirror). The result is unit length. Degenerate input
/// (a zero/non-finite normal or direction) never produces `NaN`: a zero
/// normal leaves the direction unreflected, and a zero direction yields the
/// zero vector.
pub fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    let n = normal.normalize();
    let reflected = direction - n * (2.0 * direction.dot(n));
    let unit = reflected.normalize();

    if unit.x.is_finite() && unit.y.is_finite() && unit.z.is_finite() {
        unit
    } else {
        Vec3::zero()
    }
}
