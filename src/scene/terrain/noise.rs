// Smooth two-dimensional value noise built on the terrain hash: the lattice
// values are `hash01` at the four integer corners around the sample, blended
// with the project's own smoothstep-weighted bilinear interpolation. No
// noise crate, no tables, no state.
#![allow(dead_code)]

use crate::scene::terrain::hash::hash01;

/// Hermite smoothstep `t * t * (3 - 2t)`: flat (zero slope) at `t = 0` and
/// `t = 1`, so the blend has no kinks at the lattice lines and cells never
/// read as a checkerboard.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear blend `a * (1 - t) + b * t`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Value noise at the continuous point `(x, z)`, in `[0, 1]`.
///
/// ```text
/// floor(x, z)         -> lattice cell (x0, z0), fractions (fx, fz)
/// 4 corners           -> hash01(seed, x0..=x0+1, z0..=z0+1)
/// smoothstep(fx, fz)  -> blend weights
/// bilinear blend      -> value
/// ```
///
/// At integer coordinates the result is exactly the corner's `hash01`, so
/// the field is continuous across cell boundaries; `floor` (not `trunc`)
/// keeps the lattice consistent through negative coordinates.
pub fn value_noise_2d(seed: u32, x: f32, z: f32) -> f32 {
    let x0f = x.floor();
    let z0f = z.floor();
    let x0 = x0f as i32;
    let z0 = z0f as i32;
    let fx = smoothstep(x - x0f);
    let fz = smoothstep(z - z0f);

    let c00 = hash01(seed, x0, z0);
    let c10 = hash01(seed, x0 + 1, z0);
    let c01 = hash01(seed, x0, z0 + 1);
    let c11 = hash01(seed, x0 + 1, z0 + 1);

    let bottom = lerp(c00, c10, fx);
    let top = lerp(c01, c11, fx);
    lerp(bottom, top, fz)
}
