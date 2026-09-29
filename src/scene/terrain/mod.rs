// Overworld terrain stage: the project-owned procedural pipeline
// (seed -> hash -> value noise -> fBM -> height -> voxels) that shapes the
// upper world. No noise or random-number crate is involved anywhere: every
// value is a pure function of the seed and the integer/float coordinates.
#![allow(dead_code)]

pub mod hash;

/// Seed of the definitive Overworld. Every generated feature (relief and,
/// later, strata, pond, trees, house and path) derives from it, so the same
/// seed always rebuilds the identical world.
pub const OFFICIAL_SEED: u32 = 0x5EED_2024;

/// Smallest footprint side the architecture accepts for the upper world.
pub const MIN_TERRAIN_SIDE: i32 = 16;

/// Everything the terrain generator needs to know about the Overworld's
/// footprint and vertical profile. Plain data: it computes nothing itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainConfig {
    /// Root of every deterministic choice.
    pub seed: u32,
    /// Footprint size along `x` (columns), in blocks.
    pub width: i32,
    /// Footprint size along `z` (rows), in blocks.
    pub depth: i32,
    /// Surface height of a column where the relief signal is neutral.
    pub base_height: i32,
    /// Maximum rise or fall of the surface above/below `base_height`.
    pub height_amplitude: i32,
    /// Nominal thickness of the shallow soil under the grass.
    pub dirt_depth: i32,
    /// Highest `y` that is deepslate: every cell at or below it is deep.
    pub deepslate_level: i32,
}

impl TerrainConfig {
    /// The definitive Overworld configuration: a 24 x 24 footprint whose
    /// surface rolls a few blocks around `y = 4`.
    pub const OFFICIAL: TerrainConfig = TerrainConfig {
        seed: OFFICIAL_SEED,
        width: 24,
        depth: 24,
        base_height: 4,
        height_amplitude: 3,
        dirt_depth: 3,
        deepslate_level: -1,
    };

    pub fn official() -> Self {
        Self::OFFICIAL
    }

    /// Same configuration with another seed (for comparisons and tests).
    pub fn with_seed(self, seed: u32) -> Self {
        Self { seed, ..self }
    }

    /// `true` when the footprint is at least `MIN_TERRAIN_SIDE` on each side.
    pub fn meets_minimum_area(&self) -> bool {
        self.width >= MIN_TERRAIN_SIDE && self.depth >= MIN_TERRAIN_SIDE
    }

    /// Highest surface the relief can reach.
    pub fn max_surface_height(&self) -> i32 {
        self.base_height + self.height_amplitude
    }

    /// Lowest surface the relief can reach.
    pub fn min_surface_height(&self) -> i32 {
        self.base_height - self.height_amplitude
    }
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self::OFFICIAL
    }
}
