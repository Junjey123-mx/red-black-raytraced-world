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

use core::hit::Face;
use core::math::{IVec3, Vec2};
use renderer::texture_sampling::sample_nearest;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::cobblestone_material_id;
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

fn faces(e: &Env) -> core::face_textures::FaceTextures {
    e.library
        .get(cobblestone_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

#[test]
fn cobblestone_is_a_full_cube_block_type() {
    let geometry = block_geometry(BlockType::Cobblestone, Orientation::Up);
    assert!(matches!(geometry, BlockGeometry::FullCube));
}

#[test]
fn the_material_and_every_face_texture_resolve() {
    let e = env();
    let f = faces(&e);
    for face in FACES {
        assert!(
            e.manager.get(f.texture_for_face(face)).is_some(),
            "{face:?}"
        );
    }
}

#[test]
fn the_face_texture_policy_matches_the_block() {
    let e = env();
    let f = faces(&e);
    let uniform = true;
    if uniform {
        let first = f.texture_for_face(Face::PositiveY);
        for face in FACES {
            assert_eq!(
                f.texture_for_face(face),
                first,
                "{face:?} shares the one texture"
            );
        }
    } else {
        assert_ne!(
            f.texture_for_face(Face::PositiveY),
            f.texture_for_face(Face::PositiveZ)
        );
    }
    for t in FACES.iter().map(|x| f.texture_for_face(*x)) {
        let tex = e.manager.get(t).unwrap();
        assert_eq!((tex.width(), tex.height()), (16, 16), "pixel-art tile");
    }
}

#[test]
fn the_material_is_opaque_non_emissive_and_matches_its_profile() {
    let e = env();
    let m = e.library.get(cobblestone_material_id()).unwrap();

    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.04).abs() < 1e-6, "specular {}", m.specular);
    assert!((m.shininess - 5.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.01).abs() < 1e-6);
}

#[test]
fn every_texel_is_fully_opaque() {
    let e = env();
    let f = faces(&e);
    for face in FACES {
        let t = e.manager.get(f.texture_for_face(face)).unwrap();
        assert!(t.pixels().iter().all(|c| c.a == 1.0), "{face:?}");
    }
}

#[test]
fn it_is_an_official_catalog_entry_with_a_finite_focus_point() {
    let scene = CatalogScene::new();
    let entry = scene
        .entries()
        .iter()
        .find(|e| e.block_type == BlockType::Cobblestone)
        .expect("in the catalog");

    assert_eq!(entry.display_name, "Cobblestone");
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, cobblestone_material_id());
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn there_is_exactly_one_sample_of_this_block() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::Cobblestone)
        .count();
    assert_eq!(count, 1);

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -10..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.material_id() == cobblestone_material_id() {
                        uses += 1;
                    }
                }
            }
        }
    }
    assert_eq!(uses, 1, "no diagnostic duplicate");
}

#[test]
fn sampling_is_nearest_neighbor() {
    let e = env();
    let f = faces(&e);
    let tex = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();

    for (i, j) in [(0usize, 0usize), (5, 3), (15, 15), (8, 12)] {
        let expected = tex.texel(i, j).unwrap();
        for (du, dv) in [(0.05, 0.05), (0.5, 0.5), (0.95, 0.95)] {
            let uv = Vec2::new((i as f32 + du) / 16.0, (j as f32 + dv) / 16.0);
            assert_eq!(sample_nearest(tex, uv), expected, "texel ({i},{j})");
        }
    }
}

fn luminance(c: core::color::Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

#[test]
fn cobblestone_is_grey_with_dark_joints_and_more_contrast_than_a_smooth_block() {
    let e = env();
    let f = faces(&e);
    let t = e.manager.get(f.texture_for_face(Face::PositiveY)).unwrap();
    let lums: Vec<f32> = t.pixels().iter().map(|c| luminance(*c)).collect();
    let mean = lums.iter().sum::<f32>() / lums.len() as f32;
    let var = lums.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / lums.len() as f32;

    // Neutral grey: channels stay close together.
    for c in t.pixels() {
        assert!((c.r - c.b).abs() < 0.1, "not a neutral grey: {c:?}");
    }
    assert!(
        var.sqrt() > 0.06,
        "rough, fragmented look: std {}",
        var.sqrt()
    );
    let darkest = lums.iter().cloned().fold(1.0, f32::min);
    let brightest = lums.iter().cloned().fold(0.0, f32::max);
    assert!(
        brightest - darkest > 0.3,
        "dark joints against light fragments"
    );
}

#[test]
fn cobblestone_has_no_normal_map_and_is_a_distinct_texture() {
    let e = env();
    let m = e.library.get(cobblestone_material_id()).unwrap();
    assert!(m.normal_texture.is_none(), "albedo-only definitive block");

    let other = e
        .library
        .get(scene::overworld_blocks::dirt_material_id())
        .unwrap()
        .face_textures
        .unwrap();
    assert_ne!(
        m.face_textures.unwrap().texture_for_face(Face::PositiveY),
        other.texture_for_face(Face::PositiveY)
    );
}
