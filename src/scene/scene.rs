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
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
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

pub fn door_bottom_material_id() -> MaterialId {
    MaterialId::new(6)
}

pub fn door_top_material_id() -> MaterialId {
    MaterialId::new(7)
}

/// Grass ground of the partial-geometry gallery: `x` in `0..10`, `z` in
/// `0..7`, at `y = 0`.
const GALLERY_WIDTH: i32 = 9;
const GALLERY_DEPTH: i32 = 6;

/// Builds the partial-geometry diagnostic gallery as a sparse `VoxelWorld`:
/// a plain grass platform with one specimen per shape, spaced apart so none
/// hides another. Two rows at `y = 1`, all facing South (toward the camera):
///
/// ```text
/// back  (z=1): WoodDoor x=1 (two cells tall)  PortalCore x=4  AmethystCluster x=7
/// front (z=4): Stone control cube x=1         WoodStairs x=4  Fence x=7
/// ```
///
/// This is a diagnostic layout, not the final Overworld.
pub fn diagnostic_partial_voxel_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();

    for z in 0..GALLERY_DEPTH {
        for x in 0..GALLERY_WIDTH {
            world.insert(
                IVec3::new(x, 0, z),
                BlockInstance::new(BlockType::Grass, grass_material_id(), Orientation::Up),
            );
        }
    }

    let block = |block_type, material_id, orientation| {
        BlockInstance::new(block_type, material_id, orientation)
    };

    // Front row.
    world.insert(
        IVec3::new(1, 1, 4),
        block(BlockType::Stone, stone_material_id(), Orientation::Up),
    );
    world.insert(
        IVec3::new(4, 1, 4),
        block(
            BlockType::WoodStairs,
            wood_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        IVec3::new(7, 1, 4),
        block(BlockType::Fence, wood_material_id(), Orientation::South),
    );

    // Back row: a two-cell-tall door, the portal membrane, the cluster.
    world.insert(
        IVec3::new(1, 1, 1),
        block(
            BlockType::WoodDoor,
            door_bottom_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        IVec3::new(1, 2, 1),
        block(
            BlockType::WoodDoor,
            door_top_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        IVec3::new(4, 1, 1),
        block(
            BlockType::PortalCoreDarkCrimson,
            portal_core_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        IVec3::new(7, 1, 1),
        block(
            BlockType::AmethystCluster,
            amethyst_material_id(),
            Orientation::Up,
        ),
    );

    world
}

/// Face textures of the gallery specimens. Small pixel-art diagnostics only,
/// loaded once through `TextureManager` (nearest-neighbor sampling).
pub struct PartialSceneTextures {
    pub grass: FaceTextures,
    pub stone: FaceTextures,
    pub planks: FaceTextures,
    pub door_bottom: FaceTextures,
    pub door_top: FaceTextures,
    pub portal_core: FaceTextures,
}

impl PartialSceneTextures {
    /// Loads the diagnostic PNGs. `grass_dir` holds `top/side/bottom.png`;
    /// `partial_dir` holds the gallery textures.
    pub fn load(
        manager: &mut TextureManager,
        grass_dir: &str,
        partial_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let mut load = |dir: &str, name: &str| manager.load(format!("{dir}/{name}.png"));

        let grass_top = load(grass_dir, "top")?;
        let grass_side = load(grass_dir, "side")?;
        let grass_bottom = load(grass_dir, "bottom")?;
        let stone = load(partial_dir, "stone")?;
        let planks = load(partial_dir, "planks")?;
        let door_edge = load(partial_dir, "door_edge")?;
        let door_bottom = load(partial_dir, "door_bottom")?;
        let door_top = load(partial_dir, "door_top")?;
        let portal_core = load(partial_dir, "portal_core")?;
        let portal_edge = load(partial_dir, "portal_edge")?;

        // Door and portal are thin: broad faces (+/-Z) carry the picture,
        // the narrow sides use a plain edge texture.
        let thin = |broad, edge| FaceTextures::new(edge, edge, edge, edge, broad, broad);

        Ok(Self {
            grass: FaceTextures::new(
                grass_side,
                grass_side,
                grass_top,
                grass_bottom,
                grass_side,
                grass_side,
            ),
            stone: FaceTextures::uniform(stone),
            planks: FaceTextures::uniform(planks),
            door_bottom: thin(door_bottom, door_edge),
            door_top: thin(door_top, door_edge),
            portal_core: thin(portal_core, portal_edge),
        })
    }
}

/// Materials for the gallery: terrain plus textured wood, door halves, and
/// the dark-crimson portal core (no emission or translucency yet), and a
/// uniform matte violet for amethyst (no optics yet).
pub fn diagnostic_partial_materials(
    textures: &PartialSceneTextures,
) -> HashMap<MaterialId, Material> {
    let mut materials = diagnostic_materials(textures.grass);

    materials.insert(
        stone_material_id(),
        Material::new(Color::new(0.5, 0.5, 0.52, 1.0), 0.05, 8.0)
            .with_face_textures(textures.stone),
    );
    materials.insert(
        wood_material_id(),
        Material::new(Color::new(0.72, 0.55, 0.32, 1.0), 0.10, 18.0)
            .with_face_textures(textures.planks),
    );
    materials.insert(
        door_bottom_material_id(),
        Material::new(Color::new(0.72, 0.55, 0.32, 1.0), 0.10, 18.0)
            .with_face_textures(textures.door_bottom),
    );
    materials.insert(
        door_top_material_id(),
        Material::new(Color::new(0.72, 0.55, 0.32, 1.0), 0.10, 18.0)
            .with_face_textures(textures.door_top),
    );
    materials.insert(
        portal_core_material_id(),
        Material::new(Color::new(0.42, 0.05, 0.18, 1.0), 0.12, 30.0)
            .with_face_textures(textures.portal_core),
    );
    materials.insert(
        amethyst_material_id(),
        Material::new(Color::new(0.58, 0.36, 0.82, 1.0), 0.20, 40.0),
    );

    materials
}
