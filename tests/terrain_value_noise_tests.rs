#[path = "."]
mod scene {
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
}

use scene::terrain::OFFICIAL_SEED;
use scene::terrain::hash::hash01;
use scene::terrain::noise::{lerp, smoothstep, value_noise_2d};

const SEED: u32 = OFFICIAL_SEED;

#[test]
fn value_noise_is_deterministic() {
    for (x, z) in [(0.0, 0.0), (3.25, -7.5), (-0.001, 0.999), (1234.5, 6789.25)] {
        let first = value_noise_2d(SEED, x, z);
        for _ in 0..3 {
            assert_eq!(value_noise_2d(SEED, x, z), first);
        }
    }
}

#[test]
fn the_smooth_interpolation_is_the_projects_own_hermite_curve() {
    assert_eq!(smoothstep(0.0), 0.0);
    assert_eq!(smoothstep(1.0), 1.0);
    assert_eq!(smoothstep(0.5), 0.5);
    let t = 0.25;
    assert!((smoothstep(t) - t * t * (3.0 - 2.0 * t)).abs() < 1e-7);
    // Monotonic and flat at both ends.
    let mut previous = 0.0;
    for i in 1..=100 {
        let v = smoothstep(i as f32 / 100.0);
        assert!(v >= previous);
        previous = v;
    }
    assert!(smoothstep(0.01) < 0.001);
    assert!(smoothstep(0.99) > 0.999);
    assert_eq!(lerp(2.0, 4.0, 0.5), 3.0);
    assert_eq!(lerp(2.0, 4.0, 0.0), 2.0);
    assert_eq!(lerp(2.0, 4.0, 1.0), 4.0);
}

#[test]
fn integer_lattice_points_return_the_corner_hash_exactly() {
    for x in -5..6 {
        for z in -5..6 {
            let expected = hash01(SEED, x, z);
            assert_eq!(value_noise_2d(SEED, x as f32, z as f32), expected);
        }
    }
}

#[test]
fn the_field_stays_in_range_and_finite_everywhere() {
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for i in -200..200 {
        for j in -200..200 {
            let v = value_noise_2d(SEED, i as f32 * 0.137, j as f32 * 0.091);
            assert!(v.is_finite());
            assert!((0.0..=1.0).contains(&v), "{v}");
            min = min.min(v);
            max = max.max(v);
        }
    }
    // A real field, not a constant.
    assert!(max - min > 0.5, "range {min}..{max}");
}

#[test]
fn a_different_seed_gives_a_different_field() {
    let mut differing = 0;
    let samples = 400;
    for i in 0..samples {
        let x = i as f32 * 0.37;
        let z = (i as f32 * 0.53).sin() * 10.0;
        if (value_noise_2d(SEED, x, z) - value_noise_2d(SEED + 1, x, z)).abs() > 1e-4 {
            differing += 1;
        }
    }
    assert!(differing > samples * 9 / 10, "{differing}/{samples}");
}

#[test]
fn the_field_is_continuous_across_cell_boundaries_and_negative_coordinates() {
    let eps = 1e-3;
    for boundary in [-3.0f32, -1.0, 0.0, 1.0, 7.0] {
        for other in [-2.5f32, -0.25, 0.0, 0.5, 3.75] {
            let left = value_noise_2d(SEED, boundary - eps, other);
            let right = value_noise_2d(SEED, boundary + eps, other);
            assert!(
                (left - right).abs() < 0.02,
                "x jump {left} -> {right} at {boundary}"
            );
            let below = value_noise_2d(SEED, other, boundary - eps);
            let above = value_noise_2d(SEED, other, boundary + eps);
            assert!(
                (below - above).abs() < 0.02,
                "z jump {below} -> {above} at {boundary}"
            );
        }
    }
}

#[test]
fn neighboring_samples_change_smoothly_without_jumps() {
    // Walk a fine line diagonally through several cells: consecutive samples
    // 1/64 apart never differ by more than a small fraction of the range.
    let step = 1.0 / 64.0;
    let mut previous = value_noise_2d(SEED, -4.0, -4.0 * 0.7);
    let mut largest = 0.0f32;
    for i in 1..=(12 * 64) {
        let t = -4.0 + i as f32 * step;
        let v = value_noise_2d(SEED, t, t * 0.7);
        largest = largest.max((v - previous).abs());
        previous = v;
    }
    // The steepest possible smoothstep slope is 1.5 per cell, times the
    // corner difference (<= 1), times the step: about 0.025 per sample along
    // one axis; the diagonal walk moves along two axes.
    assert!(largest < 0.06, "largest jump {largest}");
}

#[test]
fn the_field_is_not_a_checkerboard() {
    // Inside one cell, the value at the center is the average of the four
    // corners, never a saturated 0/1 alternation.
    for x0 in -3..3 {
        for z0 in -3..3 {
            let center = value_noise_2d(SEED, x0 as f32 + 0.5, z0 as f32 + 0.5);
            let corners = [
                hash01(SEED, x0, z0),
                hash01(SEED, x0 + 1, z0),
                hash01(SEED, x0, z0 + 1),
                hash01(SEED, x0 + 1, z0 + 1),
            ];
            let mean = corners.iter().sum::<f32>() / 4.0;
            assert!((center - mean).abs() < 1e-5, "{center} vs {mean}");
        }
    }
}
