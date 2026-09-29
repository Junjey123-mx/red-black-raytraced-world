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
