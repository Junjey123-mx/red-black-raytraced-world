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
use core::material::AlphaMode;
use core::math::Vec2;
use renderer::texture_sampling::sample_nearest;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::material_gallery::{GalleryTextures, advanced_materials_library, leaves_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{OverworldBlockTextures, insert_overworld_materials};
use scene::red_black_timber::{RED_BLACK_TIMBER_DIR, red_black_leaves_material_id};
use scene::texture_manager::TextureManager;

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
    gallery: MaterialLibrary,
}

fn env() -> Env {
    let mut manager = TextureManager::new();
    let blocks = OverworldBlockTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let gallery_tex = GalleryTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
    )
    .unwrap();
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &blocks);
    let gallery = advanced_materials_library(&gallery_tex);
    Env {
        manager,
        library,
        gallery,
    }
}

fn alpha_mask(path: &str) -> Vec<u8> {
    let mut m = TextureManager::new();
    let id = m.load(path).unwrap();
    let t = m.get(id).unwrap();
    t.pixels()
        .iter()
        .map(|c| if c.a > 0.5 { 1 } else { 0 })
        .collect()
}

// 1
#[test]
fn the_material_exists() {
    let e = env();
    let m = e
        .library
        .get(red_black_leaves_material_id())
        .expect("RedBlackLeaves material");
    let faces = m.face_textures.expect("albedo");
    let t = e
        .manager
        .get(faces.texture_for_face(Face::PositiveY))
        .unwrap();
    assert!(t.width() > 0);
    // The albedo base is a dark crimson, not the Overworld green.
    assert!(m.albedo.r > m.albedo.g * 2.0 && m.albedo.r > m.albedo.b);
}

// 2
#[test]
fn the_geometry_matches_leaves() {
    for o in [
        Orientation::Up,
        Orientation::Down,
        Orientation::South,
        Orientation::West,
    ] {
        assert_eq!(
            block_geometry(BlockType::RedBlackLeaves, o),
            block_geometry(BlockType::Leaves, o)
        );
    }
    assert!(block_geometry(BlockType::RedBlackLeaves, Orientation::Down).is_full_cube());
}

// 3
#[test]
fn the_cutout_holes_match_the_leaf_mask() {
    let ours = alpha_mask(&format!("{RED_BLACK_TIMBER_DIR}/leaves.png"));
    let theirs = alpha_mask("assets/textures/overworld/leaves.png");
    assert_eq!(
        ours, theirs,
        "the foliage keeps the leaf silhouette texel for texel"
    );
    let holes = ours.iter().filter(|a| **a == 0).count();
    assert!(holes > 40 && holes < 200, "{holes}");
}

// 4
#[test]
fn the_foliage_is_not_emissive() {
    let e = env();
    let m = e.library.get(red_black_leaves_material_id()).unwrap();
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.emission_strength, 0.0);
    assert_eq!(m.transparency, 0.0);
    assert!(m.reflectivity <= 0.01);
}

// 5
#[test]
fn the_alpha_contract_is_cutout_like_the_original() {
    let e = env();
    let ours = e.library.get(red_black_leaves_material_id()).unwrap();
    let theirs = e.gallery.get(leaves_material_id()).unwrap();
    assert_eq!(ours.alpha_mode, AlphaMode::Cutout);
    assert_eq!(ours.alpha_mode, theirs.alpha_mode);
    assert_eq!(
        (ours.specular, ours.shininess),
        (theirs.specular, theirs.shininess)
    );
    // Every texel is either fully opaque or a hole.
    let t = e
        .manager
        .get(
            ours.face_textures
                .unwrap()
                .texture_for_face(Face::PositiveX),
        )
        .unwrap();
    assert!(t.pixels().iter().all(|c| c.a < 0.01 || c.a > 0.99));
}

// 6
#[test]
fn the_overworld_leaves_are_unchanged() {
    let e = env();
    let leaves = e.gallery.get(leaves_material_id()).unwrap();
    assert!((leaves.albedo.r - 0.30).abs() < 1e-6 && (leaves.albedo.g - 0.45).abs() < 1e-6);
    assert_ne!(leaves_material_id(), red_black_leaves_material_id());
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "807e490",
            "HEAD",
            "--",
            "assets/textures/overworld",
            "src/scene/material_gallery.rs",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
        }
    }
    // Crimson, not green: the mean opaque texel is red-dominant.
    let mut m = TextureManager::new();
    let id = m
        .load(format!("{RED_BLACK_TIMBER_DIR}/leaves.png"))
        .unwrap();
    let px: Vec<_> = m
        .get(id)
        .unwrap()
        .pixels()
        .iter()
        .filter(|c| c.a > 0.5)
        .copied()
        .collect();
    let (r, g) = px.iter().fold((0.0, 0.0), |(r, g), c| (r + c.r, g + c.g));
    assert!(r > g * 3.0, "r {r} g {g}");
    // Varied, not a flat fill: several distinct colours, a few bright accents.
    let distinct: std::collections::HashSet<(u8, u8, u8)> = px
        .iter()
        .map(|c| {
            (
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
            )
        })
        .collect();
    assert!(distinct.len() >= 20, "{}", distinct.len());
    assert!(px.iter().any(|c| c.r > 0.70), "no brighter red accent");
    assert!(px.iter().all(|c| c.r < 0.90), "neon red");
}

// 7
#[test]
fn the_texture_is_sixteen_by_sixteen() {
    let mut m = TextureManager::new();
    let id = m
        .load(format!("{RED_BLACK_TIMBER_DIR}/leaves.png"))
        .unwrap();
    let t = m.get(id).unwrap();
    assert_eq!((t.width(), t.height()), (16, 16));
}

// 8
#[test]
fn sampling_is_nearest() {
    let mut m = TextureManager::new();
    let id = m
        .load(format!("{RED_BLACK_TIMBER_DIR}/leaves.png"))
        .unwrap();
    let t = m.get(id).unwrap();
    for (x, y) in [(0usize, 0usize), (5, 9), (15, 15), (8, 3)] {
        // The centre of texel (x, y), in either v convention, returns that texel.
        let u = (x as f32 + 0.5) / 16.0;
        let v = (y as f32 + 0.5) / 16.0;
        let a = sample_nearest(t, Vec2::new(u, v));
        let b = sample_nearest(t, Vec2::new(u, 1.0 - v));
        let texel = t.texel(x, y).unwrap();
        assert!(a == texel || b == texel, "({x},{y})");
        // A nudge inside the texel never blends with its neighbours.
        let c = sample_nearest(t, Vec2::new(u + 0.01, v));
        assert!(c == a);
    }
}
