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
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::emission::sample_emissive;
use renderer::raytracer::{VoxelScene, trace_ray};
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::light::Light;
use scene::material_gallery::{LAMP_X, gallery_lights, redstone_lamp_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const N: usize = 16;

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
    let library = catalog_materials(&textures);
    Env { manager, library }
}

fn uv(i: usize, j: usize) -> Vec2 {
    Vec2::new((i as f32 + 0.5) / 16.0, (j as f32 + 0.5) / 16.0)
}

fn black(c: Color) -> bool {
    c.r.abs() < 1e-5 && c.g.abs() < 1e-5 && c.b.abs() < 1e-5
}

fn lum(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn lamp_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::RedstoneLampLit,
            redstone_lamp_material_id(),
            Orientation::Up,
        ),
    );
    w
}

#[test]
fn the_lamp_is_a_full_cube_block_type() {
    assert!(matches!(
        block_geometry(BlockType::RedstoneLampLit, Orientation::Up),
        BlockGeometry::FullCube
    ));
}

#[test]
fn albedo_emissive_mask_and_material_resolve() {
    let e = env();
    let m = e.library.get(redstone_lamp_material_id()).unwrap();
    let a = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    let em = e
        .manager
        .get(m.emissive_texture.expect("emissive mask"))
        .unwrap();

    assert_eq!((a.width(), a.height()), (16, 16));
    assert_eq!((em.width(), em.height()), (16, 16));
    assert_ne!(
        m.face_textures.unwrap().texture_for_face(Face::PositiveZ),
        m.emissive_texture.unwrap(),
        "the mask is a separate layer"
    );
    assert!(m.normal_texture.is_none());
}

#[test]
fn the_emission_is_positive_within_the_documented_range() {
    let e = env();
    let m = e.library.get(redstone_lamp_material_id()).unwrap();
    assert!(
        (1.8..=3.0).contains(&m.emission_strength),
        "{}",
        m.emission_strength
    );
    assert_eq!(m.transparency, 0.0);
    assert!((m.specular - 0.10).abs() < 1e-6 && (m.shininess - 20.0).abs() < 1e-6);
}

#[test]
fn only_the_lit_panels_emit_and_the_dark_frame_does_not() {
    let e = env();
    let m = e.library.get(redstone_lamp_material_id()).unwrap();
    let a = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();

    let (mut lit, mut dark) = (0, 0);
    for j in 0..N {
        for i in 0..N {
            let emitted = sample_emissive(m, uv(i, j), &e.manager);
            let base = a.texel(i, j).unwrap();
            if black(emitted) {
                dark += 1;
                assert!(
                    lum(base) < 0.3,
                    "non-emitting texel is dark brown: {base:?}"
                );
            } else {
                lit += 1;
                assert!(
                    lum(base) > 0.3,
                    "emitting texel is a bright amber panel: {base:?}"
                );
            }
        }
    }
    assert!(
        lit > 80 && dark > 80,
        "structure + panels: lit {lit}, dark {dark}"
    );
}

#[test]
fn the_lamp_glows_in_the_dark_but_only_on_its_panels() {
    let e = env();
    let world = lamp_world();
    let m = e.library.get(redstone_lamp_material_id()).unwrap();
    let scene = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 0.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: 60.0,
    };
    let ray_at = |i: usize, j: usize| {
        Ray::new(
            Vec3::new((i as f32 + 0.5) / 16.0, 1.0 - (j as f32 + 0.5) / 16.0, 5.0),
            Vec3::new(0.0, 0.0, -1.0),
        )
    };

    let mut glowing = 0;
    let mut unlit = 0;
    for j in 0..N {
        for i in 0..N {
            let c = trace_ray(&scene, &ray_at(i, j), 0);
            let emitted = sample_emissive(m, uv(i, j), &e.manager);
            if black(emitted) {
                assert!(black(c), "frame stays black with no light ({i},{j}): {c:?}");
                unlit += 1;
            } else {
                assert!(lum(c) > 0.05, "panel glows in darkness ({i},{j}): {c:?}");
                glowing += 1;
            }
        }
    }
    assert!(glowing > 0 && unlit > 0, "not a flat yellow cube");
}

#[test]
fn the_gallery_lights_include_a_warm_point_light_by_the_lamp() {
    let lights = gallery_lights();
    let warm = lights.iter().any(|l| match l {
        Light::Point(p) => {
            p.color.r > p.color.g
                && p.color.g > p.color.b
                && (p.position.x - (LAMP_X as f32 + 0.5)).abs() < 1.0
        }
        _ => false,
    });
    assert!(warm, "the lamp keeps its warm PointLight");
}

#[test]
fn the_lamp_is_one_official_catalog_entry() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::RedstoneLampLit)
        .collect();
    assert_eq!(entries.len(), 1, "no duplicate lamp representation");
    assert_eq!(entries[0].kind, SampleKind::Emissive);
    assert_eq!(entries[0].material_id, redstone_lamp_material_id());

    let scene = CatalogScene::new();
    assert_eq!(
        scene.world().get(entries[0].position),
        Some(&entries[0].block())
    );
    let p = entries[0].focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}
