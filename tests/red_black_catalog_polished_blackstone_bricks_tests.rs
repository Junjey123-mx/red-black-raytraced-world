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
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
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
use scene::material_gallery::deepslate_bricks_material_id;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::polished_blackstone_bricks_material_id;
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
        .get(polished_blackstone_bricks_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

fn texture(e: &Env, face: Face) -> &core::texture::CpuTexture {
    e.manager.get(faces(e).texture_for_face(face)).unwrap()
}

#[allow(dead_code)]
fn mean_rgb(t: &core::texture::CpuTexture) -> (f32, f32, f32) {
    let n = t.pixels().len() as f32;
    let (r, g, b) = t
        .pixels()
        .iter()
        .fold((0.0, 0.0, 0.0), |a, c| (a.0 + c.r, a.1 + c.g, a.2 + c.b));
    (r / n, g / n, b / n)
}

#[test]
fn the_polished_blackstone_bricks_block_type_exists() {
    assert!(
        format!("{:?}", BlockType::PolishedBlackstoneBricks).contains("PolishedBlackstoneBricks")
    );
}

#[test]
fn it_is_a_full_cube_in_every_orientation() {
    for o in [Orientation::Up, Orientation::Down, Orientation::South] {
        assert!(matches!(
            block_geometry(BlockType::PolishedBlackstoneBricks, o),
            BlockGeometry::FullCube
        ));
    }
}

#[test]
fn the_material_and_every_face_texture_resolve() {
    let e = env();
    assert!(e.library.contains(polished_blackstone_bricks_material_id()));
    let f = faces(&e);
    for face in FACES {
        let t = e.manager.get(f.texture_for_face(face)).expect("loaded");
        assert_eq!((t.width(), t.height()), (16, 16), "pixel-art tile {face:?}");
    }
}

#[test]
fn the_face_texture_policy_matches_the_block() {
    let e = env();
    let f = faces(&e);
    if true {
        let first = f.texture_for_face(Face::PositiveZ);
        for face in FACES {
            assert_eq!(f.texture_for_face(face), first, "{face:?}");
        }
    } else {
        for face in [Face::PositiveX, Face::NegativeX, Face::NegativeZ] {
            assert_eq!(
                f.texture_for_face(face),
                f.texture_for_face(Face::PositiveZ),
                "the four sides share one texture"
            );
        }
        assert_ne!(
            f.texture_for_face(Face::PositiveY),
            f.texture_for_face(Face::PositiveZ)
        );
    }
}

#[test]
fn the_material_is_opaque_non_emissive_and_matches_its_profile() {
    let e = env();
    let m = e
        .library
        .get(polished_blackstone_bricks_material_id())
        .unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert!(m.normal_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.20).abs() < 1e-6, "specular {}", m.specular);
    assert!(
        (m.shininess - 38.0).abs() < 1e-6,
        "shininess {}",
        m.shininess
    );
    assert!(
        (m.reflectivity - 0.06).abs() < 1e-6,
        "reflectivity {}",
        m.reflectivity
    );
}

#[test]
fn every_texel_is_fully_opaque() {
    let e = env();
    for face in FACES {
        assert!(
            texture(&e, face).pixels().iter().all(|c| c.a == 1.0),
            "{face:?}"
        );
    }
}

#[test]
fn it_is_an_official_catalog_entry_with_a_finite_focus_point() {
    let scene = CatalogScene::new();
    let entry = scene
        .entries()
        .iter()
        .find(|e| e.block_type == BlockType::PolishedBlackstoneBricks)
        .expect("in the catalog");
    assert_eq!(entry.display_name, "Polished blackstone bricks");
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, polished_blackstone_bricks_material_id());
    assert_eq!(entry.orientation, Orientation::Up);
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    let c = entry.position;
    assert!((p.x - (c.x as f32 + 0.5)).abs() < 1e-6);
    assert!((p.y - (c.y as f32 + 0.5)).abs() < 1e-6);
    assert!((p.z - (c.z as f32 + 0.5)).abs() < 1e-6);

    // Selecting it and focusing puts the orbit target on the block.
    let mut s = CatalogScene::new();
    while s.selected().block_type != BlockType::PolishedBlackstoneBricks {
        s.select_next();
    }
    assert_eq!(s.focus_target(), p);
}

#[test]
fn there_is_exactly_one_sample_of_this_block() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::PolishedBlackstoneBricks)
        .count();
    assert_eq!(count, 1);

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -24..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.material_id() == polished_blackstone_bricks_material_id() {
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
    let tex = texture(&e, Face::PositiveZ);
    for (i, j) in [(0usize, 0usize), (5, 3), (15, 15), (8, 12)] {
        let expected = tex.texel(i, j).unwrap();
        for (du, dv) in [(0.05, 0.05), (0.5, 0.5), (0.95, 0.95)] {
            let uv = Vec2::new((i as f32 + du) / 16.0, (j as f32 + dv) / 16.0);
            assert_eq!(sample_nearest(tex, uv), expected, "texel ({i},{j})");
        }
    }
}

fn luma(c: &core::color::Color) -> f32 {
    0.299 * c.r + 0.587 * c.g + 0.114 * c.b
}

fn row_luma(t: &core::texture::CpuTexture, y: usize) -> f32 {
    (0..16).map(|x| luma(&t.texel(x, y).unwrap())).sum::<f32>() / 16.0
}

#[test]
fn it_reads_as_black_anthracite_masonry() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    let mean = t.pixels().iter().map(luma).sum::<f32>() / 256.0;
    assert!(mean < 0.22, "black/anthracite: {mean}");
    for c in t.pixels() {
        let spread = c.r.max(c.g).max(c.b) - c.r.min(c.g).min(c.b);
        assert!(spread < 0.1, "neutral dark stone, no colour cast {c:?}");
    }
}

#[test]
fn it_has_regular_bricks_with_dark_joints_and_brighter_edges() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    let mean = t.pixels().iter().map(luma).sum::<f32>() / 256.0;
    // Two courses of bricks: dark mortar rows 7 and 15 ...
    for y in [7, 15] {
        assert!(row_luma(t, y) < mean * 0.6, "joint row {y}");
    }
    // ... under slightly brighter top edges (rows 0 and 8).
    for y in [0, 8] {
        assert!(row_luma(t, y) > mean, "edge row {y}");
    }
}

#[test]
fn it_is_more_polished_than_deepslate_bricks_yet_not_metallic() {
    let e = env();
    let own = e
        .library
        .get(polished_blackstone_bricks_material_id())
        .unwrap();
    let bricks = e.library.get(deepslate_bricks_material_id()).unwrap();
    assert!(own.reflectivity > bricks.reflectivity);
    assert!(own.specular > bricks.specular);
    // Not a metal or mirror: low reflectivity, moderate highlight.
    assert!(own.reflectivity <= 0.1 && own.specular <= 0.3);

    let own_tex = texture(&e, Face::PositiveZ);
    let bricks_faces = bricks.face_textures.unwrap();
    assert_ne!(
        faces(&e).texture_for_face(Face::PositiveZ),
        bricks_faces.texture_for_face(Face::PositiveZ)
    );
    let bricks_tex = e
        .manager
        .get(bricks_faces.texture_for_face(Face::PositiveZ))
        .unwrap();
    let (a, b) = (
        own_tex.pixels().iter().map(luma).sum::<f32>() / 256.0,
        bricks_tex.pixels().iter().map(luma).sum::<f32>() / 256.0,
    );
    assert!(a < b - 0.05, "darker than deepslate bricks: {a} vs {b}");
}
