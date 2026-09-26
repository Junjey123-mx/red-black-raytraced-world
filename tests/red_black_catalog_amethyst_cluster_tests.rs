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
use core::math::{IVec3, Vec3};
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    AMETHYST_X, CatalogScene, CatalogTextures, SHAPES_Z, SampleKind, catalog_entries,
    catalog_materials,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{amethyst_cluster_material_id, budding_amethyst_material_id};
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
        .get(amethyst_cluster_material_id())
        .unwrap()
        .face_textures
        .unwrap()
}

#[test]
fn the_amethyst_cluster_block_type_exists() {
    assert!(format!("{:?}", BlockType::AmethystCluster).contains("AmethystCluster"));
}

#[test]
fn the_geometry_is_the_reused_gate_06_composite_cluster() {
    let geometry = block_geometry(BlockType::AmethystCluster, Orientation::Up);
    assert!(matches!(geometry, BlockGeometry::Composite(_)));
    assert_eq!(
        geometry.part_count(),
        5,
        "one tall crystal plus four shorter"
    );

    // The exact Gate 06 crystals: the promotion never rebuilt the shape.
    let expected = [
        ([0.375, 0.0, 0.375], [0.625, 0.8125, 0.625]),
        ([0.125, 0.0, 0.4375], [0.3125, 0.5, 0.6875]),
        ([0.6875, 0.0, 0.3125], [0.875, 0.4375, 0.5625]),
        ([0.4375, 0.0, 0.6875], [0.6875, 0.375, 0.875]),
        ([0.3125, 0.0, 0.125], [0.5625, 0.5625, 0.3125]),
    ];
    let v = |a: [f32; 3]| Vec3::new(a[0], a[1], a[2]);
    for (part, (min, max)) in geometry.parts().iter().zip(expected) {
        assert!((part.min() - v(min)).length() < 1e-6, "{part:?}");
        assert!((part.max() - v(max)).length() < 1e-6, "{part:?}");
    }
}

#[test]
fn it_is_never_a_full_cube() {
    for o in [Orientation::Up, Orientation::Down, Orientation::North] {
        let geometry = block_geometry(BlockType::AmethystCluster, o);
        assert!(!geometry.is_full_cube(), "{o:?}");
        assert!(!matches!(geometry, BlockGeometry::FullCube), "{o:?}");
    }
}

#[test]
fn the_material_and_every_face_texture_resolve() {
    let e = env();
    assert!(e.library.contains(amethyst_cluster_material_id()));
    let f = faces(&e);
    for face in FACES {
        let t = e.manager.get(f.texture_for_face(face)).expect("loaded");
        assert_eq!((t.width(), t.height()), (16, 16), "{face:?}");
    }
}

#[test]
fn the_material_is_opaque_non_emissive_and_matches_its_profile() {
    let e = env();
    let m = e.library.get(amethyst_cluster_material_id()).unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none() && m.normal_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.10).abs() < 1e-6);
    assert!((m.shininess - 18.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.03).abs() < 1e-6);

    let f = faces(&e);
    let t = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();
    assert!(t.pixels().iter().all(|c| c.a == 1.0), "opaque facets");
}

#[test]
fn the_texture_reads_violet_lilac_with_light_tips() {
    let e = env();
    let t = e
        .manager
        .get(faces(&e).texture_for_face(Face::PositiveZ))
        .unwrap();

    let n = t.pixels().len() as f32;
    let (r, g, b) = t
        .pixels()
        .iter()
        .fold((0.0, 0.0, 0.0), |a, c| (a.0 + c.r, a.1 + c.g, a.2 + c.b));
    let (r, g, b) = (r / n, g / n, b / n);
    assert!(b > r && r > g, "violet/lilac: {r} {g} {b}");

    // Pale pink/cream crystal accents exist.
    let pale = t.pixels().iter().filter(|c| c.r > 0.9 && c.g > 0.7).count();
    assert!(pale >= 4, "light accents: {pale}");

    // Tips are lighter than the base: cell-space v = 0 is the top.
    let row_luma = |y: usize| {
        (0..16)
            .map(|x| {
                let c = t.texel(x, y).unwrap();
                0.299 * c.r + 0.587 * c.g + 0.114 * c.b
            })
            .sum::<f32>()
    };
    let top: f32 = (0..4).map(row_luma).sum();
    let base: f32 = (12..16).map(row_luma).sum();
    assert!(top > base, "light tips over a darker base: {top} vs {base}");
}

#[test]
fn it_is_the_official_catalog_entry_at_its_existing_slot() {
    let scene = CatalogScene::new();
    let entry = scene
        .entries()
        .iter()
        .find(|e| e.block_type == BlockType::AmethystCluster)
        .expect("in the catalog");
    assert_eq!(entry.display_name, "Amethyst cluster");
    assert_eq!(entry.kind, SampleKind::PartialGeometry);
    assert_eq!(entry.material_id, amethyst_cluster_material_id());
    assert_eq!(entry.position, IVec3::new(AMETHYST_X, 1, SHAPES_Z));
    assert_eq!(entry.orientation, Orientation::Up);
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));

    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    let c = Vec3::new(AMETHYST_X as f32 + 0.5, 1.5, SHAPES_Z as f32 + 0.5);
    assert!((p - c).length() < 1e-6);
}

#[test]
fn there_is_exactly_one_cluster_in_the_catalog() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::AmethystCluster)
        .count();
    assert_eq!(count, 1, "promoted, not duplicated");

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -14..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.block_type() == BlockType::AmethystCluster {
                        uses += 1;
                    }
                }
            }
        }
    }
    assert_eq!(uses, 1);
}

#[test]
fn it_stays_distinct_from_budding_amethyst() {
    assert_ne!(BlockType::AmethystCluster, BlockType::BuddingAmethyst);
    assert!(matches!(
        block_geometry(BlockType::BuddingAmethyst, Orientation::Up),
        BlockGeometry::FullCube
    ));
    assert_ne!(
        amethyst_cluster_material_id(),
        budding_amethyst_material_id()
    );

    let e = env();
    let cluster = faces(&e).texture_for_face(Face::PositiveZ);
    let budding = e
        .library
        .get(budding_amethyst_material_id())
        .unwrap()
        .face_textures
        .unwrap()
        .texture_for_face(Face::PositiveZ);
    assert_ne!(cluster, budding, "its own crystal texture");
}
