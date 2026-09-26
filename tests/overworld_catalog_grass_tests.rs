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

use core::hit::Face;
use core::math::{Vec2, Vec3};
use renderer::texture_sampling::sample_nearest;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::orientation::Orientation;
use scene::scene::grass_material_id;
use scene::texture_manager::TextureManager;

const FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];

struct Env {
    manager: TextureManager,
    library: scene::material_library::MaterialLibrary,
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

fn grass_faces(e: &Env) -> core::face_textures::FaceTextures {
    e.library
        .get(grass_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

#[test]
fn grass_is_a_definitive_full_cube_block_type() {
    let geometry = block_geometry(BlockType::Grass, Orientation::Up);
    assert!(matches!(geometry, BlockGeometry::FullCube));
}

#[test]
fn the_grass_material_and_all_face_textures_resolve() {
    let e = env();
    let faces = grass_faces(&e);
    for f in FACES {
        assert!(e.manager.get(faces.texture_for_face(f)).is_some(), "{f:?}");
    }
}

#[test]
fn top_side_and_bottom_are_distinct_textures() {
    let e = env();
    let f = grass_faces(&e);
    let top = f.texture_for_face(Face::PositiveY);
    let side = f.texture_for_face(Face::PositiveZ);
    let bottom = f.texture_for_face(Face::NegativeY);

    assert_ne!(top, side);
    assert_ne!(side, bottom);
    assert_ne!(top, bottom);
    for face in [Face::PositiveX, Face::NegativeX, Face::NegativeZ] {
        assert_eq!(
            f.texture_for_face(face),
            side,
            "all four laterals share the side"
        );
    }
}

#[test]
fn the_side_keeps_its_green_fringe_on_top_and_dirt_below() {
    let e = env();
    let f = grass_faces(&e);
    let side = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();
    let greenish = |x: usize, y: usize| {
        let c = side.texel(x, y).unwrap();
        c.g > c.r
    };
    let row_green = |y: usize| (0..16).filter(|&x| greenish(x, y)).count();

    // Top rows are mostly green, the lower half is dirt (no vertical flip).
    assert!(
        row_green(0) >= 10,
        "row 0 has {} green texels",
        row_green(0)
    );
    let lower: usize = (8..16).map(row_green).sum();
    assert!(
        lower < 8,
        "the lower half is dirt, got {lower} green texels"
    );
    let upper: usize = (0..3).map(row_green).sum();
    assert!(upper > lower * 3);
}

#[test]
fn the_top_is_green_and_the_bottom_is_dirt() {
    let e = env();
    let f = grass_faces(&e);
    let green = |id| {
        let t = e.manager.get(id).unwrap();
        (0..256)
            .filter(|n| t.texel(n % 16, n / 16).unwrap().g > t.texel(n % 16, n / 16).unwrap().r)
            .count()
    };
    assert!(
        green(f.texture_for_face(Face::PositiveY)) > 230,
        "top is green"
    );
    assert!(
        green(f.texture_for_face(Face::NegativeY)) < 8,
        "bottom is dirt"
    );
}

#[test]
fn grass_is_opaque_non_emissive_and_matte() {
    let e = env();
    let m = e.library.get(grass_material_id()).unwrap();

    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none() && m.normal_texture.is_none());
    assert!((m.specular - 0.04).abs() < 1e-6);
    assert!((m.shininess - 8.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.01).abs() < 1e-6);
    assert_eq!(m.refractive_index, 1.0);
}

#[test]
fn grass_is_an_official_catalog_entry_with_a_finite_focus_point() {
    let scene = CatalogScene::new();
    let entry = scene
        .entries()
        .iter()
        .find(|e| e.block_type == BlockType::Grass)
        .expect("Grass is in the catalog");

    assert_eq!(entry.display_name, "Grass");
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, grass_material_id());
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p: Vec3 = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn there_is_exactly_one_grass_sample_and_no_diagnostic_duplicate() {
    let grass_entries = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::Grass || e.display_name.contains("Grass"))
        .count();
    assert_eq!(grass_entries, 1);

    // No other block of the catalog world uses the grass material.
    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -10..12 {
                if let Some(b) = scene.world().get(core::math::IVec3::new(x, y, z)) {
                    if b.material_id() == grass_material_id() {
                        uses += 1;
                    }
                }
            }
        }
    }
    assert_eq!(uses, 1);
}

#[test]
fn face_textures_are_sampled_nearest_neighbor() {
    let e = env();
    let f = grass_faces(&e);
    let side = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();

    // Every point inside one texel returns exactly that texel's color.
    for (i, j) in [(0usize, 0usize), (5, 3), (15, 15), (8, 12)] {
        let expected = side.texel(i, j).unwrap();
        for (du, dv) in [(0.05, 0.05), (0.5, 0.5), (0.95, 0.95)] {
            let uv = Vec2::new((i as f32 + du) / 16.0, (j as f32 + dv) / 16.0);
            assert_eq!(sample_nearest(side, uv), expected, "texel ({i},{j})");
        }
    }
}
