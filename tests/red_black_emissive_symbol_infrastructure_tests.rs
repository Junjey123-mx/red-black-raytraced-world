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

use core::color::Color;
use core::face_textures::FaceTextures;
use core::material::{AlphaMode, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use core::texture::TextureId;
use renderer::emission::sample_emissive;
use renderer::raytracer::cast_ray_voxel_lit;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::catalog::{CatalogTextures, catalog_materials};
use scene::light::{DirectionalLight, Light, PointLight};
use scene::material_gallery::{
    glass_material_id, leaves_material_id, portal_core_material_id, redstone_lamp_material_id,
    water_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{SymbolGlow, crimson_heart_material_id, symbol_block_material};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const ID: u32 = 900;
const GLOW: [u8; 3] = [200, 30, 40];

/// A 16x16 emissive colour mask written to a temp file: the top-left and
/// bottom-right 8x8 quadrants carry `GLOW`, the other two are black. Any two
/// texels in horizontally adjacent quadrants therefore differ in emission,
/// whatever the face's UV orientation.
fn quadrant_mask(manager: &mut TextureManager) -> TextureId {
    use raylib::prelude::{Color as RlColor, Image};
    use std::sync::atomic::{AtomicUsize, Ordering};
    // One file per call: the tests run in parallel and must never read a
    // mask another test is still writing.
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "rb_symbol_glow_fixture_{}_{}.png",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut image = Image::gen_image_color(16, 16, RlColor::BLACK);
    let glow = RlColor::new(GLOW[0], GLOW[1], GLOW[2], 255);
    image.draw_rectangle(0, 0, 8, 8, glow);
    image.draw_rectangle(8, 8, 8, 8, glow);
    image.export_image(path.to_str().unwrap());
    let id = manager.load(&path).expect("fixture loads");
    let _ = std::fs::remove_file(&path);
    id
}

struct Fixture {
    manager: TextureManager,
    albedo: FaceTextures,
    mask: TextureId,
}

fn fixture() -> Fixture {
    let mut manager = TextureManager::new();
    let mask = quadrant_mask(&mut manager);
    let albedo = manager
        .load("assets/textures/red_black_maze/crimson/heart_albedo.png")
        .unwrap();
    Fixture {
        manager,
        albedo: FaceTextures::uniform(albedo),
        mask,
    }
}

fn material(f: &Fixture, glow: Option<SymbolGlow>) -> core::material::Material {
    symbol_block_material(
        Color::new(0.82, 0.12, 0.17, 1.0),
        (0.06, 8.0, 0.01),
        f.albedo,
        glow,
    )
}

/// Traces the +Z face of a lone cube at `(x, y)` on that face.
fn trace(
    f: &Fixture,
    glow: Option<SymbolGlow>,
    lights: &[Light],
    ambient: f32,
    x: f32,
    y: f32,
) -> Color {
    let mut world = VoxelWorld::new();
    world.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::CrimsonHeart,
            MaterialId::new(ID),
            Orientation::Up,
        ),
    );
    let mut library = MaterialLibrary::new();
    library.insert(MaterialId::new(ID), material(f, glow));
    let origin = Vec3::new(x, y, 4.0);
    let ray = Ray::new(origin, Vec3::new(0.0, 0.0, -1.0));
    cast_ray_voxel_lit(
        &world,
        &library,
        &ray,
        origin,
        lights,
        ambient,
        Color::black(),
        &f.manager,
        20.0,
    )
}

/// Soft, off-axis lighting that never saturates the face, so any added
/// emission stays measurable.
fn lights() -> Vec<Light> {
    vec![
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.3, 1.0, 0.8),
            Color::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(4.0, 1.0, 2.0),
            Color::white(),
            0.3,
        )),
    ]
}

fn luma(c: Color) -> f32 {
    0.299 * c.r + 0.587 * c.g + 0.114 * c.b
}

fn glow(f: &Fixture, strength: f32) -> Option<SymbolGlow> {
    Some(SymbolGlow {
        mask: f.mask,
        strength,
    })
}

#[test]
fn the_symbol_material_accepts_an_albedo_plus_an_emissive_mask() {
    let f = fixture();
    let m = material(&f, glow(&f, 1.8));
    assert_eq!(m.face_textures, Some(f.albedo));
    assert_eq!(m.emissive_texture, Some(f.mask));
    assert!((m.emission_strength - 1.8).abs() < 1e-6);
    // Emission never changes the rest of the block.
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.refractive_index, 1.0);
    assert_eq!(m.alpha_mode, AlphaMode::Ignore);
    assert!(m.normal_texture.is_none());
}

#[test]
fn a_material_with_an_emissive_mask_resolves_through_the_library() {
    let f = fixture();
    let mut library = MaterialLibrary::new();
    library.insert(MaterialId::new(ID), material(&f, glow(&f, 1.8)));
    let m = library.get(MaterialId::new(ID)).expect("resolves");
    let mask = f.manager.get(m.emissive_texture.unwrap()).expect("loaded");
    assert_eq!((mask.width(), mask.height()), (16, 16));
}

#[test]
fn only_marked_texels_emit_light_of_their_own() {
    let f = fixture();
    // No lights, no ambient: whatever is visible is pure self-emission.
    let a = trace(&f, glow(&f, 1.8), &[], 0.0, 0.25, 0.25);
    let b = trace(&f, glow(&f, 1.8), &[], 0.0, 0.75, 0.25);
    let (lit, dark) = if luma(a) > luma(b) { (a, b) } else { (b, a) };

    let expected = |channel: u8| (channel as f32 / 255.0 * 1.8).min(1.0);
    assert!((lit.r - expected(GLOW[0])).abs() < 0.01, "{lit:?}");
    assert!((lit.g - expected(GLOW[1])).abs() < 0.01, "{lit:?}");
    assert!((lit.b - expected(GLOW[2])).abs() < 0.01, "{lit:?}");
    assert_eq!(
        dark.to_rgba8()[..3],
        [0, 0, 0],
        "a black mask texel emits nothing"
    );
}

#[test]
fn the_dark_background_adds_nothing_to_the_lit_block() {
    let f = fixture();
    let lights = lights();
    for (x, y) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
        let plain = trace(&f, None, &lights, 0.2, x, y);
        let glowing = trace(&f, glow(&f, 1.8), &lights, 0.2, x, y);
        let emitted = trace(&f, glow(&f, 1.8), &[], 0.0, x, y);
        if emitted.to_rgba8()[..3] == [0, 0, 0] {
            assert_eq!(plain, glowing, "background texel at ({x},{y})");
        } else {
            assert!(
                glowing.r >= plain.r && luma(glowing) > luma(plain),
                "{plain:?} {glowing:?} {emitted:?}"
            );
        }
    }
}

#[test]
fn zero_strength_is_exactly_the_gate_10_non_emissive_block() {
    let f = fixture();
    let lights = lights();
    let zero = material(&f, glow(&f, 0.0));
    assert_eq!(
        sample_emissive(&zero, core::math::Vec2::new(0.1, 0.1), &f.manager),
        Color::black()
    );
    for (x, y) in [(0.1, 0.1), (0.4, 0.6), (0.9, 0.2), (0.6, 0.9)] {
        assert_eq!(
            trace(&f, glow(&f, 0.0), &lights, 0.2, x, y),
            trace(&f, None, &lights, 0.2, x, y),
            "({x},{y})"
        );
    }
}

fn catalog_library() -> (TextureManager, MaterialLibrary) {
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
    (manager, library)
}

#[test]
fn glass_water_and_leaves_are_untouched() {
    let (_, library) = catalog_library();
    let glass = library.get(glass_material_id()).unwrap();
    assert!((glass.transparency - 0.92).abs() < 1e-6);
    assert!((glass.refractive_index - 1.5).abs() < 1e-6);
    assert_eq!(glass.emission_strength, 0.0);
    let water = library.get(water_material_id()).unwrap();
    assert!(water.transparency > 0.5 && (water.refractive_index - 1.33).abs() < 1e-6);
    assert_eq!(water.emission_strength, 0.0);
    let leaves = library.get(leaves_material_id()).unwrap();
    assert_eq!(leaves.alpha_mode, AlphaMode::Cutout);
    assert_eq!(leaves.emission_strength, 0.0);
}

#[test]
fn the_redstone_lamp_keeps_its_own_emission() {
    let (manager, library) = catalog_library();
    let lamp = library.get(redstone_lamp_material_id()).unwrap();
    assert!((lamp.emission_strength - 1.8).abs() < 1e-6);
    let mask = lamp.emissive_texture.expect("lamp mask");
    assert!(manager.get(mask).is_some());
}

#[test]
fn the_portal_core_keeps_its_own_emission() {
    let (manager, library) = catalog_library();
    let core = library.get(portal_core_material_id()).unwrap();
    assert!((core.emission_strength - 2.5).abs() < 1e-6);
    assert!((core.transparency - 0.25).abs() < 1e-6);
    assert!(
        manager
            .get(core.emissive_texture.expect("core mask"))
            .is_some()
    );
}

#[test]
fn the_card_suit_blocks_are_built_by_the_shared_helper() {
    let (_, library) = catalog_library();
    let heart = library.get(crimson_heart_material_id()).unwrap();
    let rebuilt = symbol_block_material(
        heart.albedo,
        (heart.specular, heart.shininess, heart.reflectivity),
        heart.face_textures.unwrap(),
        heart.emissive_texture.map(|mask| SymbolGlow {
            mask,
            strength: heart.emission_strength,
        }),
    );
    assert_eq!(&rebuilt, heart);
}

#[test]
fn no_raylib_3d_is_used() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new("src"), &mut files);
    assert!(!files.is_empty());
    for file in files {
        let code = std::fs::read_to_string(&file).unwrap();
        for banned in [
            "Camera3D",
            "BeginMode3D",
            "begin_mode3D",
            "DrawCube",
            "draw_cube",
            "DrawModel",
            "draw_model",
            "Mesh",
            "Model",
            "Shader",
        ] {
            assert!(!code.contains(banned), "{banned} in {}", file.display());
        }
    }
}
