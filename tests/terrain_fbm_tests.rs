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

use scene::terrain::fbm::{
    FbmParams, HeightField, OVERWORLD_FBM, RELIEF_CONTRAST, fbm_2d, relief_signal, terrain_height,
};
use scene::terrain::{MIN_TERRAIN_SIDE, TerrainConfig};
use std::collections::BTreeSet;

fn official() -> TerrainConfig {
    TerrainConfig::official()
}

#[test]
fn the_official_fbm_layout_is_the_documented_one() {
    assert_eq!(OVERWORLD_FBM.octaves, 4);
    assert_eq!(OVERWORLD_FBM.lacunarity, 2.0);
    assert_eq!(OVERWORLD_FBM.gain, 0.5);
    assert!(OVERWORLD_FBM.frequency > 0.0 && OVERWORLD_FBM.frequency <= 0.2);
    assert!(RELIEF_CONTRAST >= 1.0);
}

#[test]
fn fbm_and_heights_are_deterministic() {
    let config = official();
    for (x, z) in [(0, 0), (5, 17), (-3, 8), (23, 23)] {
        let a = fbm_2d(config.seed, x as f32, z as f32, OVERWORLD_FBM);
        let b = fbm_2d(config.seed, x as f32, z as f32, OVERWORLD_FBM);
        assert_eq!(a, b);
        assert_eq!(terrain_height(&config, x, z), terrain_height(&config, x, z));
    }
    assert_eq!(
        HeightField::generate(&config),
        HeightField::generate(&config)
    );
}

#[test]
fn fbm_stays_in_the_unit_range_and_is_finite() {
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for i in -100..100 {
        for j in -100..100 {
            let v = fbm_2d(
                official().seed,
                i as f32 * 0.7,
                j as f32 * 0.9,
                OVERWORLD_FBM,
            );
            assert!(v.is_finite());
            assert!((0.0..=1.0).contains(&v));
            min = min.min(v);
            max = max.max(v);
        }
    }
    assert!(max - min > 0.3, "range {min}..{max}");
    // A single octave is plain value noise; more octaves add detail.
    let one = FbmParams {
        octaves: 1,
        ..OVERWORLD_FBM
    };
    assert!(fbm_2d(1, 3.3, 4.4, one).is_finite());
}

#[test]
fn heights_are_bounded_by_the_configured_amplitude() {
    let config = official();
    for x in -10..40 {
        for z in -10..40 {
            let h = terrain_height(&config, x, z);
            assert!(h >= config.min_surface_height() && h <= config.max_surface_height());
            let s = relief_signal(&config, x, z);
            assert!((-1.0..=1.0).contains(&s));
        }
    }
}

#[test]
fn the_official_field_covers_at_least_sixteen_by_sixteen_columns() {
    let field = HeightField::generate(&official());
    assert!(field.width() >= MIN_TERRAIN_SIDE && field.depth() >= MIN_TERRAIN_SIDE);
    assert_eq!((field.width(), field.depth()), (24, 24));
    assert!(field.height(0, 0).is_some());
    assert!(field.height(23, 23).is_some());
    assert!(field.height(24, 0).is_none());
    assert!(field.height(0, -1).is_none());
}

#[test]
fn the_official_seed_has_visible_multi_block_relief() {
    let field = HeightField::generate(&official());
    let relief = field.relief();
    assert!(relief >= 2, "relief {relief}");
    // Rolling hills, not spikes: 2-4 blocks of typical variation, never more
    // than the configured amplitude on each side of the base height.
    assert!(relief <= 2 * official().height_amplitude);

    let mut distinct = BTreeSet::new();
    for z in 0..field.depth() {
        for x in 0..field.width() {
            distinct.insert(field.height(x, z).unwrap());
        }
    }
    assert!(distinct.len() >= 3, "heights {distinct:?}");
}

#[test]
fn neighboring_columns_differ_by_at_most_one_block() {
    // Natural one-block steps: no accidental cliffs anywhere in the field.
    let field = HeightField::generate(&official());
    assert_eq!(field.max_neighbor_step(), 1);
}

#[test]
fn most_of_the_surface_sits_near_the_base_height() {
    // The extremes are rare: the field is not saturated into plateaus.
    let config = official();
    let field = HeightField::generate(&config);
    let total = (field.width() * field.depth()) as f32;
    let mut extreme = 0;
    let mut middle = 0;
    for z in 0..field.depth() {
        for x in 0..field.width() {
            let h = field.height(x, z).unwrap();
            if h == config.min_surface_height() || h == config.max_surface_height() {
                extreme += 1;
            }
            if (h - config.base_height).abs() <= 1 {
                middle += 1;
            }
        }
    }
    assert!((extreme as f32) < 0.15 * total, "{extreme} extreme columns");
    assert!((middle as f32) > 0.5 * total, "{middle} middle columns");
}

#[test]
fn a_different_seed_changes_the_profile() {
    let official_field = HeightField::generate(&official());
    let other = HeightField::generate(&official().with_seed(official().seed + 1));
    let mut differing = 0;
    for z in 0..24 {
        for x in 0..24 {
            if official_field.height(x, z) != other.height(x, z) {
                differing += 1;
            }
        }
    }
    assert!(differing > 24 * 24 / 4, "{differing} columns differ");
    // Any seed keeps the same guarantees.
    assert!(other.relief() >= 2);
    assert!(other.max_neighbor_step() <= 2);
}
