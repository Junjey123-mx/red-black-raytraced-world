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
use scene::overworld_blocks::deepslate_material_id;
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
        .get(deepslate_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

#[test]
fn deepslate_is_a_full_cube_block_type() {
    let geometry = block_geometry(BlockType::Deepslate, Orientation::Up);
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
    let m = e.library.get(deepslate_material_id()).unwrap();

    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.06).abs() < 1e-6, "specular {}", m.specular);
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
        .find(|e| e.block_type == BlockType::Deepslate)
        .expect("in the catalog");

    assert_eq!(entry.display_name, "Deepslate");
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, deepslate_material_id());
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn there_is_exactly_one_sample_of_this_block() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::Deepslate)
        .count();
    assert_eq!(count, 1);

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -10..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.material_id() == deepslate_material_id() {
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

fn mean_luminance(t: &core::texture::CpuTexture) -> f32 {
    let n = t.pixels().len() as f32;
    t.pixels()
        .iter()
        .map(|c| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b)
        .sum::<f32>()
        / n
}

#[test]
fn top_and_bottom_share_one_texture_and_the_sides_share_another() {
    let e = env();
    let f = faces(&e);
    assert_eq!(
        f.texture_for_face(Face::PositiveY),
        f.texture_for_face(Face::NegativeY)
    );
    let side = f.texture_for_face(Face::PositiveZ);
    for face in [Face::PositiveX, Face::NegativeX, Face::NegativeZ] {
        assert_eq!(f.texture_for_face(face), side);
    }
}

#[test]
fn deepslate_is_clearly_darker_than_cobblestone_but_not_black() {
    let e = env();
    let lum = |id| {
        let f = e.library.get(id).unwrap().face_textures.unwrap();
        mean_luminance(e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap())
    };
    let deep = lum(deepslate_material_id());
    let cobble = lum(scene::overworld_blocks::cobblestone_material_id());

    assert!(
        deep < cobble * 0.7,
        "deepslate {deep} vs cobblestone {cobble}"
    );
    assert!(deep > 0.08, "still readable, not black: {deep}");
}

#[test]
fn deepslate_has_no_normal_map_and_differs_from_deepslate_bricks() {
    let e = env();
    let m = e.library.get(deepslate_material_id()).unwrap();
    assert!(m.normal_texture.is_none());

    let bricks = e
        .library
        .get(scene::material_gallery::deepslate_bricks_material_id())
        .unwrap();
    assert!(bricks.normal_texture.is_some());
    assert_ne!(
        m.face_textures.unwrap().texture_for_face(Face::PositiveZ),
        bricks
            .face_textures
            .unwrap()
            .texture_for_face(Face::PositiveZ)
    );
}
