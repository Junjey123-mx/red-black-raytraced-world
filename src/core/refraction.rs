// Optics primitive: lands ahead of the transparent-material rendering that
// will call it to transmit rays across media.
#![allow(dead_code)]

use crate::core::math::Vec3;

/// Snell's-law refraction of the incident unit `direction` across a surface
/// with the given `normal`, going from a medium of index `eta_i` into one of
/// index `eta_t` (`eta_i * sin(theta_i) = eta_t * sin(theta_t)`).
///
/// `normal` is normalized internally and may point either way: it is flipped
/// to oppose `direction`, so the same call serves both entering a medium
/// (air -> glass) and exiting it (glass -> air) — the caller only swaps
/// `eta_i`/`eta_t`. The returned direction is unit length and points to the
/// far side of the surface.
///
/// Returns `None` when there is no transmitted ray:
/// - **total internal reflection**, when `eta_i > eta_t` and the incidence is
///   beyond the critical angle (the caller should reflect instead);
/// - an invalid medium (`eta_i`/`eta_t` non-finite or `<= 0`), or a
///   degenerate zero/non-finite `direction` or `normal`.
///
/// The result is never `NaN`.
pub fn refract(direction: Vec3, normal: Vec3, eta_i: f32, eta_t: f32) -> Option<Vec3> {
    let valid_index = |eta: f32| eta.is_finite() && eta > 0.0;
    if !valid_index(eta_i) || !valid_index(eta_t) {
        return None;
    }

    let d = direction.normalize();
    let mut n = normal.normalize();
    if d.length_squared() == 0.0 || n.length_squared() == 0.0 {
        return None;
    }

    let mut cos_i = -d.dot(n);
    if cos_i < 0.0 {
        n = -n;
        cos_i = -cos_i;
    }

    let eta = eta_i / eta_t;
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        return None;
    }

    let transmitted = (d * eta + n * (eta * cos_i - k.sqrt())).normalize();
    if transmitted.x.is_finite() && transmitted.y.is_finite() && transmitted.z.is_finite() {
        Some(transmitted)
    } else {
        None
    }
}
