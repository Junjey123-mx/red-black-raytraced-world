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
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
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
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_collision::{
    CameraCollisionConfig, is_camera_passable, is_camera_solid, is_position_clear,
};
use core::hit::Face;
use core::math::{IVec3, Vec3};
use scene::block::BlockInstance;
use scene::block_shape_factory::{block_geometry, canonical_geometry};
use scene::block_type::BlockType;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    OverworldBlockTextures, fence_material_id, insert_overworld_materials, wood_planks_material_id,
};
use scene::red_black_timber::{red_black_fence_material_id, red_black_wood_planks_material_id};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const ORIENTATIONS: [Orientation; 6] = [
    Orientation::Up,
    Orientation::Down,
    Orientation::North,
    Orientation::South,
    Orientation::East,
    Orientation::West,
];

fn env() -> MaterialLibrary {
    let mut manager = TextureManager::new();
    let blocks = OverworldBlockTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &blocks);
    library
}

// 1
#[test]
fn the_fence_geometry_is_the_same() {
    for o in ORIENTATIONS {
        assert_eq!(
            block_geometry(BlockType::RedBlackFence, o),
            block_geometry(BlockType::Fence, o),
            "{o:?}"
        );
    }
    assert_eq!(
        canonical_geometry(BlockType::RedBlackFence),
        canonical_geometry(BlockType::Fence)
    );
}

// 2
#[test]
fn the_fence_is_camera_solid() {
    let block = BlockInstance::new(
        BlockType::RedBlackFence,
        red_black_fence_material_id(),
        Orientation::Down,
    );
    assert!(is_camera_solid(&block));
    assert!(!is_camera_passable(BlockType::RedBlackFence));
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), block);
    // Its post blocks the camera, the air beside the rails does not.
    let cfg = CameraCollisionConfig::default();
    assert!(!is_position_clear(
        &world,
        Vec3::new(0.5, 0.5, 0.5),
        &cfg,
        &is_camera_solid
    ));
    assert!(is_position_clear(
        &world,
        Vec3::new(0.5, 0.5, 2.0),
        &cfg,
        &is_camera_solid
    ));
}

// 3
#[test]
fn the_fence_wears_the_corinto_planks() {
    let library = env();
    let fence = library
        .get(red_black_fence_material_id())
        .expect("RedBlackFence material");
    let planks = library.get(red_black_wood_planks_material_id()).unwrap();
    for f in [Face::PositiveX, Face::PositiveY, Face::NegativeZ] {
        assert_eq!(
            fence.face_textures.unwrap().texture_for_face(f),
            planks.face_textures.unwrap().texture_for_face(f)
        );
    }
    let overworld = library.get(fence_material_id()).unwrap();
    assert_eq!(
        (fence.specular, fence.shininess, fence.reflectivity),
        (
            overworld.specular,
            overworld.shininess,
            overworld.reflectivity
        )
    );
    assert_eq!(fence.alpha_mode, overworld.alpha_mode);
}

// 4
#[test]
fn the_fence_is_not_emissive() {
    let library = env();
    let m = library.get(red_black_fence_material_id()).unwrap();
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.emission_strength, 0.0);
    assert_eq!(m.transparency, 0.0);
}

// 5
#[test]
fn the_overworld_fence_is_unchanged() {
    let library = env();
    let fence = library.get(fence_material_id()).unwrap();
    let planks = library.get(wood_planks_material_id()).unwrap();
    assert_eq!(
        fence.face_textures, planks.face_textures,
        "the Overworld fence still wears the Overworld planks"
    );
    assert!((fence.albedo.r - 0.63).abs() < 1e-6);
    assert_ne!(fence_material_id(), red_black_fence_material_id());
}

// 6
#[test]
fn the_shape_is_a_partial_post_and_rails() {
    for o in ORIENTATIONS {
        let g = block_geometry(BlockType::RedBlackFence, o);
        assert!(!g.is_full_cube());
        assert_eq!(g.part_count(), 3, "post and two rails");
        for part in g.parts() {
            assert!(part.is_within_unit_cell());
        }
    }
    let src = std::fs::read_to_string("src/scene/block_shape_factory.rs").unwrap();
    assert_eq!(
        src.matches("fn fence(").count(),
        1,
        "no parallel fence builder"
    );
}
