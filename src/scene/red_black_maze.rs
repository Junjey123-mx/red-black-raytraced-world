// The inverted Red-Black half of the diamond: the structural mass under the
// portal waist (this file's first part) and, stage by stage, its -Y-facing
// surface and the Crimson, Orange and Violet families laid over it. It is
// the same `VoxelWorld` as the Overworld; the inversion is geometric and
// visual (Down-oriented blocks), never a second scene.
#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::{
    deepslate_material_id, polished_blackstone_bricks_material_id, smooth_basalt_material_id,
};
use crate::scene::rhombus::RhombusConfig;
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::hash::hash01;
use crate::scene::voxel_world::VoxelWorld;

/// Seed salt of the structural material mix.
const LOWER_MASS_SALT: u32 = 0x10E7_0001;

/// The lower mass as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LowerMass {
    pub cells: Vec<IVec3>,
    pub top_y: i32,
    pub bottom_y: i32,
}

/// Structural block of a lower-mass cell: deepslate right under the waist
/// (the geology continues through the portal), smooth basalt through the
/// widest band, polished blackstone bricks as a growing accent toward the
/// tip, mixed deterministically so the exterior reads as dark stone rather
/// than flat bands.
pub fn lower_mass_block(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    x: i32,
    y: i32,
    z: i32,
) -> (BlockType, MaterialId) {
    let depth = rhombus.shelf_y - y; // 1 at the first row under the shelf
    let span = (rhombus.shelf_y - rhombus.lower_tip_y).max(1) as f32;
    let t = depth as f32 / span; // 0 near the waist, 1 at the tip
    let roll = hash01(terrain.seed ^ LOWER_MASS_SALT, x * 7 + y, z * 11 - y);
    let deepslate_share = (1.0 - t * 2.2).clamp(0.05, 1.0);
    let bricks_share = (t - 0.35).clamp(0.0, 0.45);
    if roll < deepslate_share {
        (BlockType::Deepslate, deepslate_material_id())
    } else if roll > 1.0 - bricks_share {
        (
            BlockType::PolishedBlackstoneBricks,
            polished_blackstone_bricks_material_id(),
        )
    } else {
        (BlockType::SmoothBasalt, smooth_basalt_material_id())
    }
}

/// Fills every row from one under the shelf down to the tip with the
/// diamond's lower section (widening below the waist, then tapering to the
/// tip), skipping the cutaway quadrant, in the same world as the Overworld.
/// Every block is placed `Down`: the lower world's functional top faces -Y.
pub fn build_lower_mass(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    world: &mut VoxelWorld,
) -> LowerMass {
    let top_y = rhombus.shelf_y - 1;
    let bottom_y = rhombus.lower_tip_y;
    let mut mass = LowerMass {
        cells: Vec::new(),
        top_y,
        bottom_y,
    };
    for y in (bottom_y..=top_y).rev() {
        for z in -4..terrain.depth + 4 {
            for x in -4..terrain.width + 4 {
                if !rhombus.contains(terrain, x, y, z) || rhombus.is_cut(x, y, z) {
                    continue;
                }
                let cell = IVec3::new(x, y, z);
                if world.contains(cell) {
                    continue;
                }
                let (block_type, material) = lower_mass_block(terrain, rhombus, x, y, z);
                world.insert(
                    cell,
                    BlockInstance::new(block_type, material, Orientation::Down),
                );
                mass.cells.push(cell);
            }
        }
    }
    mass
}

// ---------------------------------------------------------------------
// The -Y-facing surface
// ---------------------------------------------------------------------

use crate::scene::overworld_blocks::{
    mycelium_material_id, red_black_deepslate_bricks_crimson_material_id,
    red_black_deepslate_bricks_orange_material_id, red_black_deepslate_bricks_violet_material_id,
};

/// The three visual families of the inverted world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    Crimson,
    Orange,
    Violet,
}

impl Family {
    pub const ALL: [Family; 3] = [Family::Crimson, Family::Orange, Family::Violet];

    /// The family's accented deepslate bricks.
    pub fn bricks(self) -> (BlockType, MaterialId) {
        match self {
            Family::Crimson => (
                BlockType::RedBlackDeepslateBricksCrimson,
                red_black_deepslate_bricks_crimson_material_id(),
            ),
            Family::Orange => (
                BlockType::RedBlackDeepslateBricksOrange,
                red_black_deepslate_bricks_orange_material_id(),
            ),
            Family::Violet => (
                BlockType::RedBlackDeepslateBricksViolet,
                red_black_deepslate_bricks_violet_material_id(),
            ),
        }
    }
}

/// Seed salt of the family-boundary jitter.
const FAMILY_SALT: u32 = 0xFA31_0001;
/// Seed salt of the surface material mix.
const SURFACE_SALT: u32 = 0x5F4C_0001;
/// Seed salt of the surface relief.
const RELIEF_SALT: u32 = 0x5F4C_0002;

/// Angular jitter of the family boundaries, in radians, so the three zones
/// interleave instead of meeting on straight lines.
pub const FAMILY_JITTER: f32 = 0.55;

/// Share of the lower columns whose surface is raised by one block.
pub const SURFACE_RELIEF_SHARE: f32 = 0.22;

/// The family a lower column belongs to: one of three angular sectors
/// around the footprint center (Crimson to the north and north-west, Orange
/// to the east, Violet to the south and south-west), with a deterministic
/// jitter of the boundary angle per column.
pub fn family_zone(terrain: &TerrainConfig, rhombus: &RhombusConfig, x: i32, z: i32) -> Family {
    let dx = (x as f32 + 0.5) - rhombus.center_x as f32;
    let dz = (z as f32 + 0.5) - rhombus.center_z as f32;
    let jitter = (hash01(terrain.seed ^ FAMILY_SALT, x, z) - 0.5) * 2.0 * FAMILY_JITTER;
    let angle = dz.atan2(dx) + jitter; // 0 = east, pi/2 = south, -pi/2 = north
    let third = std::f32::consts::PI * 2.0 / 3.0;
    let a = (angle + std::f32::consts::PI / 3.0).rem_euclid(std::f32::consts::PI * 2.0);
    if a < third {
        Family::Orange
    } else if a < 2.0 * third {
        Family::Violet
    } else {
        Family::Crimson
    }
}

/// The inverted surface as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackSurface {
    /// The lowest cell of every lower column (the ground of the inverted
    /// world), all `Down`.
    pub cells: Vec<IVec3>,
    /// Cells removed to raise part of the surface by one block.
    pub raised: Vec<IVec3>,
}

impl RedBlackSurface {
    /// The surface cell of column `(x, z)`, if the column has one.
    pub fn cell_at(&self, x: i32, z: i32) -> Option<IVec3> {
        self.cells.iter().copied().find(|c| c.x == x && c.z == z)
    }
}

/// Lowest existing cell of column `(x, z)` between `bottom` and `top`.
fn column_bottom(world: &VoxelWorld, x: i32, z: i32, bottom: i32, top: i32) -> Option<i32> {
    (bottom..=top).find(|&y| world.contains(IVec3::new(x, y, z)))
}

/// `true` for the structural blocks of the lower mass (the ones the surface
/// may replace).
fn is_structural(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::Deepslate | BlockType::SmoothBasalt | BlockType::PolishedBlackstoneBricks
    )
}

/// Base ground block of a surface column: mycelium, smooth basalt, polished
/// blackstone bricks or the column's family bricks, mixed deterministically.
pub fn surface_block(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    x: i32,
    z: i32,
) -> (BlockType, MaterialId) {
    let roll = hash01(terrain.seed ^ SURFACE_SALT, x, z);
    if roll < 0.34 {
        (BlockType::Mycelium, mycelium_material_id())
    } else if roll < 0.62 {
        (BlockType::SmoothBasalt, smooth_basalt_material_id())
    } else if roll < 0.80 {
        (
            BlockType::PolishedBlackstoneBricks,
            polished_blackstone_bricks_material_id(),
        )
    } else {
        family_zone(terrain, rhombus, x, z).bricks()
    }
}

/// Turns the underside of the lower mass into the inverted world's ground:
/// for every lower column outside the cutaway, part of the columns lose
/// their lowest cell (one block of extra relief on top of the stepped
/// section), and the lowest remaining structural cell becomes a `Down`
/// surface block. Columns whose lowest cell belongs to the inverted
/// descent keep their stair.
pub fn build_red_black_surface(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    lower: &LowerMass,
    world: &mut VoxelWorld,
) -> RedBlackSurface {
    let mut surface = RedBlackSurface::default();
    for z in -4..terrain.depth + 4 {
        for x in -4..terrain.width + 4 {
            if rhombus.cutaway.contains_column(x, z) {
                continue;
            }
            let Some(mut y) = column_bottom(world, x, z, lower.bottom_y, lower.top_y) else {
                continue;
            };
            let is_mass = |world: &VoxelWorld, y: i32| {
                world
                    .get(IVec3::new(x, y, z))
                    .map(|b| is_structural(b.block_type()))
                    .unwrap_or(false)
            };
            if !is_mass(world, y) {
                continue;
            }
            // Relief: raise this column by one where the hash says so and
            // the column keeps at least two cells of mass above.
            let tall = is_mass(world, y + 1) && is_mass(world, y + 2);
            if tall && hash01(terrain.seed ^ RELIEF_SALT, x, z) < SURFACE_RELIEF_SHARE {
                let cell = IVec3::new(x, y, z);
                world.remove(cell);
                surface.raised.push(cell);
                y += 1;
            }
            let cell = IVec3::new(x, y, z);
            let (block_type, material) = surface_block(terrain, rhombus, x, z);
            world.insert(
                cell,
                BlockInstance::new(block_type, material, Orientation::Down),
            );
            surface.cells.push(cell);
        }
    }
    surface
}
