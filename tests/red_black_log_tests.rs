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

use core::hit::Face;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    OverworldBlockTextures, insert_overworld_materials, log_material_id,
};
use scene::red_black_timber::{RED_BLACK_TIMBER_DIR, red_black_log_material_id};
use scene::texture_manager::TextureManager;

const FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];

fn env() -> (TextureManager, MaterialLibrary) {
    let mut manager = TextureManager::new();
    let blocks = OverworldBlockTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &blocks);
    (manager, library)
}

// 1
#[test]
fn the_log_geometry_is_reused() {
    for o in [
        Orientation::Up,
        Orientation::Down,
        Orientation::North,
        Orientation::East,
    ] {
        assert_eq!(
            block_geometry(BlockType::RedBlackLog, o),
            block_geometry(BlockType::Log, o)
        );
        assert!(block_geometry(BlockType::RedBlackLog, o).is_full_cube());
    }
}

// 2
#[test]
fn end_grain_and_bark_are_different_faces() {
    let (mut manager, library) = env();
    let faces = library
        .get(red_black_log_material_id())
        .unwrap()
        .face_textures
        .unwrap();
    let end = faces.texture_for_face(Face::PositiveY);
    let side = faces.texture_for_face(Face::PositiveX);
    assert_ne!(end, side);
    assert_eq!(faces.texture_for_face(Face::NegativeY), end);
    for f in [
        Face::PositiveX,
        Face::NegativeX,
        Face::PositiveZ,
        Face::NegativeZ,
    ] {
        assert_eq!(faces.texture_for_face(f), side, "{f:?}");
    }
    assert_eq!(
        end,
        manager
            .load(format!("{RED_BLACK_TIMBER_DIR}/log/end.png"))
            .unwrap()
    );
    assert_eq!(
        side,
        manager
            .load(format!("{RED_BLACK_TIMBER_DIR}/log/side.png"))
            .unwrap()
    );
    // The rings read lighter than the bark.
    let mean = |id| {
        let t = manager.get(id).unwrap();
        t.pixels().iter().map(|c| c.r + c.g + c.b).sum::<f32>() / t.pixels().len() as f32
    };
    assert!(mean(end) > mean(side) * 1.4);
}

// 3
#[test]
fn the_log_is_not_emissive() {
    let (_, library) = env();
    let m = library.get(red_black_log_material_id()).unwrap();
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.emission_strength, 0.0);
    assert_eq!(m.transparency, 0.0);
}

// 4
#[test]
fn the_uv_layout_matches_the_overworld_log() {
    let (_, library) = env();
    let ours = library.get(red_black_log_material_id()).unwrap();
    let theirs = library.get(log_material_id()).unwrap();
    let ours_f = ours.face_textures.unwrap();
    let theirs_f = theirs.face_textures.unwrap();
    for a in FACES {
        for b in FACES {
            assert_eq!(
                ours_f.texture_for_face(a) == ours_f.texture_for_face(b),
                theirs_f.texture_for_face(a) == theirs_f.texture_for_face(b),
                "{a:?} {b:?}"
            );
        }
    }
    assert_eq!(ours.alpha_mode, theirs.alpha_mode);
    assert_eq!(
        (ours.specular, ours.shininess, ours.reflectivity),
        (theirs.specular, theirs.shininess, theirs.reflectivity)
    );
    // Dark corinto, not the Overworld brown.
    assert!(ours.albedo.r > ours.albedo.g * 3.0);
    assert!(
        ours.albedo.r + ours.albedo.g + ours.albedo.b
            < theirs.albedo.r + theirs.albedo.g + theirs.albedo.b
    );
}

// 5
#[test]
fn the_overworld_log_is_unchanged() {
    let (_, library) = env();
    let log = library.get(log_material_id()).unwrap();
    assert!((log.albedo.r - 0.40).abs() < 1e-6 && (log.albedo.g - 0.31).abs() < 1e-6);
    assert_ne!(log_material_id(), red_black_log_material_id());
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "807e490",
            "HEAD",
            "--",
            "assets/textures/overworld/log",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
        }
    }
}

// 6
#[test]
fn the_textures_are_sixteen_by_sixteen_and_opaque() {
    let mut m = TextureManager::new();
    for f in ["log/end.png", "log/side.png"] {
        let id = m.load(format!("{RED_BLACK_TIMBER_DIR}/{f}")).unwrap();
        let t = m.get(id).unwrap();
        assert_eq!((t.width(), t.height()), (16, 16), "{f}");
        assert!(t.pixels().iter().all(|c| c.a > 0.99), "{f}");
        // Red-violet: red dominates green, blue tints the grooves.
        let (r, g, b) = t
            .pixels()
            .iter()
            .fold((0.0, 0.0, 0.0), |a, c| (a.0 + c.r, a.1 + c.g, a.2 + c.b));
        assert!(r > g * 2.5 && b > g, "{f}: {r} {g} {b}");
    }
}
