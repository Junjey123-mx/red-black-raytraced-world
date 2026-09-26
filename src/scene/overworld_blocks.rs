// Catalog-stage component: the definitive Overworld full-cube blocks. Each
// block is an ordinary `BlockType` + `MaterialId`; this module owns only the
// material profiles and the loading of their reference-derived face textures,
// so the catalog (and later the world) never restates them.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::material::{Material, MaterialId};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::{TextureLoadError, TextureManager};

pub fn dirt_material_id() -> MaterialId {
    MaterialId::new(30)
}

pub fn cobblestone_material_id() -> MaterialId {
    MaterialId::new(33)
}

/// Face textures of the definitive blocks, loaded once through the shared
/// `TextureManager` (nearest-neighbor sampling happens at render time).
pub struct OverworldBlockTextures {
    pub dirt: FaceTextures,
    pub cobblestone: FaceTextures,
}

impl OverworldBlockTextures {
    /// `overworld_dir` holds the definitive block PNGs (`dirt.png`, ...).
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let dirt = manager.load(format!("{overworld_dir}/dirt.png"))?;
        let cobblestone = manager.load(format!("{overworld_dir}/cobblestone/albedo.png"))?;

        Ok(Self {
            dirt: FaceTextures::uniform(dirt),
            cobblestone: FaceTextures::uniform(cobblestone),
        })
    }
}

/// Registers the definitive block materials (profiles from
/// `01_bloques_mundo_normal_minecraft.md`).
///
/// - Dirt: one texture on all six faces, very matte (specular 0.02).
/// - Cobblestone: one texture on all six faces, rough and matte (specular
///   0.04, shininess 5). No normal map: its albedo already carries the joints.
pub fn insert_overworld_materials(
    library: &mut MaterialLibrary,
    textures: &OverworldBlockTextures,
) {
    library.insert(
        dirt_material_id(),
        Material::new(Color::new(0.42, 0.30, 0.20, 1.0), 0.02, 4.0)
            .with_face_textures(textures.dirt),
    );
    library.insert(
        cobblestone_material_id(),
        Material::new(Color::new(0.50, 0.50, 0.50, 1.0), 0.04, 5.0)
            .with_reflectivity(0.01)
            .with_face_textures(textures.cobblestone),
    );
}
