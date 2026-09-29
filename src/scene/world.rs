// World-stage scene: the main executable's scene (`cargo run`). It presents
// the definitive Overworld: the procedurally reliefed terrain mass built by
// the terrain generator, lit by a sun, with the definitive block materials.
// It never reuses `CatalogScene`, which stays the separate `catalog`
// binary's scene.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::math::Vec3;
use crate::renderer::skybox::{Background, SkyGradient};
use crate::scene::light::{DirectionalLight, Light};
use crate::scene::material_gallery::{
    GalleryTextures, advanced_materials_library, glass_material_id, leaves_material_id,
    redstone_lamp_material_id, water_material_id,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::overworld::{
    HouseExterior, HouseLayout, PathLayout, PondLayout, Tree, build_house, carve_pond, column_top,
    furnish_house, lay_path, plant_trees,
};
use crate::scene::overworld_blocks::{OverworldBlockTextures, insert_overworld_materials};
use crate::scene::scene::{diagnostic_materials, grass_material_id};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::fbm::HeightField;
use crate::scene::terrain::generator::{TerrainBounds, generate_terrain, terrain_bounds};
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
use crate::scene::voxel_world::VoxelWorld;

/// Scene range of the world: primary traversal and directional shadow rays.
pub const WORLD_MAX_DISTANCE: f32 = 96.0;

/// Ambient term of the daytime Overworld: enough to keep shadowed faces and
/// the undersides readable under the bright sky, low enough that the sun's
/// shadows stay clearly visible.
pub const WORLD_AMBIENT_FACTOR: f32 = 0.30;

/// The main world scene: the Overworld terrain and everything built on it.
pub struct WorldScene {
    config: TerrainConfig,
    field: HeightField,
    bounds: TerrainBounds,
    pond: PondLayout,
    house: HouseLayout,
    exterior: HouseExterior,
    trees: Vec<Tree>,
    path: PathLayout,
    world: VoxelWorld,
}

impl WorldScene {
    /// Builds the official Overworld.
    pub fn new() -> Self {
        Self::with_config(TerrainConfig::official())
    }

    /// Builds the Overworld of an arbitrary terrain configuration.
    pub fn with_config(config: TerrainConfig) -> Self {
        let mut world = VoxelWorld::new();
        let field = generate_terrain(&config, &mut world);
        let bounds = terrain_bounds(&config, &field);
        let pond = carve_pond(&config, &mut world);
        let house = build_house(&config, &mut world);
        let exterior = furnish_house(&config, &mut world);
        let trees = plant_trees(&config, &mut world, &pond);
        let path = lay_path(&config, &mut world, &pond, &trees);
        Self {
            config,
            field,
            bounds,
            pond,
            house,
            exterior,
            trees,
            path,
            world,
        }
    }

    /// The cobblestone path from the door to the descent point.
    pub fn path(&self) -> &PathLayout {
        &self.path
    }

    /// The roof, door, glass, fence and lamps of the house.
    pub fn house_exterior(&self) -> &HouseExterior {
        &self.exterior
    }

    /// The house built on the plateau.
    pub fn house(&self) -> &HouseLayout {
        &self.house
    }

    /// The trees planted on the terrain.
    pub fn trees(&self) -> &[Tree] {
        &self.trees
    }

    /// The pond dug into the terrain.
    pub fn pond(&self) -> &PondLayout {
        &self.pond
    }

    /// Highest occupied cell of column `(x, z)` (terrain or feature), or
    /// `None` for an empty column.
    pub fn column_top(&self, x: i32, z: i32) -> Option<i32> {
        column_top(&self.world, x, z, self.bounds.max.y + 16, self.bounds.min.y)
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    pub fn config(&self) -> &TerrainConfig {
        &self.config
    }

    /// The height field the terrain columns were generated from.
    pub fn height_field(&self) -> &HeightField {
        &self.field
    }

    /// Bounds of the terrain mass.
    pub fn bounds(&self) -> TerrainBounds {
        self.bounds
    }

    /// World-space center of the terrain mass.
    pub fn center(&self) -> Vec3 {
        let b = self.bounds;
        Vec3::new(
            (b.min.x + b.max.x + 1) as f32 / 2.0,
            (b.min.y + b.max.y + 1) as f32 / 2.0,
            (b.min.z + b.max.z + 1) as f32 / 2.0,
        )
    }
}

impl Default for WorldScene {
    fn default() -> Self {
        Self::new()
    }
}

/// Every texture the world needs, loaded once through the shared manager.
pub struct WorldTextures {
    pub blocks: OverworldBlockTextures,
    /// Grass Block faces (`top/side/bottom.png`).
    pub grass: FaceTextures,
    /// The optical specimens the world reuses (water, and later glass,
    /// leaves and the redstone lamp), loaded exactly as the catalog does.
    pub gallery: GalleryTextures,
}

impl WorldTextures {
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
        grass_dir: &str,
        portal_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let blocks = OverworldBlockTextures::load(manager, overworld_dir)?;
        let gallery = GalleryTextures::load(manager, overworld_dir, portal_dir)?;
        let top = manager.load(format!("{grass_dir}/top.png"))?;
        let side = manager.load(format!("{grass_dir}/side.png"))?;
        let bottom = manager.load(format!("{grass_dir}/bottom.png"))?;
        Ok(Self {
            blocks,
            grass: FaceTextures::new(side, side, top, bottom, side, side),
            gallery,
        })
    }
}

/// The world's materials: the same definitive block materials the catalog
/// uses, registered through the shared `insert_overworld_materials`, the
/// Grass Block exactly as the catalog presents it, and the optical
/// specimens the scene needs copied from the catalog's gallery library
/// (same ids, same profiles).
pub fn world_materials(textures: &WorldTextures) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &textures.blocks);

    let gallery = advanced_materials_library(&textures.gallery);
    for id in [
        water_material_id(),
        leaves_material_id(),
        glass_material_id(),
        redstone_lamp_material_id(),
    ] {
        let material = gallery
            .get(id)
            .expect("the gallery defines every optical specimen")
            .clone();
        library.insert(id, material);
    }

    let grass = diagnostic_materials(textures.grass)
        .get(grass_material_id())
        .expect("the grass material is always defined")
        .clone()
        .with_reflectivity(0.01);
    library.insert(grass_material_id(), grass);

    library
}

/// Direction from any surface toward the sun: high and diagonal (from the
/// south-east and above), so every block shows one lit top, one lit side and
/// one shaded side, with legible cast shadows.
pub fn sun_direction() -> Vec3 {
    Vec3::new(0.45, 1.0, 0.35)
}

/// The sun's color: white with a faint warm tint.
pub fn sun_color() -> Color {
    Color::new(1.0, 0.97, 0.90, 1.0)
}

/// The Overworld's lights: the sun alone (block lamps arrive with the house).
pub fn world_lights() -> Vec<Light> {
    vec![Light::Directional(DirectionalLight::new(
        sun_direction(),
        sun_color(),
        1.0,
    ))]
}

/// The clear daytime sky: a medium celeste-blue zenith, a pale luminous
/// horizon and a very pale haze below it.
pub fn world_sky() -> SkyGradient {
    SkyGradient {
        zenith: Color::new(0.33, 0.56, 0.92, 1.0),
        horizon: Color::new(0.72, 0.85, 0.97, 1.0),
        ground: Color::new(0.80, 0.88, 0.95, 1.0),
    }
}

/// What World rays see on a miss: the daytime sky.
pub fn world_background() -> Background {
    Background::Sky(world_sky())
}

/// Orbit camera looking at the terrain from the front, elevated.
pub fn world_camera(aspect_ratio: f32) -> Camera {
    let config = TerrainConfig::official();
    let center = Vec3::new(
        config.width as f32 / 2.0,
        config.base_height as f32,
        config.depth as f32 / 2.0,
    );
    Camera::new(
        center + Vec3::new(0.0, 18.0, 30.0),
        center,
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    )
}
