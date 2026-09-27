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
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
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
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::diagnostic::DiagnosticCameraState;
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{
    CatalogScene, CatalogTextures, FOCUS_DISTANCE, SampleKind, catalog_entries, catalog_materials,
    official_entries,
};
use scene::material_gallery::gallery_camera;
use scene::texture_manager::TextureManager;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Number of times `needle` appears across every source file under `src/`.
fn occurrences_in_src(needle: &str) -> usize {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    files
        .iter()
        .map(|f| std::fs::read_to_string(f).unwrap().matches(needle).count())
        .sum()
}

#[test]
fn the_shared_catalog_bootstrap_is_exposed_by_the_library() {
    // The real library crate exports the entrypoint every binary calls.
    let entry: fn() = red_black_raytraced_world::run_catalog_app;
    let _ = entry;
    let lib = std::fs::read_to_string("src/lib.rs").unwrap();
    assert!(lib.contains("pub use app::run_catalog_app;"));
    // Modules stay private to the library: only entrypoints are public.
    for module in ["app", "camera", "config", "core", "renderer", "scene"] {
        assert!(lib.contains(&format!("\nmod {module};")), "{module}");
        assert!(!lib.contains(&format!("pub mod {module};")), "{module}");
    }
}

#[test]
fn main_is_a_thin_entrypoint_without_its_own_module_tree() {
    let main = std::fs::read_to_string("src/main.rs").unwrap();
    assert!(main.contains("red_black_raytraced_world::"));
    assert!(
        !main.contains("mod "),
        "the module tree lives only in lib.rs"
    );
    assert!(main.lines().count() <= 10);
}

#[test]
fn the_catalog_scene_still_builds_with_every_official_block() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    let scene = CatalogScene::new();
    assert_eq!(scene.len(), OFFICIAL_BLOCK_COUNT + 2);
    let official = official_entries();
    assert_eq!(official.len(), OFFICIAL_BLOCK_COUNT);
    let types: Vec<BlockType> = official.iter().map(|e| e.block_type).collect();
    assert_eq!(types, BlockType::ALL.to_vec());

    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let library = catalog_materials(&textures);
    for entry in &official {
        assert!(
            library.contains(entry.material_id),
            "{}",
            entry.display_name
        );
        assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    }
}

#[test]
fn the_two_diagnostic_samples_are_preserved() {
    let diagnostics: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| !e.is_official())
        .map(|e| (e.display_name, e.kind))
        .collect();
    assert_eq!(
        diagnostics,
        vec![
            ("Control cube", SampleKind::Control),
            ("Reflective cube", SampleKind::Reflective)
        ]
    );
}

#[test]
fn selection_and_focus_behave_as_before() {
    let home = gallery_camera(4.0 / 3.0);
    let initial = DiagnosticCameraState::from_pose(home.position, home.target);
    let mut s = CatalogScene::new();
    assert_eq!(s.selected_index(), 0);
    for i in 0..s.len() {
        assert_eq!(s.selected_index(), i);
        let mut view = initial;
        s.focus_camera(&mut view);
        assert_eq!(view.target, s.selected().focus_point);
        assert!((view.distance - FOCUS_DISTANCE).abs() < 1e-5);
        assert_eq!((view.yaw, view.pitch), (initial.yaw, initial.pitch));
        s.select_next();
    }
    assert_eq!(s.selected_index(), 0, "wraps");
    s.select_previous();
    assert_eq!(s.selected_index(), s.len() - 1);
}

#[test]
fn the_registry_and_material_library_exist_exactly_once() {
    assert_eq!(occurrences_in_src("pub const OFFICIAL_BLOCK_COUNT"), 1);
    assert_eq!(occurrences_in_src("pub enum BlockType"), 1);
    assert_eq!(occurrences_in_src("pub const ALL: [BlockType"), 1);
    assert_eq!(occurrences_in_src("pub fn catalog_entries()"), 1);
    assert_eq!(occurrences_in_src("pub struct CatalogScene"), 1);
    assert_eq!(occurrences_in_src("pub struct MaterialLibrary"), 1);
    assert_eq!(occurrences_in_src("pub struct TextureManager"), 1);
    assert_eq!(occurrences_in_src("pub fn cast_ray_voxel_lit("), 1);
}

#[test]
fn the_catalog_binary_is_a_thin_launcher_of_the_shared_bootstrap() {
    let bin = std::fs::read_to_string("src/bin/catalog.rs").unwrap();
    assert!(bin.contains("red_black_raytraced_world::run_catalog_app()"));
    // No engine, scene, registry, material or texture code of its own.
    for forbidden in [
        "mod ",
        "BlockType",
        "MaterialLibrary",
        "TextureManager",
        "VoxelWorld",
        "raylib",
        "CatalogEntry",
    ] {
        assert!(!bin.contains(forbidden), "{forbidden}");
    }
    assert!(bin.lines().count() <= 10);
}
