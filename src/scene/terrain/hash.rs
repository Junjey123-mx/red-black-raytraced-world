// Deterministic coordinate hashing: the root of the procedural pipeline.
// Integer mixing only (no floating-point state, no allocation, no global
// state), so the same `(seed, x, z)` always yields the same value on every
// run and every platform.
#![allow(dead_code)]

/// Avalanche mixer (the `fmix32` finalizer of MurmurHash3): every input bit
/// influences every output bit, so neighboring coordinates produce
/// unrelated values.
fn mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

/// 32-bit hash of a lattice coordinate. Negative coordinates are valid:
/// `x` and `z` are reinterpreted as two's-complement `u32`, so `-1` and
/// `u32::MAX` are the same lattice point and no coordinate is folded onto
/// another. The two axes are mixed in separately so `(x, z)` and `(z, x)`
/// differ.
pub fn hash_2d(seed: u32, x: i32, z: i32) -> u32 {
    let h = mix(seed ^ 0x9E37_79B9);
    let h = mix(h ^ (x as u32).wrapping_mul(0x27D4_EB2F));
    mix(h ^ (z as u32).wrapping_mul(0x1656_67B1))
}

/// Number of distinct values `hash01` can return (the hash's top 24 bits).
pub const HASH01_STEPS: u32 = 1 << 24;

/// `hash_2d` scaled into `[0, 1)`: the 24 most significant bits, which an
/// `f32` represents exactly, divided by `2^24`.
pub fn hash01(seed: u32, x: i32, z: i32) -> f32 {
    (hash_2d(seed, x, z) >> 8) as f32 / HASH01_STEPS as f32
}
