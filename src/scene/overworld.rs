// The Overworld micro-scene: the deliberate features built on top of the
// generated terrain (pond and sandy shore first; trees, house and path
// follow). Each feature is a deterministic edit of the `VoxelWorld` driven
// by the terrain seed, never by a random-number crate.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::material_gallery::water_material_id;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::{sand_material_id, stone_block_material_id};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::generator::footprint_contains;
use crate::scene::terrain::hash::hash01;
use crate::scene::voxel_world::VoxelWorld;

/// Highest occupied cell of column `(x, z)` between `top` and `bottom`.
pub fn column_top(world: &VoxelWorld, x: i32, z: i32, top: i32, bottom: i32) -> Option<i32> {
    (bottom..=top)
        .rev()
        .find(|&y| world.contains(IVec3::new(x, y, z)))
}

fn full(block_type: BlockType, material: crate::core::material::MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Up)
}

// ---------------------------------------------------------------------
// Pond
// ---------------------------------------------------------------------

/// `y` of the water surface: one block under the lowest bank around it.
pub const POND_WATER_LEVEL: i32 = 1;

/// Deepest the basin goes below the water surface (interior cells).
pub const POND_MAX_DEPTH: i32 = 2;

/// Seed salt of the shoreline choice.
const SHORE_SALT: u32 = 0x5A4D_0001;

/// Share of the bank cells that turn to sand (the rest stay grass).
const SHORE_SAND_SHARE: f32 = 0.7;

/// Columns of the pond basin: a 5 x 3 core in the low south-western hollow
/// of the official terrain, plus a few outlying cells so the outline reads
/// as an irregular pool rather than a rectangle.
pub fn pond_basin_columns() -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    for z in 17..=19 {
        for x in 5..=9 {
            cells.push((x, z));
        }
    }
    cells.extend([(10, 17), (10, 18), (10, 19), (4, 18), (8, 20)]);
    cells
}

/// Where the pond ended up: its water cells, its sand cells and the columns
/// that were dug.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PondLayout {
    pub water: Vec<IVec3>,
    pub sand: Vec<IVec3>,
    pub basin: Vec<(i32, i32)>,
}

/// Interior basin columns (all four side neighbors also in the basin) are
/// two blocks deep; the rest one.
fn basin_depth(basin: &[(i32, i32)], x: i32, z: i32) -> i32 {
    let interior = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .all(|(dx, dz)| basin.contains(&(x + dx, z + dz)));
    if interior { POND_MAX_DEPTH } else { 1 }
}

/// Digs the pond into the terrain and dresses part of its bank with sand.
///
/// Every basin column is excavated from its surface down to a floor
/// (`POND_WATER_LEVEL - depth`), the floor becomes sand (shallow) or stone
/// (deep), and water fills the column up to `POND_WATER_LEVEL`. Bank
/// columns (eight-neighbors of the basin that are not basin themselves)
/// deterministically swap their grass for sand, most but not all of them;
/// a bank that does not rise above the water gets a sand block so the pool
/// stays contained. The mass under the floor is untouched.
pub fn carve_pond(config: &TerrainConfig, world: &mut VoxelWorld) -> PondLayout {
    let basin = pond_basin_columns();
    let mut layout = PondLayout {
        basin: basin.clone(),
        ..Default::default()
    };
    let top = config.max_surface_height();
    let bottom = config.deepslate_level - 8;

    for &(x, z) in &basin {
        debug_assert!(footprint_contains(config, x, z));
        let Some(surface) = column_top(world, x, z, top, bottom) else {
            continue;
        };
        let floor = POND_WATER_LEVEL - basin_depth(&basin, x, z);
        for y in (floor + 1)..=surface {
            world.remove(IVec3::new(x, y, z));
        }
        let floor_block = if floor >= POND_WATER_LEVEL - 1 {
            full(BlockType::Sand, sand_material_id())
        } else {
            full(BlockType::Stone, stone_block_material_id())
        };
        world.insert(IVec3::new(x, floor, z), floor_block);
        for y in (floor + 1)..=POND_WATER_LEVEL {
            let cell = IVec3::new(x, y, z);
            world.insert(cell, full(BlockType::Water, water_material_id()));
            layout.water.push(cell);
        }
    }

    // Sandy bank.
    let mut bank: Vec<(i32, i32)> = Vec::new();
    for &(x, z) in &basin {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let cell = (x + dx, z + dz);
                if (dx != 0 || dz != 0) && !basin.contains(&cell) && !bank.contains(&cell) {
                    bank.push(cell);
                }
            }
        }
    }
    for (x, z) in bank {
        if !footprint_contains(config, x, z) {
            continue;
        }
        let Some(y) = column_top(world, x, z, top, bottom) else {
            continue;
        };
        // A bank at or under the water surface would let the water spill:
        // it gets a sand block on top so the pool is always contained.
        if y <= POND_WATER_LEVEL {
            let raised = IVec3::new(x, POND_WATER_LEVEL + 1, z);
            world.insert(raised, full(BlockType::Sand, sand_material_id()));
            layout.sand.push(raised);
            continue;
        }
        if hash01(config.seed ^ SHORE_SALT, x, z) >= SHORE_SAND_SHARE {
            continue;
        }
        let cell = IVec3::new(x, y, z);
        if world.get(cell).map(|b| b.block_type()) == Some(BlockType::Grass) {
            world.insert(cell, full(BlockType::Sand, sand_material_id()));
            layout.sand.push(cell);
        }
    }

    layout
}
