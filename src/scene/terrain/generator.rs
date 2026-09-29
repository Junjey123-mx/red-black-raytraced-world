// Turns the fBM height field into real voxel columns: the bounded, softly
// edged Overworld mass that `WorldScene` presents. Every column is solid
// from its bottom to its grass top and stratified like Minecraft ground:
// grass, a shallow band of dirt, stone, and deepslate at depth.
#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::{
    deepslate_material_id, dirt_material_id, stone_block_material_id,
};
use crate::scene::scene::grass_material_id;
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::fbm::HeightField;
use crate::scene::terrain::hash::hash01;
use crate::scene::voxel_world::VoxelWorld;

/// Exponent of the footprint's superellipse: `1` would be a pure diamond,
/// a large value a square. `1.5` keeps a compact rounded-diamond island with
/// the corners of the bounding square cut away.
pub const FOOTPRINT_EXPONENT: f32 = 1.5;

/// Normalized footprint radius beyond which the surface dips one block, so
/// the rim of the island sits slightly lower than its interior.
pub const RIM_RADIUS: f32 = 0.78;

/// Normalized radii beyond which the underside rises by one and two blocks,
/// tapering the mass toward its edges instead of ending in a flat slab.
pub const TAPER_RADII: [f32; 2] = [0.55, 0.85];

/// How many blocks below `deepslate_level` the mass extends at its center.
pub const DEPTH_BELOW_DEEPSLATE_LEVEL: i32 = 2;

/// Distance of column `(x, z)` from the footprint center in the
/// superellipse metric: `0` at the center, `1` on the footprint boundary,
/// larger outside.
pub fn footprint_radius(config: &TerrainConfig, x: i32, z: i32) -> f32 {
    let half_width = config.width as f32 / 2.0;
    let half_depth = config.depth as f32 / 2.0;
    let dx = ((x as f32 + 0.5) - half_width).abs() / half_width;
    let dz = ((z as f32 + 0.5) - half_depth).abs() / half_depth;
    dx.powf(FOOTPRINT_EXPONENT) + dz.powf(FOOTPRINT_EXPONENT)
}

/// `true` when column `(x, z)` belongs to the island.
pub fn footprint_contains(config: &TerrainConfig, x: i32, z: i32) -> bool {
    x >= 0
        && z >= 0
        && x < config.width
        && z < config.depth
        && footprint_radius(config, x, z) <= 1.0
}

/// Lowest cell of the mass at the center of the footprint.
pub fn mass_bottom(config: &TerrainConfig) -> i32 {
    config.deepslate_level - DEPTH_BELOW_DEEPSLATE_LEVEL
}

/// Lowest occupied cell of column `(x, z)`: the mass bottom, raised toward
/// the edges of the footprint.
pub fn column_bottom(config: &TerrainConfig, x: i32, z: i32) -> i32 {
    let radius = footprint_radius(config, x, z);
    let taper = TAPER_RADII.iter().filter(|&&r| radius > r).count() as i32;
    mass_bottom(config) + taper
}

/// Surface cell (the grass block) of column `(x, z)`: the height field's
/// value, lowered by one on the rim of the island and never below the
/// configured minimum.
pub fn column_surface(config: &TerrainConfig, field: &HeightField, x: i32, z: i32) -> Option<i32> {
    let height = field.height(x, z)?;
    let rim = if footprint_radius(config, x, z) > RIM_RADIUS {
        1
    } else {
        0
    };
    Some((height - rim).max(config.min_surface_height()))
}

/// Thinnest soil band under the grass; `config.dirt_depth` is the thickest.
pub const MIN_DIRT_DEPTH: i32 = 2;

/// Seed salt of the soil-thickness choice, so it is independent of the
/// relief noise of the same column.
const DIRT_DEPTH_SALT: u32 = 0x0D12_7000;

/// Thickness of the dirt band under the grass of column `(x, z)`: varies
/// deterministically between `MIN_DIRT_DEPTH` and `config.dirt_depth`.
pub fn dirt_depth(config: &TerrainConfig, x: i32, z: i32) -> i32 {
    let max = config.dirt_depth.max(MIN_DIRT_DEPTH);
    let span = (max - MIN_DIRT_DEPTH + 1) as f32;
    MIN_DIRT_DEPTH + (hash01(config.seed ^ DIRT_DEPTH_SALT, x, z) * span) as i32
}

/// Geological stratum of cell `y` in column `(x, z)` whose grass sits at
/// `surface`:
///
/// ```text
/// y == surface                      -> Grass
/// surface - dirt_depth <= y < surface -> Dirt
/// y <= deepslate_level              -> Deepslate
/// otherwise                         -> Stone
/// ```
///
/// The soil band always follows the grass, so a low column may run out of
/// stone before reaching the deepslate; deepslate never sits above dirt.
pub fn stratum(config: &TerrainConfig, x: i32, z: i32, y: i32, surface: i32) -> BlockType {
    if y == surface {
        BlockType::Grass
    } else if y >= surface - dirt_depth(config, x, z) {
        BlockType::Dirt
    } else if y <= config.deepslate_level {
        BlockType::Deepslate
    } else {
        BlockType::Stone
    }
}

/// The material of a terrain stratum block.
pub fn stratum_material(block_type: BlockType) -> MaterialId {
    match block_type {
        BlockType::Grass => grass_material_id(),
        BlockType::Dirt => dirt_material_id(),
        BlockType::Deepslate => deepslate_material_id(),
        _ => stone_block_material_id(),
    }
}

/// The block filling cell `y` of column `(x, z)` whose grass sits at
/// `surface`: its stratum and that stratum's material.
pub fn column_block(
    config: &TerrainConfig,
    x: i32,
    z: i32,
    y: i32,
    surface: i32,
) -> (BlockType, MaterialId) {
    let block_type = stratum(config, x, z, y, surface);
    (block_type, stratum_material(block_type))
}

/// Bounds of the generated mass, for tests and for framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainBounds {
    pub min: IVec3,
    pub max: IVec3,
}

/// Fills `world` with the terrain columns of `config` and returns the height
/// field they were built from. Cells outside the footprint stay empty; every
/// column inside is solid from `column_bottom` to `column_surface`.
pub fn generate_terrain(config: &TerrainConfig, world: &mut VoxelWorld) -> HeightField {
    let field = HeightField::generate(config);

    for z in 0..config.depth {
        for x in 0..config.width {
            if !footprint_contains(config, x, z) {
                continue;
            }
            let Some(surface) = column_surface(config, &field, x, z) else {
                continue;
            };
            let bottom = column_bottom(config, x, z);
            for y in bottom..=surface {
                let (block_type, material) = column_block(config, x, z, y, surface);
                world.insert(
                    IVec3::new(x, y, z),
                    BlockInstance::new(block_type, material, Orientation::Up),
                );
            }
        }
    }

    field
}

/// Axis-aligned bounds of the terrain mass described by `config`.
pub fn terrain_bounds(config: &TerrainConfig, field: &HeightField) -> TerrainBounds {
    let mut min = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
    let mut max = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
    for z in 0..config.depth {
        for x in 0..config.width {
            if !footprint_contains(config, x, z) {
                continue;
            }
            let Some(surface) = column_surface(config, field, x, z) else {
                continue;
            };
            let bottom = column_bottom(config, x, z);
            min = IVec3::new(min.x.min(x), min.y.min(bottom), min.z.min(z));
            max = IVec3::new(max.x.max(x), max.y.max(surface), max.z.max(z));
        }
    }
    TerrainBounds { min, max }
}
