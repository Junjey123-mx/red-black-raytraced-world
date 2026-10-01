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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
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
    resolve_camera_motion,
};
use core::hit::Face;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, nearest_visible_hit};
use renderer::skybox::Background;
use scene::block::BlockInstance;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    OverworldBlockTextures, insert_overworld_materials, wood_door_bottom_material_id,
    wood_door_top_material_id,
};
use scene::red_black_timber::{
    red_black_wood_door_bottom_material_id, red_black_wood_door_top_material_id,
};
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

fn env() -> (TextureManager, MaterialLibrary) {
    let mut manager = TextureManager::new();
    let blocks = OverworldBlockTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &blocks);
    (manager, library)
}

fn door_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();
    let o = Orientation::West;
    world.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::RedBlackWoodDoor,
            red_black_wood_door_bottom_material_id(),
            o,
        ),
    );
    world.insert(
        IVec3::new(0, 1, 0),
        BlockInstance::new(
            BlockType::RedBlackWoodDoor,
            red_black_wood_door_top_material_id(),
            o,
        ),
    );
    // Stone jambs on both sides.
    for y in 0..2 {
        for z in [-1, 1] {
            world.insert(
                IVec3::new(0, y, z),
                BlockInstance::new(
                    BlockType::Stone,
                    scene::overworld_blocks::stone_block_material_id(),
                    Orientation::Up,
                ),
            );
        }
    }
    world
}

// 1
#[test]
fn the_door_geometry_is_reused() {
    for o in ORIENTATIONS {
        assert_eq!(
            block_geometry(BlockType::RedBlackWoodDoor, o),
            block_geometry(BlockType::WoodDoor, o),
            "{o:?}"
        );
    }
}

// 2
#[test]
fn the_door_is_a_thin_partial_panel() {
    for o in ORIENTATIONS {
        let g = block_geometry(BlockType::RedBlackWoodDoor, o);
        assert!(!g.is_full_cube());
        assert_eq!(g.part_count(), 1);
        let p = g.parts()[0];
        let size = p.max() - p.min();
        let thin = size.x.min(size.y).min(size.z);
        assert!((thin - 0.1875).abs() < 1e-5, "{o:?}: {size:?}");
    }
}

// 3
#[test]
fn the_door_is_camera_passable() {
    assert!(is_camera_passable(BlockType::RedBlackWoodDoor));
    let block = BlockInstance::new(
        BlockType::RedBlackWoodDoor,
        red_black_wood_door_bottom_material_id(),
        Orientation::West,
    );
    assert!(!is_camera_solid(&block));
    // Straight through the doorway, both ways.
    let world = door_world();
    let cfg = CameraCollisionConfig::default();
    for (from, to) in [
        (Vec3::new(-1.5, 1.0, 0.5), Vec3::new(1.5, 1.0, 0.5)),
        (Vec3::new(1.5, 1.0, 0.5), Vec3::new(-1.5, 1.0, 0.5)),
    ] {
        let reached = resolve_camera_motion(&world, from, to - from, &cfg, &is_camera_solid);
        assert!((reached - to).length() < 1e-3, "{reached:?}");
    }
    assert!(is_position_clear(
        &world,
        Vec3::new(0.5, 1.0, 0.5),
        &cfg,
        &is_camera_solid
    ));
}

// 4
#[test]
fn the_door_is_hit_by_rays() {
    let (manager, library) = env();
    let world = door_world();
    let lights = Vec::new();
    let view = VoxelScene {
        world: &world,
        materials: &library,
        camera_position: Vec3::new(-3.0, 0.5, 0.5),
        lights: &lights,
        ambient_factor: 0.3,
        background: Background::Solid(core::color::Color::new(0.0, 0.0, 0.0, 1.0)),
        texture_manager: &manager,
        max_distance: 50.0,
    };
    // A ray at the solid lower panel stops on the door, not behind it.
    let ray = Ray::new(Vec3::new(-3.0, 0.3, 0.5), Vec3::new(1.0, 0.0, 0.0));
    let hit = nearest_visible_hit(&view, &ray, 50.0).expect("the door is visible");
    assert_eq!(hit.cell, IVec3::new(0, 0, 0));
    assert_eq!(hit.block.block_type(), BlockType::RedBlackWoodDoor);
}

// 5
#[test]
fn the_door_is_not_emissive() {
    let (_, library) = env();
    for id in [
        red_black_wood_door_bottom_material_id(),
        red_black_wood_door_top_material_id(),
    ] {
        let m = library.get(id).expect("door material");
        assert!(m.emissive_texture.is_none());
        assert_eq!(m.emission_strength, 0.0);
    }
}

// 6
#[test]
fn the_overworld_door_is_unchanged() {
    let (_, library) = env();
    for (ours, theirs) in [
        (
            red_black_wood_door_bottom_material_id(),
            wood_door_bottom_material_id(),
        ),
        (
            red_black_wood_door_top_material_id(),
            wood_door_top_material_id(),
        ),
    ] {
        let a = library.get(ours).unwrap();
        let b = library.get(theirs).unwrap();
        assert_ne!(ours, theirs);
        assert_eq!(a.alpha_mode, b.alpha_mode, "same window cutout");
        assert_eq!(
            (a.specular, a.shininess, a.reflectivity),
            (b.specular, b.shininess, b.reflectivity)
        );
        assert!((b.albedo.r - 0.55).abs() < 1e-6, "Overworld door profile");
        // Same face layout: broad faces on +/-Z, edges elsewhere.
        let (fa, fb) = (a.face_textures.unwrap(), b.face_textures.unwrap());
        assert_eq!(
            fa.texture_for_face(Face::PositiveZ) == fa.texture_for_face(Face::PositiveX),
            fb.texture_for_face(Face::PositiveZ) == fb.texture_for_face(Face::PositiveX)
        );
    }
    assert!(is_camera_passable(BlockType::WoodDoor));
}

// 7
#[test]
fn the_surrounding_walls_stay_solid() {
    let world = door_world();
    let cfg = CameraCollisionConfig::default();
    // Into a jamb: stopped.
    let from = Vec3::new(-1.5, 1.0, 1.5);
    let to = Vec3::new(1.5, 1.0, 1.5);
    let reached = resolve_camera_motion(&world, from, to - from, &cfg, &is_camera_solid);
    assert!(reached.x < -0.2, "{reached:?}");
    for t in [
        BlockType::Stone,
        BlockType::PolishedBlackstoneBricks,
        BlockType::RedBlackFence,
        BlockType::RedBlackWoodPlanks,
        BlockType::RedBlackLog,
        BlockType::RedBlackLeaves,
        BlockType::Glass,
        BlockType::PortalFrameRedObsidian,
    ] {
        assert!(!is_camera_passable(t), "{t:?}");
    }
}

// 8
#[test]
fn the_collision_contract_is_explicit() {
    let src = std::fs::read_to_string("src/camera/world_collision.rs").unwrap();
    let start = src.find("pub fn is_camera_passable").unwrap();
    let body = &src[start..start + src[start..].find("\n}").unwrap()];
    for t in ["PortalCoreDarkCrimson", "WoodDoor", "RedBlackWoodDoor"] {
        assert!(body.contains(&format!("BlockType::{t}")), "{t}");
    }
    assert_eq!(
        body.matches("BlockType::").count(),
        3,
        "exactly three passable blocks:\n{body}"
    );
    let passable: std::collections::HashSet<BlockType> = BlockType::ALL
        .iter()
        .copied()
        .chain([BlockType::RedBlackWoodDoor])
        .filter(|t| is_camera_passable(*t))
        .collect();
    assert!(passable.contains(&BlockType::RedBlackWoodDoor));
    assert!(passable.len() <= 3);
    let fortress = std::fs::read_to_string("src/scene/red_black_timber.rs").unwrap();
    assert!(
        !fortress.contains("Portal"),
        "the door borrows no portal semantics"
    );
}
