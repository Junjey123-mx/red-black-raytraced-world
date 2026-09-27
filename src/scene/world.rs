// World-stage scene: the main executable's scene (`cargo run`). Gate 11.5
// only reserves it: the voxel world is still empty, lit by a plain sun, with
// the definitive block materials already registered. Gate 12 builds the
// Overworld here. It never reuses `CatalogScene`, which stays the separate
// `catalog` binary's scene.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::color::Color;
use crate::core::math::Vec3;
use crate::scene::light::{DirectionalLight, Light};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::overworld_blocks::{OverworldBlockTextures, insert_overworld_materials};
use crate::scene::voxel_world::VoxelWorld;

/// Scene range of the world: primary traversal and directional shadow rays.
pub const WORLD_MAX_DISTANCE: f32 = 64.0;

/// The main world scene.
pub struct WorldScene {
    world: VoxelWorld,
}

impl WorldScene {
    /// The world before Gate 12: no blocks placed yet.
    pub fn new() -> Self {
        Self {
            world: VoxelWorld::new(),
        }
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }
}

impl Default for WorldScene {
    fn default() -> Self {
        Self::new()
    }
}

/// The world's materials: the same definitive block materials the catalog
/// uses, registered through the shared `insert_overworld_materials`.
pub fn world_materials(textures: &OverworldBlockTextures) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, textures);
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

/// Neutral dark sky until Gate 15 brings the skybox.
pub fn world_background() -> Color {
    Color::new(0.10, 0.12, 0.16, 1.0)
}

/// Orbit camera looking at the world origin from above and in front.
pub fn world_camera(aspect_ratio: f32) -> Camera {
    Camera::new(
        Vec3::new(0.0, 12.0, 24.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    )
}
