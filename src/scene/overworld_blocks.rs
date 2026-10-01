// Catalog-stage component: the definitive Overworld full-cube blocks. Each
// block is an ordinary `BlockType` + `MaterialId`; this module owns only the
// material profiles and the loading of their reference-derived face textures,
// so the catalog (and later the world) never restates them.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::hit::Face;
use crate::core::material::{AlphaMode, Material, MaterialId};
use crate::core::texture::TextureId;
use crate::scene::material_library::MaterialLibrary;
use crate::scene::red_black_timber::{
    RED_BLACK_TIMBER_DIR, red_black_leaves_material_id, red_black_log_material_id,
};
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

pub fn budding_amethyst_material_id() -> MaterialId {
    MaterialId::new(44)
}

pub fn amethyst_cluster_material_id() -> MaterialId {
    MaterialId::new(45)
}

pub fn mycelium_material_id() -> MaterialId {
    MaterialId::new(46)
}

pub fn smooth_basalt_material_id() -> MaterialId {
    MaterialId::new(47)
}

pub fn crimson_heart_material_id() -> MaterialId {
    MaterialId::new(48)
}

pub fn crimson_diamond_material_id() -> MaterialId {
    MaterialId::new(49)
}

pub fn orange_club_material_id() -> MaterialId {
    MaterialId::new(50)
}

pub fn orange_spade_material_id() -> MaterialId {
    MaterialId::new(51)
}

pub fn purple_heart_material_id() -> MaterialId {
    MaterialId::new(52)
}

pub fn purple_diamond_material_id() -> MaterialId {
    MaterialId::new(53)
}

pub fn purple_club_material_id() -> MaterialId {
    MaterialId::new(54)
}

pub fn crying_obsidian_crimson_material_id() -> MaterialId {
    MaterialId::new(55)
}

pub fn crying_obsidian_orange_material_id() -> MaterialId {
    MaterialId::new(56)
}

pub fn crying_obsidian_violet_material_id() -> MaterialId {
    MaterialId::new(57)
}

pub fn red_black_deepslate_bricks_crimson_material_id() -> MaterialId {
    MaterialId::new(58)
}

pub fn red_black_deepslate_bricks_orange_material_id() -> MaterialId {
    MaterialId::new(59)
}

pub fn red_black_deepslate_bricks_violet_material_id() -> MaterialId {
    MaterialId::new(60)
}

pub fn polished_blackstone_bricks_material_id() -> MaterialId {
    MaterialId::new(61)
}

pub fn nether_wart_block_material_id() -> MaterialId {
    MaterialId::new(62)
}

pub fn purple_spade_material_id() -> MaterialId {
    MaterialId::new(63)
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
    pub budding_amethyst: FaceTextures,
    /// Crystal facets of the partial-geometry cluster (cell-space UV, so the
    /// light tips sit in the upper rows).
    pub amethyst_cluster: FaceTextures,
    /// Fungal top, living-surface sides, earthy (Dirt) bottom.
    pub mycelium: FaceTextures,
    pub smooth_basalt: FaceTextures,
    pub crimson_heart: FaceTextures,
    pub purple_spade: FaceTextures,
    /// Emissive colour masks of the card-suit symbols (black off-symbol).
    pub crimson_heart_glow: TextureId,
    pub crimson_diamond_glow: TextureId,
    pub orange_club_glow: TextureId,
    pub orange_spade_glow: TextureId,
    pub purple_heart_glow: TextureId,
    pub purple_diamond_glow: TextureId,
    pub purple_club_glow: TextureId,
    pub purple_spade_glow: TextureId,
    pub crimson_diamond: FaceTextures,
    pub orange_club: FaceTextures,
    pub orange_spade: FaceTextures,
    pub purple_heart: FaceTextures,
    pub purple_diamond: FaceTextures,
    pub purple_club: FaceTextures,
    /// Crying obsidian: one pattern on all six faces, cracks glowing
    /// through their own emissive colour mask.
    pub crying_obsidian_crimson: FaceTextures,
    pub crying_obsidian_crimson_glow: TextureId,
    pub crying_obsidian_orange: FaceTextures,
    pub crying_obsidian_orange_glow: TextureId,
    pub crying_obsidian_violet: FaceTextures,
    pub crying_obsidian_violet_glow: TextureId,
    /// Red-Black deepslate bricks: the DeepslateBricks masonry with a few
    /// recoloured joints, edges and fragments, sharing its normal map.
    pub red_black_bricks_crimson: FaceTextures,
    pub red_black_bricks_crimson_glow: TextureId,
    pub red_black_bricks_orange: FaceTextures,
    pub red_black_bricks_orange_glow: TextureId,
    pub red_black_bricks_violet: FaceTextures,
    pub red_black_bricks_violet_glow: TextureId,
    pub deepslate_bricks_normal: TextureId,
    pub polished_blackstone_bricks: FaceTextures,
    pub nether_wart_block: FaceTextures,
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
    /// Red-Black timber family (Gate 17.5): crimson foliage, same alpha
    /// mask layout as the Overworld leaves.
    pub red_black_leaves: FaceTextures,
    /// Corinto logs: dark crimson end grain on +Y/-Y, burgundy bark on the
    /// four sides, exactly the Overworld log's face layout.
    pub red_black_log: FaceTextures,
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
        let budding = manager.load(format!("{RED_BLACK_TEXTURES_DIR}/budding_amethyst.png"))?;
        let cluster = manager.load(format!("{RED_BLACK_TEXTURES_DIR}/amethyst_cluster.png"))?;
        let mycelium_top = manager.load(format!("{RED_BLACK_TEXTURES_DIR}/mycelium/top.png"))?;
        let mycelium_side = manager.load(format!("{RED_BLACK_TEXTURES_DIR}/mycelium/side.png"))?;
        let smooth_basalt = manager.load(format!("{RED_BLACK_TEXTURES_DIR}/smooth_basalt.png"))?;
        let crimson_heart_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crimson/heart_emissive.png"
        ))?;
        let crimson_diamond_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crimson/diamond_emissive.png"
        ))?;
        let orange_club_glow =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/orange/club_emissive.png"))?;
        let orange_spade_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/orange/spade_emissive.png"
        ))?;
        let purple_heart_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/violet/heart_emissive.png"
        ))?;
        let purple_diamond_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/violet/diamond_emissive.png"
        ))?;
        let purple_club_glow =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/violet/club_emissive.png"))?;
        let purple_spade =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/violet/spade_albedo.png"))?;
        let purple_spade_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/violet/spade_emissive.png"
        ))?;
        let crimson_heart =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/crimson/heart_albedo.png"))?;
        let crimson_diamond = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crimson/diamond_albedo.png"
        ))?;
        let orange_club =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/orange/club_albedo.png"))?;
        let orange_spade =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/orange/spade_albedo.png"))?;
        let purple_heart =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/violet/heart_albedo.png"))?;
        let purple_diamond = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/violet/diamond_albedo.png"
        ))?;
        let purple_club =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/violet/club_albedo.png"))?;
        let crying_crimson = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/crimson_albedo.png"
        ))?;
        let crying_crimson_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/crimson_emissive.png"
        ))?;
        let crying_orange = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/orange_albedo.png"
        ))?;
        let crying_orange_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/orange_emissive.png"
        ))?;
        let crying_violet = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/violet_albedo.png"
        ))?;
        let crying_violet_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/crying_obsidian/violet_emissive.png"
        ))?;
        // Same path as the gallery's DeepslateBricks normal map, so the
        // manager hands back the very same texture.
        let deepslate_bricks_normal =
            manager.load(format!("{overworld_dir}/deepslate_bricks/normal.png"))?;
        let rb_bricks_crimson = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/crimson_albedo.png"
        ))?;
        let rb_bricks_crimson_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/crimson_emissive.png"
        ))?;
        let rb_bricks_orange = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/orange_albedo.png"
        ))?;
        let rb_bricks_orange_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/orange_emissive.png"
        ))?;
        let rb_bricks_violet = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/violet_albedo.png"
        ))?;
        let rb_bricks_violet_glow = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/redblack_deepslate_bricks/violet_emissive.png"
        ))?;
        let polished_blackstone = manager.load(format!(
            "{RED_BLACK_TEXTURES_DIR}/polished_blackstone_bricks.png"
        ))?;
        let nether_wart =
            manager.load(format!("{RED_BLACK_TEXTURES_DIR}/nether_wart_block.png"))?;
        let cobblestone = manager.load(format!("{overworld_dir}/cobblestone/albedo.png"))?;
        let sand = manager.load(format!("{overworld_dir}/sand.png"))?;
        let deepslate_top = manager.load(format!("{overworld_dir}/deepslate/top.png"))?;
        let deepslate_side = manager.load(format!("{overworld_dir}/deepslate/side.png"))?;
        let red_black_leaves = manager.load(format!("{RED_BLACK_TIMBER_DIR}/leaves.png"))?;
        let rb_log_end = manager.load(format!("{RED_BLACK_TIMBER_DIR}/log/end.png"))?;
        let rb_log_side = manager.load(format!("{RED_BLACK_TIMBER_DIR}/log/side.png"))?;

        Ok(Self {
            dirt: FaceTextures::uniform(dirt),
            stone: FaceTextures::uniform(stone),
            budding_amethyst: FaceTextures::uniform(budding),
            amethyst_cluster: FaceTextures::uniform(cluster),
            // The reference's lower side is plain dirt, so the bottom reuses
            // the Dirt texture (same id) instead of a copy of it.
            mycelium: FaceTextures::new(
                mycelium_side,
                mycelium_side,
                mycelium_top,
                dirt,
                mycelium_side,
                mycelium_side,
            ),
            // Its reference shows the same texture on the top and the sides.
            smooth_basalt: FaceTextures::uniform(smooth_basalt),
            crimson_heart: FaceTextures::uniform(crimson_heart),
            crimson_heart_glow,
            crimson_diamond_glow,
            orange_club_glow,
            orange_spade_glow,
            purple_heart_glow,
            purple_diamond_glow,
            purple_club_glow,
            purple_spade_glow,
            purple_spade: FaceTextures::uniform(purple_spade),
            crimson_diamond: FaceTextures::uniform(crimson_diamond),
            orange_club: FaceTextures::uniform(orange_club),
            orange_spade: FaceTextures::uniform(orange_spade),
            purple_heart: FaceTextures::uniform(purple_heart),
            purple_diamond: FaceTextures::uniform(purple_diamond),
            purple_club: FaceTextures::uniform(purple_club),
            crying_obsidian_crimson: FaceTextures::uniform(crying_crimson),
            crying_obsidian_crimson_glow: crying_crimson_glow,
            crying_obsidian_orange: FaceTextures::uniform(crying_orange),
            crying_obsidian_orange_glow: crying_orange_glow,
            crying_obsidian_violet: FaceTextures::uniform(crying_violet),
            crying_obsidian_violet_glow: crying_violet_glow,
            red_black_bricks_crimson: FaceTextures::uniform(rb_bricks_crimson),
            red_black_bricks_crimson_glow: rb_bricks_crimson_glow,
            red_black_bricks_orange: FaceTextures::uniform(rb_bricks_orange),
            red_black_bricks_orange_glow: rb_bricks_orange_glow,
            red_black_bricks_violet: FaceTextures::uniform(rb_bricks_violet),
            red_black_bricks_violet_glow: rb_bricks_violet_glow,
            deepslate_bricks_normal,
            // Its reference shows one texture on the top and the sides.
            polished_blackstone_bricks: FaceTextures::uniform(polished_blackstone),
            nether_wart_block: FaceTextures::uniform(nether_wart),
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
            red_black_leaves: FaceTextures::uniform(red_black_leaves),
            red_black_log: FaceTextures::new(
                rb_log_side,
                rb_log_side,
                rb_log_end,
                rb_log_end,
                rb_log_side,
                rb_log_side,
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
/// - BuddingAmethyst: a full violet crystalline cube (specular 0.10, shininess
///   18, reflectivity 0.03), opaque and non-emissive. It is the mother rock,
///   distinct from the partial-geometry AmethystCluster.
/// - AmethystCluster: the Gate 06 clustered crystal geometry (unchanged),
///   wearing crystal facets taken from its reference palette (violet body,
///   lilac/pink edges, cream highlights). Final crystal optics: strong
///   highlights (specular 0.65, shininess 110), reflectivity 0.20, slight
///   translucency (0.08, IOR 1.45) and a soft violet glow (0.6) from its own
///   facets: a crystal accent, never glass nor a lamp.
/// - Mycelium: grey-violet fungal top, sides where that surface drips over
///   the soil, and a Dirt bottom. Opaque, matte, non-emissive (specular 0.03,
///   shininess 5, no reflectivity).
/// - SmoothBasalt: dense, dark blue-grey volcanic stone, one texture on all
///   six faces (darker than Stone and Deepslate, far less contrast than
///   Cobblestone). Opaque, non-emissive (specular 0.05, shininess 8,
///   reflectivity 0.01).
/// - CrimsonHeart: heart.png re-drawn as a 16x16 Red-Black block: a deep
///   crimson heart (Dark outline, Bright body, Medium bevel, one ivory glint)
///   on dark wine masonry. Opaque, non-emissive (specular 0.06, shininess 8,
///   reflectivity 0.01).
/// - CrimsonDiamond: diamond.png re-drawn as a 16x16 Red-Black block: a deep
///   crimson rhombus with its ivory centre facet on dark wine masonry.
///   Opaque, non-emissive, a little more polished than the heart (specular
///   0.10, shininess 18, reflectivity 0.03).
/// - OrangeClub: club.png re-drawn as a 16x16 Red-Black block and recoloured
///   to the warm Orange family (three-lobed club with stem and foot) on
///   near-black masonry. Opaque, non-emissive (specular 0.06, shininess 8,
///   reflectivity 0.01).
/// - OrangeSpade: spade.png re-drawn as a 16x16 Red-Black block with the
///   spade tinted to the warm Orange family (pointed tip, round lower lobes,
///   stem and foot) on near-black masonry. Opaque, non-emissive (specular
///   0.06, shininess 8, reflectivity 0.01).
/// - PurpleHeart: the same re-drawn heart in the Violet family (dark purple
///   outline, violet body, lilac glint) on dark purple masonry, clearly
///   distinct from CrimsonHeart. Opaque, non-emissive (specular 0.06,
///   shininess 8, reflectivity 0.01).
/// - PurpleDiamond: the same re-drawn rhombus in the Violet family with a
///   lilac centre facet on dark purple masonry, clearly distinct from
///   CrimsonDiamond. Opaque, non-emissive, with the diamond sheen (specular
///   0.10, shininess 18, reflectivity 0.03).
/// - PurpleSpade: the OrangeSpade re-drawing (same spade, bevel and glint)
///   in the Violet family on dark purple masonry, with the Purple glow.
/// - PurpleClub: the same re-drawn three-lobed club in the Violet family on
///   dark purple masonry, clearly distinct from OrangeClub. Opaque,
///   non-emissive (specular 0.06, shininess 8, reflectivity 0.01).
/// - Cobblestone: one texture on all six faces, rough and matte (specular
///   0.04, shininess 5). No normal map: its albedo already carries the joints.
/// - Sand: one texture on all six faces, light and matte (specular 0.03). It
///   is a static block: nothing here (or anywhere) simulates gravity.
/// - Deepslate: dark and matte (specular 0.06, shininess 10). Top/bottom and
///   sides use the two textures visible in its reference. No normal map.
/// Directory (relative to the repository root) of the portal frame texture.
pub const PORTAL_TEXTURES_DIR: &str = "assets/textures/portal";

/// Directory (relative to the repository root) of the Red-Black block textures.
pub const RED_BLACK_TEXTURES_DIR: &str = "assets/textures/red_black_maze";

/// Self-illumination of a Red-Black block (card-suit symbols, crying
/// obsidian cracks). `mask` is an emissive
/// colour mask sharing the albedo UV, exactly like the redstone lamp's: the
/// symbol texels carry the colour they radiate and every other texel (the
/// masonry, its joints and the dark frame) is black, so it never emits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolGlow {
    pub mask: TextureId,
    pub strength: f32,
}

/// Soft violet self-illumination of the amethyst cluster crystals.
pub const AMETHYST_CLUSTER_GLOW_STRENGTH: f32 = 0.6;

/// Emission strength of the crying-obsidian cracks (every colour family).
pub const CRYING_OBSIDIAN_GLOW_STRENGTH: f32 = 1.4;

/// Emission strength of the Red-Black deepslate-brick joint accents: barely
/// there, so the bricks stay structural rather than a light source.
pub const RED_BLACK_BRICKS_GLOW_STRENGTH: f32 = 0.25;

/// Emission strength of the Crimson symbols (heart, diamond).
pub const CRIMSON_GLOW_STRENGTH: f32 = 1.8;

/// Emission strength of the Orange symbols (club, spade).
pub const ORANGE_GLOW_STRENGTH: f32 = 1.8;

/// Emission strength of the Purple symbols (heart, diamond, club, spade).
pub const PURPLE_GLOW_STRENGTH: f32 = 1.8;

/// Material of a glowing Red-Black block: an opaque, textured full cube with the
/// given `(specular, shininess, reflectivity)` profile and, when `glow` is
/// present, the existing emissive path (`emissive_texture` +
/// `emission_strength`). With `glow = None` (or a zero strength) the block
/// shades exactly as a plain non-emissive block.
pub fn symbol_block_material(
    base: Color,
    (specular, shininess, reflectivity): (f32, f32, f32),
    faces: FaceTextures,
    glow: Option<SymbolGlow>,
) -> Material {
    let material = Material::new(base, specular, shininess)
        .with_reflectivity(reflectivity)
        .with_face_textures(faces);
    match glow {
        Some(glow) => material
            .with_emissive_texture(glow.mask)
            .with_emission_strength(glow.strength),
        None => material,
    }
}

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
    library.insert(
        budding_amethyst_material_id(),
        Material::new(Color::new(0.45, 0.30, 0.70, 1.0), 0.10, 18.0)
            .with_reflectivity(0.03)
            .with_face_textures(textures.budding_amethyst),
    );
    // The cluster's crystal facets double as its emissive mask (same texture
    // id), so the soft glow carries each facet's own violet/lilac tone.
    library.insert(
        amethyst_cluster_material_id(),
        Material::new(Color::new(0.58, 0.40, 0.82, 1.0), 0.65, 110.0)
            .with_reflectivity(0.20)
            .with_transparency(0.08)
            .with_refractive_index(1.45)
            .with_face_textures(textures.amethyst_cluster)
            .with_emissive_texture(textures.amethyst_cluster.texture_for_face(Face::PositiveZ))
            .with_emission_strength(AMETHYST_CLUSTER_GLOW_STRENGTH),
    );
    library.insert(
        mycelium_material_id(),
        Material::new(Color::new(0.43, 0.38, 0.40, 1.0), 0.03, 5.0)
            .with_face_textures(textures.mycelium),
    );
    library.insert(
        smooth_basalt_material_id(),
        Material::new(Color::new(0.28, 0.28, 0.30, 1.0), 0.05, 8.0)
            .with_reflectivity(0.01)
            .with_face_textures(textures.smooth_basalt),
    );
    // The Red-Black card-suit blocks share one profile builder so their
    // optional symbol glow is wired exactly like the redstone lamp's.
    let crimson = Color::new(0.82, 0.12, 0.17, 1.0);
    let orange = Color::new(1.00, 0.48, 0.00, 1.0);
    let purple = Color::new(0.76, 0.24, 1.00, 1.0);
    // (specular, shininess, reflectivity): diamonds are a little more polished.
    let plain = (0.06, 8.0, 0.01);
    let polished = (0.10, 18.0, 0.03);
    let glow = |mask, strength| Some(SymbolGlow { mask, strength });
    let crimson_glow = |mask| glow(mask, CRIMSON_GLOW_STRENGTH);
    let orange_glow = |mask| glow(mask, ORANGE_GLOW_STRENGTH);
    let purple_glow = |mask| glow(mask, PURPLE_GLOW_STRENGTH);
    library.insert(
        crimson_heart_material_id(),
        symbol_block_material(
            crimson,
            plain,
            textures.crimson_heart,
            crimson_glow(textures.crimson_heart_glow),
        ),
    );
    library.insert(
        crimson_diamond_material_id(),
        symbol_block_material(
            crimson,
            polished,
            textures.crimson_diamond,
            crimson_glow(textures.crimson_diamond_glow),
        ),
    );
    library.insert(
        orange_club_material_id(),
        symbol_block_material(
            orange,
            plain,
            textures.orange_club,
            orange_glow(textures.orange_club_glow),
        ),
    );
    library.insert(
        orange_spade_material_id(),
        symbol_block_material(
            orange,
            plain,
            textures.orange_spade,
            orange_glow(textures.orange_spade_glow),
        ),
    );
    library.insert(
        purple_heart_material_id(),
        symbol_block_material(
            purple,
            plain,
            textures.purple_heart,
            purple_glow(textures.purple_heart_glow),
        ),
    );
    library.insert(
        purple_diamond_material_id(),
        symbol_block_material(
            purple,
            polished,
            textures.purple_diamond,
            purple_glow(textures.purple_diamond_glow),
        ),
    );
    library.insert(
        purple_club_material_id(),
        symbol_block_material(
            purple,
            plain,
            textures.purple_club,
            purple_glow(textures.purple_club_glow),
        ),
    );
    library.insert(
        purple_spade_material_id(),
        symbol_block_material(
            purple,
            plain,
            textures.purple_spade,
            purple_glow(textures.purple_spade_glow),
        ),
    );
    // Crying obsidian: a near-black, slightly polished body (specular 0.30,
    // shininess 60, reflectivity 0.12) whose cracks alone glow.
    let crying = (0.30, 60.0, 0.12);
    library.insert(
        crying_obsidian_crimson_material_id(),
        symbol_block_material(
            Color::new(0.30, 0.03, 0.07, 1.0),
            crying,
            textures.crying_obsidian_crimson,
            glow(
                textures.crying_obsidian_crimson_glow,
                CRYING_OBSIDIAN_GLOW_STRENGTH,
            ),
        ),
    );
    library.insert(
        crying_obsidian_orange_material_id(),
        symbol_block_material(
            Color::new(0.30, 0.03, 0.07, 1.0),
            crying,
            textures.crying_obsidian_orange,
            glow(
                textures.crying_obsidian_orange_glow,
                CRYING_OBSIDIAN_GLOW_STRENGTH,
            ),
        ),
    );
    library.insert(
        crying_obsidian_violet_material_id(),
        symbol_block_material(
            Color::new(0.30, 0.03, 0.07, 1.0),
            crying,
            textures.crying_obsidian_violet,
            glow(
                textures.crying_obsidian_violet_glow,
                CRYING_OBSIDIAN_GLOW_STRENGTH,
            ),
        ),
    );
    // Red-Black deepslate bricks: the DeepslateBricks relief (same normal
    // map) with the structural profile of the spec.
    let bricks = (0.10, 18.0, 0.025);
    library.insert(
        red_black_deepslate_bricks_crimson_material_id(),
        symbol_block_material(
            Color::new(0.25, 0.22, 0.22, 1.0),
            bricks,
            textures.red_black_bricks_crimson,
            glow(
                textures.red_black_bricks_crimson_glow,
                RED_BLACK_BRICKS_GLOW_STRENGTH,
            ),
        )
        .with_normal_texture(textures.deepslate_bricks_normal),
    );
    library.insert(
        red_black_deepslate_bricks_orange_material_id(),
        symbol_block_material(
            Color::new(0.25, 0.22, 0.22, 1.0),
            bricks,
            textures.red_black_bricks_orange,
            glow(
                textures.red_black_bricks_orange_glow,
                RED_BLACK_BRICKS_GLOW_STRENGTH,
            ),
        )
        .with_normal_texture(textures.deepslate_bricks_normal),
    );
    library.insert(
        red_black_deepslate_bricks_violet_material_id(),
        symbol_block_material(
            Color::new(0.25, 0.22, 0.22, 1.0),
            bricks,
            textures.red_black_bricks_violet,
            glow(
                textures.red_black_bricks_violet_glow,
                RED_BLACK_BRICKS_GLOW_STRENGTH,
            ),
        )
        .with_normal_texture(textures.deepslate_bricks_normal),
    );
    // Polished blackstone bricks: refined anthracite masonry, semipolished
    // but never metallic (specular 0.20, shininess 38, reflectivity 0.06),
    // opaque and non-emissive. Its albedo already carries the joints.
    library.insert(
        polished_blackstone_bricks_material_id(),
        Material::new(Color::new(0.16, 0.14, 0.16, 1.0), 0.20, 38.0)
            .with_reflectivity(0.06)
            .with_face_textures(textures.polished_blackstone_bricks),
    );
    // Red-Black leaves: the Overworld leaves' profile (matte, alpha cutout,
    // never refractive nor emissive) over burgundy and wine foliage.
    library.insert(
        red_black_leaves_material_id(),
        Material::new(Color::new(0.36, 0.06, 0.10, 1.0), 0.03, 4.0)
            .with_face_textures(textures.red_black_leaves)
            .with_alpha_mode(AlphaMode::Cutout),
    );
    // Red-Black log: the Overworld log's semimatte profile over almost-black
    // burgundy bark and dark crimson rings; never emissive.
    library.insert(
        red_black_log_material_id(),
        Material::new(Color::new(0.22, 0.06, 0.10, 1.0), 0.07, 10.0)
            .with_reflectivity(0.015)
            .with_face_textures(textures.red_black_log),
    );
    // Nether wart block: dense fungal red, matte (specular 0.04, shininess 6,
    // reflectivity 0.01), opaque and never a light source of its own.
    library.insert(
        nether_wart_block_material_id(),
        Material::new(Color::new(0.44, 0.01, 0.01, 1.0), 0.04, 6.0)
            .with_reflectivity(0.01)
            .with_face_textures(textures.nether_wart_block),
    );
}
