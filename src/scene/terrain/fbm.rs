// Fractional Brownian motion over the value noise: a few octaves of the same
// field at doubling frequency and halving amplitude, normalized back into
// `[0, 1]`, and the integer surface height each terrain column takes from it.
#![allow(dead_code)]

use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::noise::value_noise_2d;

/// Octave layout of the Overworld relief.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FbmParams {
    /// Number of noise layers summed.
    pub octaves: u32,
    /// Lattice cells per block of the first octave: low, so one hill spans
    /// several blocks instead of every column having its own bump.
    pub frequency: f32,
    /// Frequency multiplier from one octave to the next.
    pub lacunarity: f32,
    /// Amplitude multiplier from one octave to the next.
    pub gain: f32,
}

/// The definitive relief layout: four octaves, a base wavelength of about
/// ten blocks, standard `2.0 / 0.5` octave scaling.
pub const OVERWORLD_FBM: FbmParams = FbmParams {
    octaves: 4,
    frequency: 0.1,
    lacunarity: 2.0,
    gain: 0.5,
};

/// Contrast applied to the centered fBM signal before it is scaled to the
/// height amplitude. Summed octaves crowd around the middle of the range, so
/// without it the surface would barely leave the base height; with it the
/// rolling hills reach the configured amplitude on the official seed.
pub const RELIEF_CONTRAST: f32 = 1.6;

/// fBM at `(x, z)`: `octaves` layers of `value_noise_2d`, each offset in the
/// lattice (so the octaves do not share corners) and weighted by `gain^i`,
/// with the total divided by the summed weights so the result stays in
/// `[0, 1]`. Deterministic for a given `seed`.
pub fn fbm_2d(seed: u32, x: f32, z: f32, params: FbmParams) -> f32 {
    let mut total = 0.0;
    let mut weight_sum = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = params.frequency;

    for octave in 0..params.octaves.max(1) {
        // Each octave hashes with its own seed and sits on a shifted lattice.
        let octave_seed = seed.wrapping_add(octave.wrapping_mul(0x9E37_79B9));
        let shift = octave as f32 * 17.31;
        total +=
            amplitude * value_noise_2d(octave_seed, x * frequency + shift, z * frequency - shift);
        weight_sum += amplitude;
        amplitude *= params.gain;
        frequency *= params.lacunarity;
    }

    (total / weight_sum).clamp(0.0, 1.0)
}

/// The centered relief signal in `[-1, 1]` for a column: the official fBM
/// pushed away from the middle by `RELIEF_CONTRAST` and clamped.
pub fn relief_signal(config: &TerrainConfig, x: i32, z: i32) -> f32 {
    let noise = fbm_2d(config.seed, x as f32 + 0.5, z as f32 + 0.5, OVERWORLD_FBM);
    ((noise - 0.5) * 2.0 * RELIEF_CONTRAST).clamp(-1.0, 1.0)
}

/// Integer surface height (the `y` of the grass block) of the column at
/// `(x, z)`: `base_height` plus the relief signal scaled to
/// `height_amplitude`, so it always lies in
/// `[min_surface_height, max_surface_height]`.
pub fn terrain_height(config: &TerrainConfig, x: i32, z: i32) -> i32 {
    let offset = (relief_signal(config, x, z) * config.height_amplitude as f32).round() as i32;
    (config.base_height + offset).clamp(config.min_surface_height(), config.max_surface_height())
}

/// Surface heights of the whole footprint, indexed `[z][x]`, plus their
/// range: the concrete height field the generator turns into columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightField {
    width: i32,
    depth: i32,
    heights: Vec<i32>,
}

impl HeightField {
    /// Samples `terrain_height` over `config.width x config.depth` columns
    /// with the footprint's origin at `(0, 0)`.
    pub fn generate(config: &TerrainConfig) -> Self {
        let mut heights = Vec::with_capacity((config.width * config.depth).max(0) as usize);
        for z in 0..config.depth {
            for x in 0..config.width {
                heights.push(terrain_height(config, x, z));
            }
        }
        Self {
            width: config.width,
            depth: config.depth,
            heights,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }

    /// Height of column `(x, z)`, or `None` outside the footprint.
    pub fn height(&self, x: i32, z: i32) -> Option<i32> {
        if x < 0 || z < 0 || x >= self.width || z >= self.depth {
            return None;
        }
        Some(self.heights[(z * self.width + x) as usize])
    }

    pub fn min_height(&self) -> i32 {
        self.heights.iter().copied().min().unwrap_or(0)
    }

    pub fn max_height(&self) -> i32 {
        self.heights.iter().copied().max().unwrap_or(0)
    }

    /// `max_height - min_height`: the visible relief of the field.
    pub fn relief(&self) -> i32 {
        self.max_height() - self.min_height()
    }

    /// Largest height difference between two side-by-side columns.
    pub fn max_neighbor_step(&self) -> i32 {
        let mut largest = 0;
        for z in 0..self.depth {
            for x in 0..self.width {
                let h = self.height(x, z).unwrap();
                for (dx, dz) in [(1, 0), (0, 1)] {
                    if let Some(n) = self.height(x + dx, z + dz) {
                        largest = largest.max((h - n).abs());
                    }
                }
            }
        }
        largest
    }
}
