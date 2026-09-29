// Overworld terrain stage: the project-owned procedural pipeline
// (seed -> hash -> value noise -> fBM -> height -> voxels) that shapes the
// upper world. No noise or random-number crate is involved anywhere: every
// value is a pure function of the seed and the integer/float coordinates.
#![allow(dead_code)]

pub mod config;
pub mod fbm;
pub mod generator;
pub mod hash;
pub mod noise;

#[allow(unused_imports)]
pub use config::{MIN_TERRAIN_SIDE, OFFICIAL_SEED, TerrainConfig};
