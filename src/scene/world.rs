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
use crate::scene::light::{DirectionalLight, Light};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::overworld_blocks::{OverworldBlockTextures, insert_overworld_materials};
use crate::scene::scene::{diagnostic_materials, grass_material_id};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::fbm::HeightField;
use crate::scene::terrain::generator::{TerrainBounds, generate_terrain, terrain_bounds};
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
use crate::scene::voxel_world::VoxelWorld;

/// Scene range of the world: primary traversal and directional shadow rays.
pub const WORLD_MAX_DISTANCE: f32 = 96.0;

/// The main world scene: the Overworld terrain and everything built on it.
pub struct WorldScene {
    config: TerrainConfig,
    field: HeightField,
    bounds: TerrainBounds,
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
        Self {
            config,
            field,
            bounds,
            world,
        }
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
}

impl WorldTextures {
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
        grass_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let blocks = OverworldBlockTextures::load(manager, overworld_dir)?;
        let top = manager.load(format!("{grass_dir}/top.png"))?;
        let side = manager.load(format!("{grass_dir}/side.png"))?;
        let bottom = manager.load(format!("{grass_dir}/bottom.png"))?;
        Ok(Self {
            blocks,
            grass: FaceTextures::new(side, side, top, bottom, side, side),
        })
    }
}

/// The world's materials: the same definitive block materials the catalog
/// uses, registered through the shared `insert_overworld_materials`, plus
/// the Grass Block exactly as the catalog presents it.
pub fn world_materials(textures: &WorldTextures) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &textures.blocks);

    let grass = diagnostic_materials(textures.grass)
        .get(grass_material_id())
        .expect("the grass material is always defined")
        .clone()
        .with_reflectivity(0.01);
    library.insert(grass_material_id(), grass);

    library
}

/// A single white sun from above.
pub fn world_lights() -> Vec<Light> {
    vec![Light::Directional(DirectionalLight::new(
        Vec3::new(0.3, 1.0, 0.4),
        Color::white(),
        1.0,
    ))]
}

/// Neutral dark sky until the daytime sky lands.
pub fn world_background() -> Color {
    Color::new(0.10, 0.12, 0.16, 1.0)
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
