#[path = "."]
mod scene {
    #[path = "."]
    pub mod terrain {
        #[path = "../src/scene/terrain/config.rs"]
        pub mod config;
        #[path = "../src/scene/terrain/fbm.rs"]
        pub mod fbm;
        #[path = "../src/scene/terrain/hash.rs"]
        pub mod hash;
        #[path = "../src/scene/terrain/noise.rs"]
        pub mod noise;

        #[allow(unused_imports)]
        pub use config::{MIN_TERRAIN_SIDE, OFFICIAL_SEED, TerrainConfig};
    }
}

use scene::terrain::hash::{HASH01_STEPS, hash_2d, hash01};
use scene::terrain::{MIN_TERRAIN_SIDE, OFFICIAL_SEED, TerrainConfig};
use std::collections::HashSet;

const SEED: u32 = OFFICIAL_SEED;

#[test]
fn the_same_seed_and_coordinates_always_hash_identically() {
    for (x, z) in [(0, 0), (1, 0), (0, 1), (-7, 13), (1_000_000, -1_000_000)] {
        let first = hash_2d(SEED, x, z);
        for _ in 0..5 {
            assert_eq!(hash_2d(SEED, x, z), first);
        }
        assert_eq!(hash01(SEED, x, z), hash01(SEED, x, z));
    }
}

#[test]
fn a_different_seed_changes_the_values() {
    let mut differing = 0;
    for x in -8..8 {
        for z in -8..8 {
            if hash_2d(SEED, x, z) != hash_2d(SEED ^ 1, x, z) {
                differing += 1;
            }
        }
    }
    // 256 lattice points: essentially all of them must change.
    assert!(differing > 250, "only {differing} of 256 changed");
}

#[test]
fn negative_coordinates_are_valid_distinct_lattice_points() {
    assert_ne!(hash_2d(SEED, -1, 0), hash_2d(SEED, 1, 0));
    assert_ne!(hash_2d(SEED, 0, -1), hash_2d(SEED, 0, 1));
    assert_ne!(hash_2d(SEED, -3, -3), hash_2d(SEED, 3, 3));
    assert!((0.0..1.0).contains(&hash01(SEED, -12, -40)));
    assert!((0.0..1.0).contains(&hash01(SEED, i32::MIN, i32::MAX)));
}

#[test]
fn hash01_stays_inside_the_unit_interval_and_is_finite() {
    for x in -40..40 {
        for z in -40..40 {
            let v = hash01(SEED, x, z);
            assert!(v.is_finite());
            assert!((0.0..1.0).contains(&v), "{v} at ({x}, {z})");
        }
    }
    // The upper bound is exclusive by construction.
    assert_eq!(
        (HASH01_STEPS - 1) as f32 / HASH01_STEPS as f32,
        1.0 - 1.0 / HASH01_STEPS as f32
    );
}

#[test]
fn neighboring_coordinates_produce_diverse_values() {
    let mut seen = HashSet::new();
    let mut sum = 0.0;
    let n = 32 * 32;
    for x in 0..32 {
        for z in 0..32 {
            seen.insert(hash_2d(SEED, x, z));
            sum += hash01(SEED, x, z);
        }
    }
    assert!(seen.len() > n - 4, "collisions: {}", n - seen.len());
    let mean = sum / n as f32;
    assert!((0.4..0.6).contains(&mean), "mean {mean}");
    // The two axes are not interchangeable.
    assert_ne!(hash_2d(SEED, 2, 5), hash_2d(SEED, 5, 2));
}

#[test]
fn the_call_order_never_matters() {
    let forward: Vec<u32> = (0..64).map(|i| hash_2d(SEED, i, -i)).collect();
    let backward: Vec<u32> = (0..64).rev().map(|i| hash_2d(SEED, i, -i)).collect();
    let mut backward = backward;
    backward.reverse();
    assert_eq!(forward, backward);
}

#[test]
fn the_official_config_meets_the_sixteen_by_sixteen_contract() {
    let config = TerrainConfig::official();
    assert_eq!(config.seed, OFFICIAL_SEED);
    assert!(config.meets_minimum_area());
    assert!(config.width >= MIN_TERRAIN_SIDE && config.depth >= MIN_TERRAIN_SIDE);
    assert_eq!((config.width, config.depth), (24, 24));
    assert_eq!(config.base_height, 4);
    assert_eq!(config.height_amplitude, 3);
    assert!((2..=3).contains(&config.dirt_depth));
    assert_eq!(config.max_surface_height() - config.min_surface_height(), 6);
    // The nominal column has stone between its soil and the deepslate.
    assert!(config.deepslate_level < config.base_height - config.dirt_depth);
    assert_eq!(TerrainConfig::default(), config);
    assert_eq!(config.with_seed(7).seed, 7);
    assert_eq!(config.with_seed(7).width, 24);

    let small = TerrainConfig {
        width: 15,
        ..config
    };
    assert!(!small.meets_minimum_area());
}
