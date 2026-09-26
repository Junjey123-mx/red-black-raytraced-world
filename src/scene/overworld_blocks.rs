// Catalog-stage component: the definitive Overworld full-cube blocks. Each
// block is an ordinary `BlockType` + `MaterialId`; this module owns only the
// material profiles and the loading of their reference-derived face textures,
// so the catalog (and later the world) never restates them.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::material::{AlphaMode, Material, MaterialId};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::{TextureLoadError, TextureManager};

pub fn dirt_material_id() -> MaterialId {
    MaterialId::new(30)
}

pub fn stone_block_material_id() -> MaterialId {
    MaterialId::new(32)
}

pub fn log_material_id() -> MaterialId {
    MaterialId::new(36)
}

pub fn wood_planks_material_id() -> MaterialId {
    MaterialId::new(37)
}

pub fn double_wood_slab_material_id() -> MaterialId {
    MaterialId::new(38)
}

pub fn wood_stairs_material_id() -> MaterialId {
    MaterialId::new(39)
}

pub fn fence_material_id() -> MaterialId {
    MaterialId::new(40)
}

pub fn wood_door_bottom_material_id() -> MaterialId {
    MaterialId::new(41)
}

pub fn wood_door_top_material_id() -> MaterialId {
    MaterialId::new(42)
}

pub fn portal_frame_material_id() -> MaterialId {
    MaterialId::new(43)
}

pub fn cobblestone_material_id() -> MaterialId {
    MaterialId::new(33)
}

pub fn sand_material_id() -> MaterialId {
    MaterialId::new(34)
}

pub fn deepslate_material_id() -> MaterialId {
    MaterialId::new(35)
}

/// Face textures of the definitive blocks, loaded once through the shared
/// `TextureManager` (nearest-neighbor sampling happens at render time).
pub struct OverworldBlockTextures {
    pub dirt: FaceTextures,
    pub stone: FaceTextures,
    pub portal_frame: FaceTextures,
    /// Lower and upper halves of the two-cell-tall door (broad faces on
    /// +/-Z, narrow dark-wood edges elsewhere).
    pub wood_door_bottom: FaceTextures,
    pub wood_door_top: FaceTextures,
    pub wood_planks: FaceTextures,
    pub log: FaceTextures,
    pub cobblestone: FaceTextures,
    pub sand: FaceTextures,
    pub deepslate: FaceTextures,
}

impl OverworldBlockTextures {
    /// `overworld_dir` holds the definitive block PNGs (`dirt.png`, ...).
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let dirt = manager.load(format!("{overworld_dir}/dirt.png"))?;
        let stone = manager.load(format!("{overworld_dir}/stone.png"))?;
        let log_end = manager.load(format!("{overworld_dir}/log/end.png"))?;
        let log_side = manager.load(format!("{overworld_dir}/log/side.png"))?;
        let wood_planks = manager.load(format!("{overworld_dir}/wood_planks.png"))?;
        let door_bottom = manager.load(format!("{overworld_dir}/wood_door/bottom.png"))?;
        let door_top = manager.load(format!("{overworld_dir}/wood_door/top.png"))?;
        let edge_bottom = manager.load(format!("{overworld_dir}/wood_door/edge_bottom.png"))?;
        let edge_top = manager.load(format!("{overworld_dir}/wood_door/edge_top.png"))?;
        let thin = |broad, edge| FaceTextures::new(edge, edge, edge, edge, broad, broad);
        let portal_frame = manager.load(format!("{PORTAL_TEXTURES_DIR}/frame_albedo.png"))?;
        let cobblestone = manager.load(format!("{overworld_dir}/cobblestone/albedo.png"))?;
        let sand = manager.load(format!("{overworld_dir}/sand.png"))?;
        let deepslate_top = manager.load(format!("{overworld_dir}/deepslate/top.png"))?;
        let deepslate_side = manager.load(format!("{overworld_dir}/deepslate/side.png"))?;

        Ok(Self {
            dirt: FaceTextures::uniform(dirt),
            stone: FaceTextures::uniform(stone),
            portal_frame: FaceTextures::uniform(portal_frame),
            wood_door_bottom: thin(door_bottom, edge_bottom),
            wood_door_top: thin(door_top, edge_top),
            wood_planks: FaceTextures::uniform(wood_planks),
            // End grain on +Y/-Y, bark on the four sides.
            log: FaceTextures::new(log_side, log_side, log_end, log_end, log_side, log_side),
            cobblestone: FaceTextures::uniform(cobblestone),
            sand: FaceTextures::uniform(sand),
            // The reference shows a distinct top (and bottom) from the four
            // sides, so the block is not a single-texture cube.
            deepslate: FaceTextures::new(
                deepslate_side,
                deepslate_side,
                deepslate_top,
                deepslate_top,
                deepslate_side,
                deepslate_side,
            ),
        })
    }
}

/// Registers the definitive block materials (profiles from
/// `01_bloques_mundo_normal_minecraft.md`).
///
/// - Dirt: one texture on all six faces, very matte (specular 0.02).
/// - Stone: one low-contrast grey texture on all six faces, matte with a very
///   small specular response (0.08, shininess 12). No normal map.
/// - Log: end grain on top and bottom, bark on the four sides (specular 0.07,
///   shininess 10), semimatte.
/// - WoodPlanks: one plank texture on all six faces, semimatte (specular 0.06,
///   shininess 8); the canonical wooden base of the architectural blocks.
/// - DoubleWoodSlab: a full cube that deliberately reuses the WoodPlanks
///   texture (same texture id) with the wood profile of the spec (specular
///   0.10, shininess 18, reflectivity 0.02).
/// - WoodStairs: the Gate 06 stepped geometry, now wearing the final
///   WoodPlanks texture with the wood profile (specular 0.10, shininess 18).
/// - Fence: the Gate 06 post-and-rails geometry with the WoodPlanks texture
///   and the wood profile (specular 0.10, shininess 18).
/// - WoodDoor: two stacked halves of the reference door texture on the broad
///   faces; the four window panes of the upper half are real holes
///   (`AlphaMode::Cutout`). Wood profile (specular 0.10, shininess 18).
/// - PortalFrameRedObsidian: the project's red-black obsidian (dark crimson
///   mass with red veins, never vanilla purple), opaque and non-emissive
///   (specular 0.05, shininess 8, reflectivity 0.02).
/// - Cobblestone: one texture on all six faces, rough and matte (specular
///   0.04, shininess 5). No normal map: its albedo already carries the joints.
/// - Sand: one texture on all six faces, light and matte (specular 0.03). It
///   is a static block: nothing here (or anywhere) simulates gravity.
/// - Deepslate: dark and matte (specular 0.06, shininess 10). Top/bottom and
///   sides use the two textures visible in its reference. No normal map.
/// Directory (relative to the repository root) of the portal frame texture.
pub const PORTAL_TEXTURES_DIR: &str = "assets/textures/portal";

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
    library.insert(
        sand_material_id(),
        Material::new(Color::new(0.78, 0.72, 0.52, 1.0), 0.03, 5.0)
            .with_reflectivity(0.01)
            .with_face_textures(textures.sand),
    );
    library.insert(
        deepslate_material_id(),
        Material::new(Color::new(0.30, 0.30, 0.32, 1.0), 0.06, 10.0)
            .with_reflectivity(0.015)
            .with_face_textures(textures.deepslate),
    );
    library.insert(
        stone_block_material_id(),
        Material::new(Color::new(0.50, 0.50, 0.50, 1.0), 0.08, 12.0)
            .with_reflectivity(0.02)
            .with_face_textures(textures.stone),
    );
    library.insert(
        log_material_id(),
        Material::new(Color::new(0.40, 0.31, 0.19, 1.0), 0.07, 10.0)
            .with_reflectivity(0.015)
            .with_face_textures(textures.log),
    );
    library.insert(
        wood_planks_material_id(),
        Material::new(Color::new(0.63, 0.50, 0.30, 1.0), 0.06, 8.0)
            .with_reflectivity(0.01)
            .with_face_textures(textures.wood_planks),
    );
    library.insert(
        double_wood_slab_material_id(),
        Material::new(Color::new(0.63, 0.50, 0.30, 1.0), 0.10, 18.0)
            .with_reflectivity(0.02)
            .with_face_textures(textures.wood_planks),
    );
    library.insert(
        wood_stairs_material_id(),
        Material::new(Color::new(0.63, 0.50, 0.30, 1.0), 0.10, 18.0)
            .with_reflectivity(0.02)
            .with_face_textures(textures.wood_planks),
    );
    library.insert(
        fence_material_id(),
        Material::new(Color::new(0.63, 0.50, 0.30, 1.0), 0.10, 18.0)
            .with_reflectivity(0.02)
            .with_face_textures(textures.wood_planks),
    );
    for (id, faces) in [
        (wood_door_bottom_material_id(), textures.wood_door_bottom),
        (wood_door_top_material_id(), textures.wood_door_top),
    ] {
        library.insert(
            id,
            Material::new(Color::new(0.55, 0.42, 0.24, 1.0), 0.10, 18.0)
                .with_reflectivity(0.02)
                .with_face_textures(faces)
                .with_alpha_mode(AlphaMode::Cutout),
        );
    }
    library.insert(
        portal_frame_material_id(),
        Material::new(Color::new(0.20, 0.04, 0.06, 1.0), 0.05, 8.0)
            .with_reflectivity(0.02)
            .with_face_textures(textures.portal_frame),
    );
}
