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
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::hit::Face;
use core::material::{AlphaMode, MaterialId};
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use core::texture::CpuTexture;
use renderer::normal_mapping::sample_shading_normal;
use renderer::raytracer::nearest_voxel_hit;
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_gallery::deepslate_bricks_material_id;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    RED_BLACK_BRICKS_GLOW_STRENGTH, red_black_deepslate_bricks_violet_material_id,
};
use scene::overworld_blocks::{
    red_black_deepslate_bricks_crimson_material_id, red_black_deepslate_bricks_orange_material_id,
};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

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

fn id() -> MaterialId {
    red_black_deepslate_bricks_violet_material_id()
}

fn rgb8(c: &core::color::Color) -> [u8; 3] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    ]
}

fn albedo_of(e: &Env, id: MaterialId) -> &CpuTexture {
    let f = e.library.get(id).unwrap().face_textures.unwrap();
    e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap()
}

/// Texels where this block departs from the DeepslateBricks albedo.
fn accents(e: &Env, id: MaterialId) -> Vec<bool> {
    let (own, base) = (
        albedo_of(e, id),
        albedo_of(e, deepslate_bricks_material_id()),
    );
    own.pixels()
        .iter()
        .zip(base.pixels())
        .map(|(a, b)| rgb8(a) != rgb8(b))
        .collect()
}

#[test]
fn the_block_type_exists_and_is_a_full_cube() {
    assert!(
        format!("{:?}", BlockType::RedBlackDeepslateBricksViolet)
            .contains("RedBlackDeepslateBricksViolet")
    );
    for o in [Orientation::Up, Orientation::Down, Orientation::South] {
        assert!(matches!(
            block_geometry(BlockType::RedBlackDeepslateBricksViolet, o),
            BlockGeometry::FullCube
        ));
    }
}

#[test]
fn it_reuses_the_deepslate_bricks_normal_map() {
    let e = env();
    let own = e.library.get(id()).expect("MaterialId resolves");
    let base = e.library.get(deepslate_bricks_material_id()).unwrap();
    let normal = own.normal_texture.expect("normal map");
    assert_eq!(
        Some(normal),
        base.normal_texture,
        "the very same normal texture"
    );
    assert!(e.manager.get(normal).is_some());
    let faces = own.face_textures.unwrap();
    for face in FACES {
        assert_eq!(
            faces.texture_for_face(face),
            faces.texture_for_face(Face::PositiveZ)
        );
    }
}

#[test]
fn the_albedo_is_the_deepslate_masonry_with_a_few_accents() {
    let e = env();
    let changed = accents(&e, id());
    let n = changed.iter().filter(|c| **c).count();
    assert!(
        (12..=40).contains(&n),
        "localized accents, never a global tint: {n}"
    );
    let own = albedo_of(&e, id());
    assert!(own.pixels().iter().all(|c| c.a == 1.0));
    // The untouched masonry keeps the charcoal body.
    let base = albedo_of(&e, deepslate_bricks_material_id());
    let body: Vec<_> = base
        .pixels()
        .iter()
        .zip(&changed)
        .filter(|(_, c)| !**c)
        .map(|(p, _)| (p.r + p.g + p.b) / 3.0)
        .collect();
    let mean = body.iter().sum::<f32>() / body.len() as f32;
    assert!(mean < 0.35, "dark masonry body: {mean}");
    for c in base
        .pixels()
        .iter()
        .zip(&changed)
        .filter(|(_, c)| !**c)
        .map(|(p, _)| p)
    {
        let spread = c.r.max(c.g).max(c.b) - c.r.min(c.g).min(c.b);
        assert!(spread < 0.08, "the body stays neutral grey {c:?}");
    }
}

#[test]
fn only_the_accent_joints_glow_and_only_faintly() {
    let e = env();
    let m = e.library.get(id()).unwrap();
    assert!(m.emission_strength > 0.0 && m.emission_strength <= 0.35);
    assert_eq!(m.emission_strength, RED_BLACK_BRICKS_GLOW_STRENGTH);
    let mask = e.manager.get(m.emissive_texture.expect("mask")).unwrap();
    let changed = accents(&e, id());
    let mut lit = 0;
    for (glow, accent) in mask.pixels().iter().zip(&changed) {
        if rgb8(glow) != [0, 0, 0] {
            assert!(*accent, "only recoloured texels glow");
            lit += 1;
        }
    }
    assert!(lit > 0 && lit < 30, "{lit}");
}

#[test]
fn the_material_stays_structural() {
    let e = env();
    let m = e.library.get(id()).unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.refractive_index, 1.0);
    assert_eq!(m.alpha_mode, AlphaMode::Ignore);
    assert!((m.specular - 0.10).abs() < 1e-6);
    assert!((m.shininess - 18.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.025).abs() < 1e-6);
}

#[test]
fn normal_mapping_is_active_and_relieves_the_bricks() {
    let e = env();
    let m = e.library.get(id()).unwrap();
    let tilted = (0..256)
        .filter(|n| {
            let uv = Vec2::new(
                ((n % 16) as f32 + 0.5) / 16.0,
                ((n / 16) as f32 + 0.5) / 16.0,
            );
            let s =
                sample_shading_normal(m, Face::PositiveZ, uv, Face::PositiveZ.normal(), &e.manager);
            s.z < 0.97
        })
        .count();
    assert!(tilted > 30 && tilted < 256, "{tilted}");
}

#[test]
fn the_geometric_normal_and_silhouette_are_preserved() {
    let mut world = VoxelWorld::new();
    world.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::RedBlackDeepslateBricksViolet,
            id(),
            Orientation::Up,
        ),
    );
    for (x, y) in [(0.1, 0.1), (0.5, 0.5), (0.9, 0.3)] {
        let r = Ray::new(Vec3::new(x, y, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = nearest_voxel_hit(&world, &r, 20.0).expect("full cube");
        assert_eq!(hit.hit.normal, Face::PositiveZ.normal());
        assert!((hit.hit.point.z - 1.0).abs() < 1e-4);
    }
    let r = Ray::new(Vec3::new(1.02, 0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
    assert!(
        nearest_voxel_hit(&world, &r, 20.0).is_none(),
        "no extra geometry"
    );
}

#[test]
fn the_accents_are_violet() {
    let e = env();
    let own = albedo_of(&e, id());
    for (c, accent) in own.pixels().iter().zip(accents(&e, id())) {
        if accent {
            assert!(c.b > c.r && c.r > c.g, "violet accent {c:?}");
            assert!(c.r >= c.b * 0.3, "never pure blue {c:?}");
        }
    }
}

#[test]
fn it_is_distinct_from_the_crimson_and_orange_bricks() {
    let e = env();
    let own = albedo_of(&e, id());
    for other in [
        red_black_deepslate_bricks_crimson_material_id(),
        red_black_deepslate_bricks_orange_material_id(),
    ] {
        assert_ne!(other, id());
        assert_eq!(accents(&e, other), accents(&e, id()));
        let theirs = albedo_of(&e, other);
        for ((a, b), accent) in own
            .pixels()
            .iter()
            .zip(theirs.pixels())
            .zip(accents(&e, id()))
        {
            if accent {
                assert!(a.b > a.r && b.r > b.b, "violet vs warm accent");
            }
        }
    }
}

#[test]
fn it_is_one_official_catalog_entry_with_a_valid_focus() {
    let matching: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::RedBlackDeepslateBricksViolet)
        .collect();
    assert_eq!(matching.len(), 1);
    let entry = &matching[0];
    assert_eq!(entry.display_name, "Red-Black deepslate bricks (violet)");
    assert_eq!(entry.kind, SampleKind::NormalMapped);
    assert_eq!(entry.material_id, id());
    let scene = CatalogScene::new();
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    assert!((p.x - (entry.position.x as f32 + 0.5)).abs() < 1e-6);
    assert!((p.z - (entry.position.z as f32 + 0.5)).abs() < 1e-6);
    let mut s = CatalogScene::new();
    while s.selected().block_type != BlockType::RedBlackDeepslateBricksViolet {
        s.select_next();
    }
    assert_eq!(s.focus_target(), p);
}

#[test]
fn it_reuses_the_crimson_accent_placement_and_the_same_relief() {
    let e = env();
    let crimson = red_black_deepslate_bricks_crimson_material_id();
    assert_ne!(id(), crimson);
    assert_eq!(
        accents(&e, id()),
        accents(&e, crimson),
        "same accent texels"
    );
    assert_eq!(
        e.library.get(id()).unwrap().normal_texture,
        e.library.get(crimson).unwrap().normal_texture
    );
    let (a, b) = (
        e.library.get(id()).unwrap(),
        e.library.get(crimson).unwrap(),
    );
    assert_eq!(
        (a.specular, a.shininess, a.reflectivity, a.emission_strength),
        (b.specular, b.shininess, b.reflectivity, b.emission_strength)
    );
    // Same masonry everywhere else, texel for texel.
    let (own, base) = (albedo_of(&e, id()), albedo_of(&e, crimson));
    for ((x, y), accent) in own
        .pixels()
        .iter()
        .zip(base.pixels())
        .zip(accents(&e, id()))
    {
        if !accent {
            assert_eq!(rgb8(x), rgb8(y));
        }
    }
}
