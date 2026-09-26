// Inspection-stage scene: the persistent block catalog used to look at every
// block type from any angle. It is the Gate 06 partial-geometry shapes plus
// the Gate 07 optical gallery, organized in rows and made selectable — not a
// second block system. Every sample is an ordinary `BlockInstance` in the real
// `VoxelWorld`, resolved through the real `MaterialLibrary`, so it renders via
// the same DDA, block geometry, and advanced-material code as the final world.
#![allow(dead_code)]

use crate::camera::diagnostic::DiagnosticCameraState;
use crate::core::material::MaterialId;
use crate::core::math::{IVec3, Vec3};
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::material_gallery::{
    BACK_Z, BRICKS_X, FRONT_Z, GALLERY_WIDTH, GALLERY_Z_MIN, GLASS_X, GalleryTextures, LAMP_X,
    LEAVES_X, MATTE_X, MIRROR_X, PORTAL_X, WATER_X, advanced_materials_library,
    advanced_materials_world, checker_dark_material_id, checker_light_material_id,
    deepslate_bricks_material_id, glass_material_id, leaves_material_id, matte_material_id,
    mirror_material_id, portal_core_material_id, redstone_lamp_material_id, water_material_id,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::double_wood_slab_material_id;
use crate::scene::overworld_blocks::fence_material_id;
use crate::scene::overworld_blocks::portal_frame_material_id;
use crate::scene::overworld_blocks::wood_planks_material_id;
use crate::scene::overworld_blocks::wood_stairs_material_id;
use crate::scene::overworld_blocks::{
    OverworldBlockTextures, cobblestone_material_id, deepslate_material_id, dirt_material_id,
    insert_overworld_materials, log_material_id, sand_material_id, stone_block_material_id,
};
use crate::scene::overworld_blocks::{wood_door_bottom_material_id, wood_door_top_material_id};
use crate::scene::scene::{
    PartialSceneTextures, amethyst_material_id, diagnostic_partial_materials, grass_material_id,
    wood_material_id,
};
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
use crate::scene::voxel_world::VoxelWorld;

/// Row of the Gate 06 partial-geometry shapes, between the two gallery rows.
pub const SHAPES_Z: i32 = 2;
pub const STAIRS_X: i32 = 0;
pub const FENCE_X: i32 = 3;
pub const DOOR_X: i32 = 6;
pub const AMETHYST_X: i32 = 9;

/// Row of the definitive Overworld I full-cube blocks, behind the gallery's
/// back row. Slots (x): Grass 1, Dirt 3, Stone 5 (reserved), Cobblestone 7,
/// Sand 9, Deepslate 11 (Stone now fills its reserved slot).
pub const OVERWORLD_I_Z: i32 = -4;
/// Row of the Overworld II / Portal blocks behind the Overworld I row.
pub const OVERWORLD_II_Z: i32 = -6;
pub const LOG_X: i32 = 2;
pub const WOOD_PLANKS_X: i32 = 4;
pub const DOUBLE_SLAB_X: i32 = 6;
pub const PORTAL_FRAME_X: i32 = 8;
pub const GRASS_X: i32 = 1;
pub const DIRT_X: i32 = 3;
pub const STONE_X: i32 = 5;
pub const COBBLESTONE_X: i32 = 7;
pub const SAND_X: i32 = 9;
pub const DEEPSLATE_X: i32 = 11;

/// Standard inspection distance used when focusing a sample.
pub const FOCUS_DISTANCE: f32 = 4.5;

/// What each sample demonstrates, for the on-screen label and the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleKind {
    /// Plain full cube used as the untouched reference.
    Control,
    /// A definitive Overworld full-cube block (opaque, non-emissive).
    PlainBlock,
    /// A full cube with strong `reflectivity`.
    Reflective,
    /// Non-cubic geometry resolved through `BlockGeometry`.
    PartialGeometry,
    /// Continuous transparency + refraction.
    Transmissive,
    /// Alpha cutout.
    Cutout,
    /// Emissive mask.
    Emissive,
    /// Tangent-space normal map.
    NormalMapped,
}

/// Placement and description of one catalog sample. It carries no material
/// data and does not replace `BlockInstance`: `block()` builds the real
/// instance the `VoxelWorld` stores.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    pub display_name: &'static str,
    pub kind: SampleKind,
    pub block_type: BlockType,
    pub material_id: MaterialId,
    pub position: IVec3,
    pub orientation: Orientation,
    /// World-space point the camera orbits when this sample is focused.
    pub focus_point: Vec3,
    /// Further cells that belong to the same sample (the door's upper half).
    pub extra_blocks: Vec<(IVec3, BlockInstance)>,
}

impl CatalogEntry {
    fn new(
        display_name: &'static str,
        kind: SampleKind,
        block_type: BlockType,
        material_id: MaterialId,
        position: IVec3,
        orientation: Orientation,
    ) -> Self {
        Self {
            display_name,
            kind,
            block_type,
            material_id,
            position,
            orientation,
            focus_point: cell_center(position),
            extra_blocks: Vec::new(),
        }
    }

    /// The real block instance stored at `position`.
    pub fn block(&self) -> BlockInstance {
        BlockInstance::new(self.block_type, self.material_id, self.orientation)
    }
}

fn cell_center(cell: IVec3) -> Vec3 {
    Vec3::new(
        cell.x as f32 + 0.5,
        cell.y as f32 + 0.5,
        cell.z as f32 + 0.5,
    )
}

/// The catalog samples in inspection order, three rows of four:
///
/// ```text
/// control  | reflective | glass     | water
/// leaves   | redstone   | portal    | normal-map
/// stairs   | fence      | door      | amethyst
/// ```
///
/// (Physically the gallery's back row is `z = -1`, the shapes row `z = 2`
/// and the front row `z = 5`; the order is the reading order above.)
pub fn catalog_entries() -> Vec<CatalogEntry> {
    let at = |x: i32, z: i32| IVec3::new(x, 1, z);
    let up = Orientation::Up;
    let south = Orientation::South;

    let mut door = CatalogEntry::new(
        "WoodDoor",
        SampleKind::PartialGeometry,
        BlockType::WoodDoor,
        wood_door_bottom_material_id(),
        at(DOOR_X, SHAPES_Z),
        south,
    );
    // A door is two cells tall; the upper half has its own texture. Focus on
    // the middle of the whole door.
    door.extra_blocks.push((
        IVec3::new(DOOR_X, 2, SHAPES_Z),
        BlockInstance::new(BlockType::WoodDoor, wood_door_top_material_id(), south),
    ));
    door.focus_point = Vec3::new(DOOR_X as f32 + 0.5, 2.0, SHAPES_Z as f32 + 0.5);

    vec![
        CatalogEntry::new(
            "Grass",
            SampleKind::PlainBlock,
            BlockType::Grass,
            grass_material_id(),
            at(GRASS_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Dirt",
            SampleKind::PlainBlock,
            BlockType::Dirt,
            dirt_material_id(),
            at(DIRT_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Cobblestone",
            SampleKind::PlainBlock,
            BlockType::Cobblestone,
            cobblestone_material_id(),
            at(COBBLESTONE_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Sand",
            SampleKind::PlainBlock,
            BlockType::Sand,
            sand_material_id(),
            at(SAND_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Deepslate",
            SampleKind::PlainBlock,
            BlockType::Deepslate,
            deepslate_material_id(),
            at(DEEPSLATE_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Stone",
            SampleKind::PlainBlock,
            BlockType::Stone,
            stone_block_material_id(),
            at(STONE_X, OVERWORLD_I_Z),
            up,
        ),
        CatalogEntry::new(
            "Log",
            SampleKind::PlainBlock,
            BlockType::Log,
            log_material_id(),
            at(LOG_X, OVERWORLD_II_Z),
            up,
        ),
        CatalogEntry::new(
            "Wood planks",
            SampleKind::PlainBlock,
            BlockType::WoodPlanks,
            wood_planks_material_id(),
            at(WOOD_PLANKS_X, OVERWORLD_II_Z),
            up,
        ),
        CatalogEntry::new(
            "Double wood slab",
            SampleKind::PlainBlock,
            BlockType::DoubleWoodSlab,
            double_wood_slab_material_id(),
            at(DOUBLE_SLAB_X, OVERWORLD_II_Z),
            up,
        ),
        CatalogEntry::new(
            "Portal frame (red obsidian)",
            SampleKind::PlainBlock,
            BlockType::PortalFrameRedObsidian,
            portal_frame_material_id(),
            at(PORTAL_FRAME_X, OVERWORLD_II_Z),
            up,
        ),
        CatalogEntry::new(
            "Control cube",
            SampleKind::Control,
            BlockType::Stone,
            matte_material_id(),
            at(MATTE_X, FRONT_Z),
            up,
        ),
        CatalogEntry::new(
            "Reflective cube",
            SampleKind::Reflective,
            BlockType::Stone,
            mirror_material_id(),
            at(MIRROR_X, FRONT_Z),
            up,
        ),
        CatalogEntry::new(
            "Glass",
            SampleKind::Transmissive,
            BlockType::Glass,
            glass_material_id(),
            at(GLASS_X, FRONT_Z),
            up,
        ),
        CatalogEntry::new(
            "Water",
            SampleKind::Transmissive,
            BlockType::Water,
            water_material_id(),
            at(WATER_X, FRONT_Z),
            up,
        ),
        CatalogEntry::new(
            "Leaves",
            SampleKind::Cutout,
            BlockType::Leaves,
            leaves_material_id(),
            at(LEAVES_X, BACK_Z),
            up,
        ),
        CatalogEntry::new(
            "Redstone Lamp (lit)",
            SampleKind::Emissive,
            BlockType::RedstoneLampLit,
            redstone_lamp_material_id(),
            at(LAMP_X, BACK_Z),
            up,
        ),
        CatalogEntry::new(
            "Portal core",
            SampleKind::PartialGeometry,
            BlockType::PortalCoreDarkCrimson,
            portal_core_material_id(),
            at(PORTAL_X, BACK_Z),
            south,
        ),
        CatalogEntry::new(
            "Deepslate Bricks",
            SampleKind::NormalMapped,
            BlockType::DeepslateBricks,
            deepslate_bricks_material_id(),
            at(BRICKS_X, BACK_Z),
            up,
        ),
        CatalogEntry::new(
            "Wood stairs",
            SampleKind::PartialGeometry,
            BlockType::WoodStairs,
            wood_stairs_material_id(),
            at(STAIRS_X, SHAPES_Z),
            south,
        ),
        CatalogEntry::new(
            "Fence",
            SampleKind::PartialGeometry,
            BlockType::Fence,
            fence_material_id(),
            at(FENCE_X, SHAPES_Z),
            south,
        ),
        door,
        CatalogEntry::new(
            "Amethyst cluster",
            SampleKind::PartialGeometry,
            BlockType::AmethystCluster,
            amethyst_material_id(),
            at(AMETHYST_X, SHAPES_Z),
            up,
        ),
    ]
}

/// Every texture the catalog needs, loaded once through the shared manager.
pub struct CatalogTextures {
    pub gallery: GalleryTextures,
    pub shapes: PartialSceneTextures,
    pub blocks: OverworldBlockTextures,
}

impl CatalogTextures {
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
        portal_dir: &str,
        grass_dir: &str,
        partial_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        Ok(Self {
            gallery: GalleryTextures::load(manager, overworld_dir, portal_dir)?,
            shapes: PartialSceneTextures::load(manager, grass_dir, partial_dir)?,
            blocks: OverworldBlockTextures::load(manager, overworld_dir)?,
        })
    }
}

/// The materials of every sample: the optical gallery's library plus the
/// existing Gate 06 wood, door and amethyst materials, copied unchanged.
pub fn catalog_materials(textures: &CatalogTextures) -> MaterialLibrary {
    let mut library = advanced_materials_library(&textures.gallery);
    let shapes = diagnostic_partial_materials(&textures.shapes);

    for id in [
        grass_material_id(),
        wood_material_id(),
        amethyst_material_id(),
    ] {
        let material = shapes
            .get(id)
            .expect("the Gate 06 shape materials are always defined")
            .clone();
        library.insert(id, material);
    }

    insert_overworld_materials(&mut library, &textures.blocks);

    // Grass keeps its Gate 04 face textures and matte look, plus the 0.01
    // reflectivity of the definitive Overworld profile.
    let grass = library
        .get(grass_material_id())
        .expect("grass was just inserted")
        .clone()
        .with_reflectivity(0.01);
    library.insert(grass_material_id(), grass);

    library
}

/// Continues the gallery's checker floor behind it (`z` from
/// `OVERWORLD_II_Z - 2` up to the gallery's own first row) so the Overworld I
/// and II rows stand on the same floor.
fn extend_floor_backward(world: &mut VoxelWorld) {
    for z in OVERWORLD_II_Z - 2..GALLERY_Z_MIN {
        for x in 0..GALLERY_WIDTH {
            let id = if (x + z) % 2 == 0 {
                checker_light_material_id()
            } else {
                checker_dark_material_id()
            };
            world.insert(
                IVec3::new(x, 0, z),
                BlockInstance::new(BlockType::Stone, id, Orientation::Up),
            );
        }
    }
}

/// The persistent inspection scene: the world, its sample list, and which
/// sample is selected.
pub struct CatalogScene {
    entries: Vec<CatalogEntry>,
    world: VoxelWorld,
    selected: usize,
}

impl CatalogScene {
    /// Builds the world from the gallery (floor, backdrops, reflection patch)
    /// and then places every catalog entry through its own `BlockInstance`.
    pub fn new() -> Self {
        let entries = catalog_entries();
        let mut world = advanced_materials_world();
        extend_floor_backward(&mut world);

        for entry in &entries {
            world.insert(entry.position, entry.block());
            for (cell, block) in &entry.extra_blocks {
                world.insert(*cell, *block);
            }
        }

        Self {
            entries,
            world,
            selected: 0,
        }
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    pub fn entries(&self) -> &[CatalogEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected(&self) -> &CatalogEntry {
        &self.entries[self.selected]
    }

    /// Selects the next sample, wrapping from the last to the first.
    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.entries.len();
    }

    /// Selects the previous sample, wrapping from the first to the last.
    pub fn select_previous(&mut self) {
        self.selected = (self.selected + self.entries.len() - 1) % self.entries.len();
    }

    /// The orbit target for the selected sample.
    pub fn focus_target(&self) -> Vec3 {
        self.selected().focus_point
    }

    /// Centers the camera on the selected sample: only the orbit target and
    /// the distance change (yaw and pitch are kept); no block moves.
    pub fn focus_camera(&self, view: &mut DiagnosticCameraState) {
        view.set_target(self.focus_target());
        view.set_distance(FOCUS_DISTANCE);
    }

    /// One-line on-screen label, e.g. `Selected: Fence | 10 / 12`.
    pub fn label(&self) -> String {
        format!(
            "Selected: {} | {} / {}",
            self.selected().display_name,
            self.selected + 1,
            self.entries.len()
        )
    }
}

impl Default for CatalogScene {
    fn default() -> Self {
        Self::new()
    }
}
