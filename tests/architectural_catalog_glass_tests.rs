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
use core::hit::Face;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, trace_ray};
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_gallery::{glass_material_id, water_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const RED: u32 = 220;

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn env() -> Env {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let mut library = catalog_materials(&textures);
    library.insert(
        MaterialId::new(RED),
        Material::matte(Color::new(1.0, 0.0, 0.0, 1.0)),
    );
    Env { manager, library }
}

fn trace(e: &Env, world: &VoxelWorld, r: &Ray) -> Color {
    let s = VoxelScene {
        world,
        materials: &e.library,
        camera_position: r.origin,
        lights: &[],
        ambient_factor: 1.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: 60.0,
    };
    trace_ray(&s, r, 0)
}

#[test]
fn glass_is_a_full_cube_block_type() {
    assert!(matches!(
        block_geometry(BlockType::Glass, Orientation::Up),
        BlockGeometry::FullCube
    ));
}

#[test]
fn the_material_keeps_the_glass_optical_profile() {
    let e = env();
    let m = e.library.get(glass_material_id()).unwrap();

    assert!((m.transparency - 0.92).abs() < 1e-6);
    assert!((m.refractive_index - 1.5).abs() < 1e-6);
    assert!((m.specular - 0.75).abs() < 1e-6 && (m.shininess - 140.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.10).abs() < 1e-6);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert_eq!(
        m.alpha_mode,
        AlphaMode::Blend,
        "opaque frame texels, clear center"
    );
}

#[test]
fn the_texture_is_the_reference_glass_frame_with_a_clear_center() {
    let e = env();
    let m = e.library.get(glass_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    assert_eq!((t.width(), t.height()), (16, 16));

    // Border ring: opaque, pale cyan/grey. Interior: mostly clear.
    for k in 0..16 {
        for (x, y) in [(k, 0), (k, 15), (0, k), (15, k)] {
            let c = t.texel(x, y).unwrap();
            assert_eq!(c.a, 1.0, "frame texel ({x},{y})");
            assert!(c.b >= c.r && c.r > 0.4, "pale cool tone: {c:?}");
        }
    }
    let clear = (1..15)
        .flat_map(|y| (1..15).map(move |x| (x, y)))
        .filter(|&(x, y)| t.texel(x, y).unwrap().a == 0.0)
        .count();
    assert!(clear > 180, "{clear} clear interior texels");
}

#[test]
fn a_ray_sees_the_object_behind_the_glass() {
    let e = env();
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Glass, glass_material_id(), Orientation::Up),
    );
    w.insert(
        IVec3::new(0, 0, -3),
        BlockInstance::new(BlockType::Stone, MaterialId::new(RED), Orientation::Up),
    );
    // Center of the face: a clear texel.
    let c = trace(
        &e,
        &w,
        &Ray::new(Vec3::new(0.53, 0.53, 5.0), Vec3::new(0.0, 0.0, -1.0)),
    );
    assert!(c.r > 0.5 && c.r > 3.0 * c.b, "red behind glass: {c:?}");
}

#[test]
fn glass_refracts_differently_from_water() {
    let e = env();
    let glass = e.library.get(glass_material_id()).unwrap();
    let water = e.library.get(water_material_id()).unwrap();
    assert_ne!(glass.refractive_index, water.refractive_index);
    assert!(glass.transparency > water.transparency);
    assert_ne!(
        glass
            .face_textures
            .unwrap()
            .texture_for_face(Face::PositiveY),
        water
            .face_textures
            .unwrap()
            .texture_for_face(Face::PositiveY)
    );

    // An oblique ray through each ends up in a different place.
    let mut hits = Vec::new();
    for id in [glass_material_id(), water_material_id()] {
        let mut w = VoxelWorld::new();
        w.insert(
            IVec3::new(0, 0, 0),
            BlockInstance::new(BlockType::Glass, id, Orientation::Up),
        );
        for x in -2..6 {
            let mat = if x % 2 == 0 { RED } else { 2 };
            w.insert(
                IVec3::new(x, -1, 0),
                BlockInstance::new(BlockType::Stone, MaterialId::new(mat), Orientation::Up),
            );
        }
        let colors: Vec<Color> = (0..16)
            .map(|i| {
                let x = 0.1 + i as f32 * 0.05;
                trace(
                    &e,
                    &w,
                    &Ray::new(Vec3::new(x - 3.0, 3.0, 0.5), Vec3::new(3.0, -3.0, 0.0)),
                )
            })
            .collect();
        hits.push(colors);
    }
    assert_ne!(hits[0], hits[1]);
}

#[test]
fn glass_is_one_official_catalog_entry_with_no_diagnostic_duplicate() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::Glass || e.material_id == glass_material_id())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display_name, "Glass");
    assert_eq!(entries[0].kind, SampleKind::Transmissive);

    let scene = CatalogScene::new();
    assert_eq!(
        scene.world().get(entries[0].position),
        Some(&entries[0].block())
    );
    let p = entries[0].focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn the_reused_glass_asset_is_the_single_gate_07_file() {
    let dir = format!("{}/assets/textures/overworld", env!("CARGO_MANIFEST_DIR"));
    let glass: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.contains("glass"))
        .collect();
    assert_eq!(glass, ["glass.png"]);
}
