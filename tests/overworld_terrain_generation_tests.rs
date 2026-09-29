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
use scene::block_type::{BlockFamily, BlockType};
use scene::terrain::TerrainConfig;
use scene::terrain::fbm::HeightField;
use scene::terrain::generator::{
    FOOTPRINT_EXPONENT, column_bottom, column_surface, footprint_contains, footprint_radius,
    mass_bottom,
};
use scene::terrain::generator::{TerrainBounds, generate_terrain, terrain_bounds};
use scene::voxel_world::VoxelWorld;

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

fn columns(scene: &Terrain) -> Vec<(i32, i32)> {
    let c = scene.config();
    (0..c.depth)
        .flat_map(|z| (0..c.width).map(move |x| (x, z)))
        .filter(|&(x, z)| footprint_contains(&official(), x, z))
        .collect()
}

#[test]
fn the_world_is_no_longer_empty() {
    let terrain = Terrain::new();
    assert!(!terrain.world().is_empty());
    assert!(
        terrain.world().len() > 1000,
        "{} voxels",
        terrain.world().len()
    );
    // The scene is built on exactly this terrain.
    let scene = scene::world::WorldScene::new();
    assert_eq!(scene.height_field(), terrain.height_field());
    assert_eq!(scene.bounds(), terrain.bounds());
    assert!(scene.world().len() > 1000);
}

#[test]
fn the_footprint_exceeds_sixteen_by_sixteen() {
    let scene = Terrain::new();
    let bounds = scene.bounds();
    assert!(bounds.max.x - bounds.min.x + 1 >= 16);
    assert!(bounds.max.z - bounds.min.z + 1 >= 16);
    assert_eq!((bounds.min.x, bounds.min.z), (0, 0));
    assert_eq!((bounds.max.x, bounds.max.z), (23, 23));
    // More occupied columns than a full 16 x 16 plate.
    assert!(
        columns(&scene).len() >= 16 * 16,
        "{} columns",
        columns(&scene).len()
    );
}

#[test]
fn the_voxel_count_is_deterministic() {
    let a = Terrain::new();
    let b = Terrain::new();
    assert_eq!(a.world().len(), b.world().len());
    assert_eq!(a.height_field(), b.height_field());
    for (x, z) in columns(&a) {
        for y in a.bounds().min.y..=a.bounds().max.y {
            let cell = IVec3::new(x, y, z);
            assert_eq!(a.world().get(cell), b.world().get(cell));
        }
    }
}

#[test]
fn the_surface_has_several_distinct_heights() {
    let scene = Terrain::new();
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    for (x, z) in columns(&scene) {
        let s = column_surface(scene.config(), scene.height_field(), x, z).unwrap();
        min = min.min(s);
        max = max.max(s);
        assert!(scene.world().contains(IVec3::new(x, s, z)));
        assert!(!scene.world().contains(IVec3::new(x, s + 1, z)));
    }
    assert!(max - min >= 2, "relief {min}..{max}");
}

#[test]
fn the_center_is_occupied_and_the_corners_are_cut_away() {
    let scene = Terrain::new();
    let c = official();
    for (x, z) in [(11, 11), (12, 12), (8, 15), (15, 8)] {
        assert!(footprint_contains(&c, x, z));
        let s = column_surface(&c, scene.height_field(), x, z).unwrap();
        assert!(scene.world().contains(IVec3::new(x, s, z)));
    }
    for (x, z) in [(0, 0), (23, 0), (0, 23), (23, 23), (1, 1), (22, 22)] {
        assert!(!footprint_contains(&c, x, z), "corner ({x}, {z}) kept");
        for y in scene.bounds().min.y..=scene.bounds().max.y {
            assert!(!scene.world().contains(IVec3::new(x, y, z)));
        }
    }
    // Mid-edge columns survive: the mask trims corners, not whole sides.
    for (x, z) in [(12, 0), (12, 23), (0, 12), (23, 12)] {
        assert!(footprint_contains(&c, x, z));
    }
    assert!(footprint_radius(&c, 12, 12) < 0.1);
    assert!(footprint_radius(&c, 0, 0) > 1.0);
    assert!(FOOTPRINT_EXPONENT > 1.0);
}

#[test]
fn the_edges_are_softened_not_a_flat_slab() {
    let scene = Terrain::new();
    let c = official();
    // The underside rises toward the rim and the rim surface dips.
    assert_eq!(column_bottom(&c, 12, 12), mass_bottom(&c));
    assert!(column_bottom(&c, 12, 0) > mass_bottom(&c));
    let center_surface = column_surface(&c, scene.height_field(), 12, 12).unwrap();
    let rim_surface = column_surface(&c, scene.height_field(), 12, 0).unwrap();
    assert!(rim_surface <= scene.height_field().height(12, 0).unwrap());
    assert!(center_surface >= c.min_surface_height());
    // Not every column reaches the same bottom.
    let bottoms: std::collections::BTreeSet<i32> = columns(&scene)
        .iter()
        .map(|&(x, z)| column_bottom(&c, x, z))
        .collect();
    assert!(bottoms.len() >= 2, "{bottoms:?}");
}

#[test]
fn no_voxel_lies_outside_the_configured_range_and_columns_are_solid() {
    let scene = Terrain::new();
    let c = official();
    let bounds = scene.bounds();
    assert_eq!(bounds.min.y, mass_bottom(&c));
    assert!(bounds.max.y <= c.max_surface_height());
    let mut counted = 0;
    for (x, z) in columns(&scene) {
        let bottom = column_bottom(&c, x, z);
        let surface = column_surface(&c, scene.height_field(), x, z).unwrap();
        for y in bottom..=surface {
            assert!(
                scene.world().contains(IVec3::new(x, y, z)),
                "hole at ({x}, {y}, {z})"
            );
            counted += 1;
        }
        assert!(!scene.world().contains(IVec3::new(x, bottom - 1, z)));
    }
    // Every voxel belongs to a column: nothing floats elsewhere.
    assert_eq!(counted, scene.world().len());
}

#[test]
fn the_terrain_uses_only_overworld_blocks_and_no_red_black_ones() {
    let scene = Terrain::new();
    let c = official();
    for (x, z) in columns(&scene) {
        let surface = column_surface(&c, scene.height_field(), x, z).unwrap();
        for y in column_bottom(&c, x, z)..=surface {
            let block = scene.world().get(IVec3::new(x, y, z)).unwrap();
            let family = block.block_type().family();
            assert!(
                matches!(family, BlockFamily::OverworldTerrain),
                "{:?} at ({x}, {y}, {z})",
                block.block_type()
            );
            if y == surface {
                assert_eq!(block.block_type(), BlockType::Grass);
            }
        }
    }
}
