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
use scene::overworld_blocks::log_material_id;
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
        .get(log_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

#[test]
fn log_is_a_full_cube_block_type() {
    let geometry = block_geometry(BlockType::Log, Orientation::Up);
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
    let uniform = false;
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
    let m = e.library.get(log_material_id()).unwrap();

    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.07).abs() < 1e-6, "specular {}", m.specular);
    assert!((m.shininess - 10.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.015).abs() < 1e-6);
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
        .find(|e| e.material_id == log_material_id())
        .expect("in the catalog");

    assert_eq!(entry.display_name, "Log");
    assert_eq!(entry.block_type, BlockType::Log);
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, log_material_id());
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn there_is_exactly_one_sample_of_this_block() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.material_id == log_material_id())
        .count();
    assert_eq!(count, 1);

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -10..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.material_id() == log_material_id() {
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

#[test]
fn top_and_bottom_use_the_end_grain_texture() {
    let e = env();
    let f = faces(&e);
    let end = f.texture_for_face(Face::PositiveY);
    assert_eq!(f.texture_for_face(Face::NegativeY), end);
    let t = e.manager.get(end).unwrap();
    // The end grain is light wood at the center with darker bark on the rim.
    let lum = |x: usize, y: usize| {
        let c = t.texel(x, y).unwrap();
        0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
    };
    assert!(
        lum(7, 7) > lum(0, 8) && lum(8, 8) > lum(8, 0),
        "light center vs bark rim"
    );
}

#[test]
fn the_four_side_faces_use_the_bark_texture() {
    let e = env();
    let f = faces(&e);
    let side = f.texture_for_face(Face::PositiveX);
    for face in [Face::NegativeX, Face::PositiveZ, Face::NegativeZ] {
        assert_eq!(f.texture_for_face(face), side, "{face:?}");
    }
    assert_ne!(side, f.texture_for_face(Face::PositiveY), "end != side");
}

#[test]
fn bark_runs_vertically_in_darker_streaks_than_the_end_grain() {
    let e = env();
    let f = faces(&e);
    let side = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();
    let end = e.manager.get(f.texture_for_face(Face::PositiveY)).unwrap();
    let lum = |c: core::color::Color| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;

    // Vertical fiber: neighboring texels down a column are more alike than
    // neighboring texels along a row.
    let (mut dv, mut dh) = (0.0, 0.0);
    for y in 0..15 {
        for x in 0..15 {
            let c = lum(side.texel(x, y).unwrap());
            dv += (c - lum(side.texel(x, y + 1).unwrap())).abs();
            dh += (c - lum(side.texel(x + 1, y).unwrap())).abs();
        }
    }
    assert!(dv < dh, "columns are smoother than rows: {dv} vs {dh}");

    let mean =
        |t: &core::texture::CpuTexture| t.pixels().iter().map(|c| lum(*c)).sum::<f32>() / 256.0;
    assert!(mean(side) < mean(end), "bark is darker than the cut face");
}

#[test]
fn the_log_is_not_wood_planks_and_is_semimatte() {
    let e = env();
    let m = e.library.get(log_material_id()).unwrap();
    assert!(m.normal_texture.is_none());
    assert!(
        m.specular < 0.1 && m.reflectivity < 0.02,
        "no plastic shine"
    );
}

#[test]
fn the_faces_map_correctly_on_real_hits() {
    let e = env();
    let mut world = scene::voxel_world::VoxelWorld::new();
    world.insert(
        IVec3::new(0, 0, 0),
        scene::block::BlockInstance::new(BlockType::Log, log_material_id(), Orientation::Up),
    );
    let f = faces(&e);
    let cases = [
        ((0.5, 5.0, 0.5), (0.0, -1.0, 0.0), Face::PositiveY),
        ((0.5, -5.0, 0.5), (0.0, 1.0, 0.0), Face::NegativeY),
        ((5.0, 0.5, 0.5), (-1.0, 0.0, 0.0), Face::PositiveX),
        ((0.5, 0.5, 5.0), (0.0, 0.0, -1.0), Face::PositiveZ),
    ];
    for (o, d, expected) in cases {
        let r = core::ray::Ray::new(
            core::math::Vec3::new(o.0, o.1, o.2),
            core::math::Vec3::new(d.0, d.1, d.2),
        );
        let hit = renderer::raytracer::nearest_voxel_hit(&world, &r, 50.0).unwrap();
        assert_eq!(hit.hit.face, expected);
        let want = if matches!(expected, Face::PositiveY | Face::NegativeY) {
            f.texture_for_face(Face::PositiveY)
        } else {
            f.texture_for_face(Face::PositiveZ)
        };
        assert_eq!(f.texture_for_face(hit.hit.face), want);
    }
}
