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

use core::color::Color;
use core::hit::Face;
use scene::block_shape_factory::block_geometry;
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    OverworldBlockTextures, RED_BLACK_TEXTURES_DIR, insert_overworld_materials,
    wood_planks_material_id,
};
use scene::red_black_timber::{
    RED_BLACK_TIMBER_DIR, red_black_wood_planks_material_id, timber_contract,
};
use scene::texture_manager::TextureManager;

fn env() -> (TextureManager, MaterialLibrary) {
    let mut manager = TextureManager::new();
    let blocks = OverworldBlockTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &blocks);
    (manager, library)
}

fn mean(path: &str) -> Color {
    let mut m = TextureManager::new();
    let id = m.load(path).unwrap();
    let px = m.get(id).unwrap().pixels().to_vec();
    let n = px.len() as f32;
    let (r, g, b) = px
        .iter()
        .fold((0.0, 0.0, 0.0), |a, c| (a.0 + c.r, a.1 + c.g, a.2 + c.b));
    Color::new(r / n, g / n, b / n, 1.0)
}

fn saturation(c: Color) -> f32 {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    if max <= 0.0 { 0.0 } else { (max - min) / max }
}

fn distance(a: Color, b: Color) -> f32 {
    ((a.r - b.r).powi(2) + (a.g - b.g).powi(2) + (a.b - b.b).powi(2)).sqrt()
}

// 1
#[test]
fn the_planks_geometry_is_reused() {
    for o in [
        Orientation::Up,
        Orientation::Down,
        Orientation::South,
        Orientation::West,
    ] {
        assert_eq!(
            block_geometry(BlockType::RedBlackWoodPlanks, o),
            block_geometry(BlockType::WoodPlanks, o)
        );
        assert!(block_geometry(BlockType::RedBlackWoodPlanks, o).is_full_cube());
    }
    let (_, library) = env();
    let ours = library.get(red_black_wood_planks_material_id()).unwrap();
    let theirs = library.get(wood_planks_material_id()).unwrap();
    assert_eq!(ours.alpha_mode, theirs.alpha_mode);
    assert_eq!(
        (ours.specular, ours.shininess, ours.reflectivity),
        (theirs.specular, theirs.shininess, theirs.reflectivity)
    );
}

// 2
#[test]
fn the_texture_is_its_own_corinto_identity() {
    let (mut manager, library) = env();
    let faces = library
        .get(red_black_wood_planks_material_id())
        .unwrap()
        .face_textures
        .unwrap();
    let id = manager
        .load(format!("{RED_BLACK_TIMBER_DIR}/wood_planks.png"))
        .unwrap();
    for f in [Face::PositiveX, Face::NegativeY, Face::PositiveZ] {
        assert_eq!(
            faces.texture_for_face(f),
            id,
            "one texture on every face, like WoodPlanks"
        );
    }
    let t = manager.get(id).unwrap();
    assert_eq!((t.width(), t.height()), (16, 16));
    assert!(t.pixels().iter().all(|c| c.a > 0.99));
    let c = mean(&format!("{RED_BLACK_TIMBER_DIR}/wood_planks.png"));
    assert!(c.r > c.g * 2.5 && c.r > c.b * 2.0, "corinto: {c:?}");
    // Black seams between boards: some texels are near black.
    assert!(t.pixels().iter().any(|p| p.r + p.g + p.b < 0.20));
}

// 3
#[test]
fn the_planks_are_not_emissive() {
    let (_, library) = env();
    let m = library.get(red_black_wood_planks_material_id()).unwrap();
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.emission_strength, 0.0);
    assert_eq!(m.transparency, 0.0);
}

// 4
#[test]
fn there_is_no_conflict_with_the_overworld_planks() {
    let (_, library) = env();
    assert_ne!(
        red_black_wood_planks_material_id(),
        wood_planks_material_id()
    );
    let theirs = library.get(wood_planks_material_id()).unwrap();
    assert!((theirs.albedo.r - 0.63).abs() < 1e-6 && (theirs.albedo.g - 0.50).abs() < 1e-6);
    let a = mean("assets/textures/overworld/wood_planks.png");
    let b = mean(&format!("{RED_BLACK_TIMBER_DIR}/wood_planks.png"));
    assert!(distance(a, b) > 0.25, "{a:?} {b:?}");
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "807e490",
            "HEAD",
            "--",
            "assets/textures/overworld/wood_planks.png",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
        }
    }
}

// 5
#[test]
fn the_planks_read_against_the_dark_stone() {
    let planks = mean(&format!("{RED_BLACK_TIMBER_DIR}/wood_planks.png"));
    for stone in [
        format!("{RED_BLACK_TEXTURES_DIR}/polished_blackstone_bricks.png"),
        format!("{RED_BLACK_TEXTURES_DIR}/smooth_basalt.png"),
        "assets/textures/overworld/deepslate/side.png".to_string(),
        "assets/textures/overworld/deepslate/top.png".to_string(),
    ] {
        let s = mean(&stone);
        assert!(distance(planks, s) > 0.25, "{stone}: {planks:?} vs {s:?}");
        assert!(saturation(planks) > saturation(s) + 0.4, "{stone}");
    }
    // Brighter than blackstone, the fortress body.
    let blackstone = mean(&format!(
        "{RED_BLACK_TEXTURES_DIR}/polished_blackstone_bricks.png"
    ));
    assert!(planks.r + planks.g + planks.b > (blackstone.r + blackstone.g + blackstone.b) * 1.2);
}

// 6
#[test]
fn the_planks_are_ready_for_catalog_registration() {
    let c = timber_contract(BlockType::RedBlackWoodPlanks).unwrap();
    assert_eq!(c.material_ids, vec![red_black_wood_planks_material_id()]);
    for p in &c.texture_paths {
        assert!(std::path::Path::new(p).exists(), "{p}");
    }
    let (_, library) = env();
    assert!(library.contains(red_black_wood_planks_material_id()));
    assert!(OFFICIAL_BLOCK_COUNT == 39 || BlockType::ALL.contains(&BlockType::RedBlackWoodPlanks));
}
