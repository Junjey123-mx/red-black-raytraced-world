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
