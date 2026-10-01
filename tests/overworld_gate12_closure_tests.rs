#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_gallery.rs"]
    pub mod material_gallery;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::math::{IVec3, Vec3};
use renderer::skybox::Background;
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::light::Light;
use scene::material_gallery::gallery_background;
use scene::overworld::{DESCENT_ENDPOINT, HOUSE_FOOTPRINT, POND_WATER_LEVEL, pond_basin_columns};
use scene::terrain::generator::{column_surface, footprint_contains};
use scene::terrain::{MIN_TERRAIN_SIDE, OFFICIAL_SEED, TerrainConfig};
use scene::world::{WorldScene, world_background, world_camera, world_lights, world_sky};
use std::collections::HashMap;

/// Every block of the scene, by type.
fn census(scene: &WorldScene) -> HashMap<BlockType, usize> {
    let c = *scene.config();
    let b = scene.bounds();
    let mut counts = HashMap::new();
    for z in -2..c.depth + 2 {
        for x in -2..c.width + 2 {
            for y in b.min.y..=b.max.y + 16 {
                if let Some(block) = scene.world().get(IVec3::new(x, y, z)) {
                    *counts.entry(block.block_type()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

fn count(counts: &HashMap<BlockType, usize>, t: BlockType) -> usize {
    counts.get(&t).copied().unwrap_or(0)
}

/// Surface heights of the generated terrain columns (before the features).
fn surface_range(scene: &WorldScene) -> (i32, i32) {
    let c = *scene.config();
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    for z in 0..c.depth {
        for x in 0..c.width {
            if footprint_contains(&c, x, z) {
                let s = column_surface(&c, scene.height_field(), x, z).unwrap();
                min = min.min(s);
                max = max.max(s);
            }
        }
    }
    (min, max)
}

#[test]
fn terrain_footprint_relief_and_population() {
    let scene = WorldScene::new();
    let c = *scene.config();
    // 1. terrain >= 16 x 16
    assert!(c.width >= MIN_TERRAIN_SIDE && c.depth >= MIN_TERRAIN_SIDE);
    let b = scene.bounds();
    assert!(b.max.x - b.min.x + 1 >= 16 && b.max.z - b.min.z + 1 >= 16);
    // 2. world not empty
    assert!(!scene.world().is_empty());
    // 3. relief >= 2
    let (min, max) = surface_range(&scene);
    assert!(max - min >= 2, "relief {min}..{max}");
}

#[test]
fn every_required_block_is_present() {
    let scene = WorldScene::new();
    let counts = census(&scene);
    // 4-18.
    for t in [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Deepslate,
        BlockType::Water,
        BlockType::Sand,
        BlockType::Log,
        BlockType::Leaves,
        BlockType::WoodPlanks,
        BlockType::WoodStairs,
        BlockType::Fence,
        BlockType::Glass,
        BlockType::RedstoneLampLit,
        BlockType::WoodDoor,
        BlockType::Cobblestone,
    ] {
        assert!(count(&counts, t) > 0, "no {t:?} in the world");
    }
    assert!(count(&counts, BlockType::DoubleWoodSlab) > 0);
}

#[test]
fn the_landmarks_are_all_built() {
    let scene = WorldScene::new();
    // 19. house
    assert!(!scene.house().floor.is_empty() && !scene.house().walls.is_empty());
    assert!(!scene.house_exterior().roof.is_empty());
    assert_eq!(scene.house_exterior().door.len(), 2);
    assert_eq!(scene.house_exterior().lamps.len(), 2);
    // 20. pond
    assert!(!scene.pond().water.is_empty() && !scene.pond().sand.is_empty());
    // 21. 3-5 trees
    assert!((3..=5).contains(&scene.trees().len()));
    // 22. path
    assert!(!scene.path().is_empty());
    let end = scene.path().end().unwrap();
    assert_eq!((end.x, end.z), DESCENT_ENDPOINT);
}

#[test]
fn daylight_and_sunlight_are_active() {
    // 23. daylight background
    let background = world_background();
    assert!(background.is_sky());
    let up = background.color(Vec3::new(0.0, 1.0, 0.0));
    assert!(up.b > up.r && up.b > 0.6);
    assert_ne!(background, Background::Solid(gallery_background()));
    assert_eq!(background, Background::Sky(world_sky()));
    // 24. directional sunlight
    let lights = world_lights();
    let sun = lights.iter().find_map(|l| match l {
        Light::Directional(d) => Some(d),
        Light::Point(_) => None,
    });
    let sun = sun.expect("a sun");
    assert!(sun.direction.y > 0.5 && sun.intensity > 0.0);
}

#[test]
fn nothing_of_the_later_gates_is_built() {
    let scene = WorldScene::new();
    let counts = census(&scene);
    for (t, n) in &counts {
        // 25. no Red-Black, 26. no portal
        assert!(
            matches!(
                t.family(),
                BlockFamily::OverworldTerrain | BlockFamily::OverworldArchitecture
            ),
            "{t:?} x{n} does not belong to the Overworld"
        );
    }
    assert_eq!(count(&counts, BlockType::PortalFrameRedObsidian), 0);
    assert_eq!(count(&counts, BlockType::PortalCoreDarkCrimson), 0);
    // Whatever continues under the terrain bottom (the diamond's geological
    // taper of Gate 13) is Overworld terrain, checked above; no inverted
    // Red-Black geometry exists in the Overworld half.
    let b = scene.bounds();
    let c = *scene.config();
    for z in 0..c.depth {
        for x in 0..c.width {
            if let Some(block) = scene.world().get(IVec3::new(x, b.min.y - 1, z)) {
                assert!(matches!(
                    block.block_type().family(),
                    BlockFamily::OverworldTerrain | BlockFamily::OverworldArchitecture
                ));
            }
        }
    }
}

#[test]
fn the_official_seed_rebuilds_the_identical_world() {
    // 27. determinism
    assert_eq!(TerrainConfig::official().seed, OFFICIAL_SEED);
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.world().len(), b.world().len());
    assert_eq!(census(&a), census(&b));
    assert_eq!(a.pond(), b.pond());
    assert_eq!(a.trees(), b.trees());
    assert_eq!(a.house(), b.house());
    assert_eq!(a.house_exterior(), b.house_exterior());
    assert_eq!(a.path(), b.path());
    let ca = world_camera(4.0 / 3.0);
    let cb = world_camera(4.0 / 3.0);
    assert_eq!((ca.position, ca.target), (cb.position, cb.target));
}

#[test]
fn the_catalog_still_offers_all_official_blocks() {
    // 28. catalog 39/39
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    assert_eq!(official_entries().len(), 39);
    let catalog = CatalogScene::new();
    assert_eq!(catalog.len(), 41);
    assert_eq!(
        catalog.entries().iter().filter(|e| e.is_official()).count(),
        39
    );
}

#[test]
fn gate12_metrics() {
    let scene = WorldScene::new();
    let c = *scene.config();
    let counts = census(&scene);
    let (min, max) = surface_range(&scene);
    let house_blocks = scene.house().floor.len()
        + scene.house().walls.len()
        + scene.house().corners.len()
        + scene.house_exterior().roof.len()
        + scene.house_exterior().door.len()
        + scene.house_exterior().glass.len()
        + scene.house_exterior().fence.len()
        + scene.house_exterior().lamps.len();
    let columns = (0..c.depth)
        .flat_map(|z| (0..c.width).map(move |x| (x, z)))
        .filter(|&(x, z)| footprint_contains(&c, x, z))
        .count();
    let lines = [
        format!(
            "terrain dimensions: {} x {} (footprint columns: {})",
            c.width, c.depth, columns
        ),
        format!("voxel count: {}", scene.world().len()),
        format!("min surface height: {min}"),
        format!("max surface height: {max}"),
        format!("relief: {}", max - min),
        format!("Grass count: {}", count(&counts, BlockType::Grass)),
        format!("Dirt count: {}", count(&counts, BlockType::Dirt)),
        format!("Stone count: {}", count(&counts, BlockType::Stone)),
        format!("Deepslate count: {}", count(&counts, BlockType::Deepslate)),
        format!("Sand count: {}", count(&counts, BlockType::Sand)),
        format!("Water count: {}", count(&counts, BlockType::Water)),
        format!("tree count: {}", scene.trees().len()),
        format!("house block count: {house_blocks}"),
        format!(
            "path length: {} (+{} landing)",
            scene.path().len(),
            scene.path().landing.len()
        ),
        format!(
            "pond basin columns: {} at water level {}",
            pond_basin_columns().len(),
            POND_WATER_LEVEL
        ),
        format!(
            "house footprint: {} x {}",
            HOUSE_FOOTPRINT.width(),
            HOUSE_FOOTPRINT.depth()
        ),
    ];
    for line in &lines {
        println!("GATE12 {line}");
    }
    assert!(columns >= 16 * 16);
    assert!(scene.world().len() > 2000);
    assert!(max - min >= 2);
    assert!(house_blocks > 100 && house_blocks < 400);
}
