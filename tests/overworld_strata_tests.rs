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
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
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
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
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

use core::math::IVec3;
use scene::block_type::BlockType;
use scene::overworld_blocks::{deepslate_material_id, dirt_material_id, stone_block_material_id};
use scene::scene::grass_material_id;
use scene::terrain::TerrainConfig;
use scene::terrain::fbm::HeightField;
use scene::terrain::generator::{
    MIN_DIRT_DEPTH, column_bottom, column_surface, dirt_depth, footprint_contains, stratum,
};
use scene::terrain::generator::{TerrainBounds, generate_terrain, terrain_bounds};
use scene::voxel_world::VoxelWorld;
use std::collections::{BTreeMap, HashMap};

fn official() -> TerrainConfig {
    TerrainConfig::official()
}

/// The raw generated terrain (before the micro-scene features are built on
/// it), exposed with the same accessors the scene offers.
struct Terrain {
    config: TerrainConfig,
    field: HeightField,
    bounds: TerrainBounds,
    world: VoxelWorld,
}

impl Terrain {
    fn new() -> Self {
        let config = TerrainConfig::official();
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

    fn config(&self) -> &TerrainConfig {
        &self.config
    }

    fn height_field(&self) -> &HeightField {
        &self.field
    }

    fn bounds(&self) -> TerrainBounds {
        self.bounds
    }

    fn world(&self) -> &VoxelWorld {
        &self.world
    }
}

fn columns(scene: &Terrain) -> Vec<(i32, i32, i32, i32)> {
    let c = *scene.config();
    let mut out = Vec::new();
    for z in 0..c.depth {
        for x in 0..c.width {
            if footprint_contains(&c, x, z) {
                let surface = column_surface(&c, scene.height_field(), x, z).unwrap();
                out.push((x, z, column_bottom(&c, x, z), surface));
            }
        }
    }
    out
}

fn block_at(scene: &Terrain, x: i32, y: i32, z: i32) -> BlockType {
    scene.world().get(IVec3::new(x, y, z)).unwrap().block_type()
}

#[test]
fn grass_is_only_ever_the_surface_block() {
    let scene = Terrain::new();
    for (x, z, bottom, surface) in columns(&scene) {
        assert_eq!(block_at(&scene, x, surface, z), BlockType::Grass);
        for y in bottom..surface {
            assert_ne!(
                block_at(&scene, x, y, z),
                BlockType::Grass,
                "buried grass at ({x}, {y}, {z})"
            );
        }
    }
}

#[test]
fn dirt_lies_directly_under_the_grass_with_a_valid_thickness() {
    let scene = Terrain::new();
    let c = official();
    for (x, z, bottom, surface) in columns(&scene) {
        let depth = dirt_depth(&c, x, z);
        assert!((MIN_DIRT_DEPTH..=c.dirt_depth).contains(&depth));
        for y in (surface - depth).max(bottom)..surface {
            assert_eq!(
                block_at(&scene, x, y, z),
                BlockType::Dirt,
                "at ({x}, {y}, {z})"
            );
        }
        // Below the soil band there is never dirt again.
        for y in bottom..(surface - depth) {
            assert_ne!(block_at(&scene, x, y, z), BlockType::Dirt);
        }
    }
}

#[test]
fn the_soil_thickness_varies_deterministically_between_two_and_three() {
    let c = official();
    let mut seen = BTreeMap::new();
    for z in 0..c.depth {
        for x in 0..c.width {
            *seen.entry(dirt_depth(&c, x, z)).or_insert(0) += 1;
            assert_eq!(dirt_depth(&c, x, z), dirt_depth(&c, x, z));
        }
    }
    assert_eq!(seen.keys().copied().collect::<Vec<_>>(), vec![2, 3]);
    assert!(seen[&2] > 50 && seen[&3] > 50, "{seen:?}");
}

#[test]
fn stone_lies_under_the_dirt_and_deepslate_lies_at_depth() {
    let scene = Terrain::new();
    let c = official();
    let mut stone_columns = 0;
    for (x, z, bottom, surface) in columns(&scene) {
        let soil_floor = surface - dirt_depth(&c, x, z);
        for y in bottom..soil_floor {
            let expected = if y <= c.deepslate_level {
                BlockType::Deepslate
            } else {
                BlockType::Stone
            };
            assert_eq!(block_at(&scene, x, y, z), expected, "at ({x}, {y}, {z})");
        }
        if soil_floor - 1 > c.deepslate_level {
            stone_columns += 1;
            assert_eq!(block_at(&scene, x, soil_floor - 1, z), BlockType::Stone);
        }
    }
    assert!(
        stone_columns > 200,
        "{stone_columns} columns with stone under the soil"
    );
    // The center of the mass reaches its deepest cells, and they are deepslate.
    let (bottom, surface) = (
        column_bottom(&c, 12, 12),
        column_surface(&c, scene.height_field(), 12, 12).unwrap(),
    );
    assert!(bottom <= c.deepslate_level);
    assert_eq!(block_at(&scene, 12, bottom, 12), BlockType::Deepslate);
    assert_eq!(stratum(&c, 12, 12, surface, surface), BlockType::Grass);
}

#[test]
fn deepslate_never_sits_above_dirt_or_grass() {
    let scene = Terrain::new();
    for (x, z, bottom, surface) in columns(&scene) {
        let column: Vec<BlockType> = (bottom..=surface)
            .map(|y| block_at(&scene, x, y, z))
            .collect();
        // Deepslate only at the bottom: once it stops, it never resumes.
        let deep = column
            .iter()
            .take_while(|b| **b == BlockType::Deepslate)
            .count();
        assert!(
            column[deep..].iter().all(|b| *b != BlockType::Deepslate),
            "({x}, {z}): {column:?}"
        );
        // Nothing soft ever lies under deepslate.
        for pair in column.windows(2) {
            if pair[1] == BlockType::Deepslate {
                assert_eq!(pair[0], BlockType::Deepslate, "({x}, {z}): {column:?}");
            }
        }
    }
}

#[test]
fn all_four_strata_are_present_with_their_definitive_materials() {
    let scene = Terrain::new();
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for (x, z, bottom, surface) in columns(&scene) {
        for y in bottom..=surface {
            let block = scene.world().get(IVec3::new(x, y, z)).unwrap();
            let expected = match block.block_type() {
                BlockType::Grass => grass_material_id(),
                BlockType::Dirt => dirt_material_id(),
                BlockType::Stone => stone_block_material_id(),
                BlockType::Deepslate => deepslate_material_id(),
                other => panic!("{other:?} is not a terrain stratum"),
            };
            assert_eq!(block.material_id(), expected);
            *counts.entry(block.block_type()).or_insert(0) += 1;
        }
    }
    assert_eq!(counts.len(), 4, "{counts:?}");
    for b in [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Deepslate,
    ] {
        assert!(counts[&b] > 100, "{b:?}: {counts:?}");
    }
    // Deepslate bricks are architecture, never geology.
    assert!(!counts.contains_key(&BlockType::DeepslateBricks));
}

#[test]
fn columns_stay_solid_without_accidental_holes() {
    let scene = Terrain::new();
    let mut total = 0;
    for (x, z, bottom, surface) in columns(&scene) {
        for y in bottom..=surface {
            assert!(
                scene.world().contains(IVec3::new(x, y, z)),
                "hole at ({x}, {y}, {z})"
            );
            total += 1;
        }
    }
    assert_eq!(total, scene.world().len());
}

#[test]
fn the_strata_are_deterministic() {
    let a = Terrain::new();
    let b = Terrain::new();
    for (x, z, bottom, surface) in columns(&a) {
        for y in bottom..=surface {
            assert_eq!(block_at(&a, x, y, z), block_at(&b, x, y, z));
        }
    }
}
