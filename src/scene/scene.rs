// Diagnostic-scene stage: a tiny hand-placed terrain used to demonstrate the
// voxel DDA render path. This is deliberately NOT the final Overworld: no
// noise, no rhombus, no house, pond, portal, or Red-Black content.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::material::{Material, MaterialId};
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;
use crate::scene::voxel_world::VoxelWorld;

/// Column heights of the diagnostic terrain, indexed `[z][x]`. A column of
/// height `h` fills cells `y = 0..h`; only its top cell is grass, the cells
/// below are stone. Stepped heights give visible tops, sides, empty air
/// cells above lower columns, and voxels that shadow or hide each other.
const TERRAIN_HEIGHTS: [[i32; 4]; 4] = [[3, 2, 1, 1], [3, 2, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]];

pub fn grass_material_id() -> MaterialId {
    MaterialId::new(1)
}

pub fn stone_material_id() -> MaterialId {
    MaterialId::new(2)
}

/// Builds the small stepped diagnostic terrain (22 voxels) as a sparse
/// `VoxelWorld`.
pub fn diagnostic_voxel_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();

    for (z, row) in TERRAIN_HEIGHTS.iter().enumerate() {
        for (x, &height) in row.iter().enumerate() {
            for y in 0..height {
                let block = if y == height - 1 {
                    BlockInstance::new(BlockType::Grass, grass_material_id(), Orientation::Up)
                } else {
                    BlockInstance::new(BlockType::Stone, stone_material_id(), Orientation::Up)
                };
                world.insert(IVec3::new(x as i32, y, z as i32), block);
            }
        }
    }

    world
}

/// The minimal `MaterialId -> Material` map for the diagnostic terrain (not
/// a `MaterialLibrary`): grass uses the real Grass Block face textures with
/// the project's subtle Grass specular, and stone is a plain matte uniform
/// material, showing that uniform materials keep working next to textured
/// ones.
pub fn diagnostic_materials(grass_face_textures: FaceTextures) -> HashMap<MaterialId, Material> {
    let mut materials = HashMap::new();

    materials.insert(
        grass_material_id(),
        Material::new(Color::new(0.45, 0.55, 0.25, 1.0), 0.04, 8.0)
            .with_face_textures(grass_face_textures),
    );
    materials.insert(
        stone_material_id(),
        Material::new(Color::new(0.5, 0.5, 0.52, 1.0), 0.05, 8.0),
    );

    materials
}

pub fn wood_material_id() -> MaterialId {
    MaterialId::new(3)
}

pub fn portal_core_material_id() -> MaterialId {
    MaterialId::new(4)
}

pub fn amethyst_material_id() -> MaterialId {
    MaterialId::new(5)
}

/// Grass ground of the partial-geometry diagnostic scene: `x` in `0..7`,
/// `z` in `0..4`, at `y = 0`.
const PARTIAL_GROUND_WIDTH: i32 = 7;
const PARTIAL_GROUND_DEPTH: i32 = 4;

/// Builds the mixed partial-geometry diagnostic scene as a sparse
/// `VoxelWorld`: a small grass ground with one block of each shape in a row
/// at `y = 1`, from west to east:
///
/// ```text
/// x=0 Stone control cube      x=3 WoodDoor        (facing South)
/// x=1 WoodStairs (South)      x=4 PortalCore      (facing South)
/// x=2 Fence      (South)      x=5 AmethystCluster (growing Up)
/// ```
///
/// Two stone cubes sit right behind the fence and the amethyst cluster so
/// rays that slip through their gaps end on a visible full cube. This is a
/// diagnostic layout, not the final Overworld.
pub fn diagnostic_partial_voxel_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();

    for z in 0..PARTIAL_GROUND_DEPTH {
        for x in 0..PARTIAL_GROUND_WIDTH {
            world.insert(
                IVec3::new(x, 0, z),
                BlockInstance::new(BlockType::Grass, grass_material_id(), Orientation::Up),
            );
        }
    }

    let stone = BlockInstance::new(BlockType::Stone, stone_material_id(), Orientation::Up);
    let wood =
        |block_type, orientation| BlockInstance::new(block_type, wood_material_id(), orientation);

    world.insert(IVec3::new(0, 1, 2), stone);
    world.insert(
        IVec3::new(1, 1, 2),
        wood(BlockType::WoodStairs, Orientation::South),
    );
    world.insert(
        IVec3::new(2, 1, 2),
        wood(BlockType::Fence, Orientation::South),
    );
    world.insert(
        IVec3::new(3, 1, 2),
        wood(BlockType::WoodDoor, Orientation::South),
    );
    world.insert(
        IVec3::new(4, 1, 2),
        BlockInstance::new(
            BlockType::PortalCoreDarkCrimson,
            portal_core_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        IVec3::new(5, 1, 2),
        BlockInstance::new(
            BlockType::AmethystCluster,
            amethyst_material_id(),
            Orientation::Up,
        ),
    );

    // Full cubes right behind the fence and the cluster.
    world.insert(IVec3::new(2, 1, 1), stone);
    world.insert(IVec3::new(5, 1, 1), stone);

    world
}

/// Materials for the partial-geometry scene: the terrain materials plus
/// plain uniform, opaque, matte colors for wood, the portal core (dark
/// crimson, no emission or translucency yet), and amethyst (no optics yet).
pub fn diagnostic_partial_materials(
    grass_face_textures: FaceTextures,
) -> HashMap<MaterialId, Material> {
    let mut materials = diagnostic_materials(grass_face_textures);

    materials.insert(
        wood_material_id(),
        Material::new(Color::new(0.72, 0.55, 0.32, 1.0), 0.10, 18.0),
    );
    materials.insert(
        portal_core_material_id(),
        Material::new(Color::new(0.42, 0.05, 0.18, 1.0), 0.12, 30.0),
    );
    materials.insert(
        amethyst_material_id(),
        Material::new(Color::new(0.58, 0.36, 0.82, 1.0), 0.20, 40.0),
    );

    materials
}
