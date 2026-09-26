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
use core::material::MaterialId;
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::emission::sample_emissive;
use renderer::raytracer::{VoxelScene, nearest_voxel_hit, trace_ray};
use scene::block::BlockInstance;
use scene::block_geometry::{BlockGeometry, GeometryKind};
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_gallery::portal_core_material_id;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

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

fn core_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::PortalCoreDarkCrimson,
            portal_core_material_id(),
            Orientation::South,
        ),
    );
    w
}

fn uv(i: usize, j: usize) -> Vec2 {
    Vec2::new((i as f32 + 0.5) / 16.0, (j as f32 + 0.5) / 16.0)
}

#[test]
fn the_core_is_thin_partial_geometry_not_a_full_cube() {
    let g = block_geometry(BlockType::PortalCoreDarkCrimson, Orientation::South);
    assert!(!matches!(g, BlockGeometry::FullCube));
    assert_eq!(g.kind(), GeometryKind::Prism);
    let p = g.parts()[0];
    assert!(
        p.size().z <= 0.15 && p.size().x > 0.99 && p.size().y > 0.99,
        "{:?}",
        p.size()
    );
}

#[test]
fn material_and_textures_resolve_and_are_the_project_core_layers() {
    let e = env();
    let m = e.library.get(portal_core_material_id()).unwrap();
    let f = m.face_textures.unwrap();
    for face in [
        Face::PositiveX,
        Face::PositiveY,
        Face::PositiveZ,
        Face::NegativeZ,
    ] {
        assert!(e.manager.get(f.texture_for_face(face)).is_some());
    }
    let mask = e
        .manager
        .get(m.emissive_texture.expect("emissive layer"))
        .unwrap();
    assert_eq!((mask.width(), mask.height()), (16, 16));
    assert_ne!(
        Some(f.texture_for_face(Face::PositiveZ)),
        m.emissive_texture
    );
    assert!(m.normal_texture.is_none());
}

#[test]
fn it_emits_light_and_is_partly_transparent_without_refracting() {
    let e = env();
    let m = e.library.get(portal_core_material_id()).unwrap();

    assert!(
        (2.5..=4.0).contains(&m.emission_strength),
        "{}",
        m.emission_strength
    );
    assert!(
        (0.20..=0.35).contains(&m.transparency),
        "{}",
        m.transparency
    );
    assert_eq!(m.refractive_index, 1.0, "no glass-like refraction");
    assert!(m.reflectivity <= 0.05);

    let lit = (0..256)
        .map(|n| (n % 16, n / 16))
        .filter(|&(i, j)| {
            let c = sample_emissive(m, uv(i, j), &e.manager);
            c.r + c.g + c.b > 0.0
        })
        .count();
    assert!(
        lit > 20 && lit < 200,
        "only the brighter swirls emit: {lit}"
    );
}

#[test]
fn the_color_is_dark_crimson_wine_and_never_vanilla_purple() {
    let e = env();
    let m = e.library.get(portal_core_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();

    for c in t.pixels() {
        assert!(c.r >= c.g && c.r >= c.b * 1.5, "red-dominant wine: {c:?}");
    }
    let n = t.pixels().len() as f32;
    let r: f32 = t.pixels().iter().map(|c| c.r).sum::<f32>() / n;
    assert!(r < 0.5, "dark overall: {r}");

    // What actually lights the scene is the emitted color: also crimson.
    let (mut er, mut eb) = (0.0, 0.0);
    for j in 0..16 {
        for i in 0..16 {
            let c: Color = sample_emissive(m, uv(i, j), &e.manager);
            er += c.r;
            eb += c.b;
        }
    }
    assert!(
        er > eb * 1.5,
        "emission is crimson/magenta, not violet: {er} vs {eb}"
    );
}

#[test]
fn a_ray_hits_the_thin_membrane_and_the_rest_of_the_cell_is_air() {
    let world = core_world();
    let h = nearest_voxel_hit(
        &world,
        &Ray::new(Vec3::new(0.5, 0.5, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        50.0,
    )
    .unwrap();
    assert!((h.hit.point.z - 0.5625).abs() < 1e-4);
    // Sideways through the empty part of the cell, in front of the slab.
    assert!(
        nearest_voxel_hit(
            &world,
            &Ray::new(Vec3::new(-3.0, 0.5, 0.2), Vec3::new(1.0, 0.0, 0.0)),
            50.0
        )
        .is_none()
    );
}

#[test]
fn the_emission_persists_in_the_dark_through_the_transparency() {
    let e = env();
    let world = core_world();
    let m = e.library.get(portal_core_material_id()).unwrap();
    let s = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 0.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: 50.0,
    };
    let (i, j) = (0..256)
        .map(|n| (n % 16, n / 16))
        .find(|&(i, j)| {
            let c = sample_emissive(m, uv(i, j), &e.manager);
            c.r + c.g + c.b > 0.0
        })
        .unwrap();
    let r = Ray::new(
        Vec3::new((i as f32 + 0.5) / 16.0, 1.0 - (j as f32 + 0.5) / 16.0, 5.0),
        Vec3::new(0.0, 0.0, -1.0),
    );
    let c = trace_ray(&s, &r, 0);
    assert!(c.r > 0.1, "glows with no lights at all: {c:?}");
}

#[test]
fn there_is_one_official_core_and_no_second_portal_core_asset() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::PortalCoreDarkCrimson)
        .collect();
    assert_eq!(entries.len(), 1, "no duplicate core entry");
    assert_eq!(entries[0].kind, SampleKind::PartialGeometry);
    assert_eq!(entries[0].material_id, portal_core_material_id());
    assert_ne!(
        entries[0].material_id,
        MaterialId::new(4),
        "not the Gate 06 placeholder"
    );

    let scene = CatalogScene::new();
    assert_eq!(
        scene.world().get(entries[0].position),
        Some(&entries[0].block())
    );
    let p = entries[0].focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());

    // Reused, not regenerated: the portal folder holds the two Gate 07 core
    // layers plus the frame, and nothing else.
    let dir = format!("{}/assets/textures/portal", env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        ["core_albedo.png", "core_emissive.png", "frame_albedo.png"]
    );
}
