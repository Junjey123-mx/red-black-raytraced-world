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
use core::math::{IVec3, Vec2, Vec3};
use renderer::emission::sample_emissive;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    AMETHYST_X, CatalogScene, CatalogTextures, SHAPES_Z, SampleKind, catalog_entries,
    catalog_materials,
};
use scene::material_gallery::{glass_material_id, redstone_lamp_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{AMETHYST_CLUSTER_GLOW_STRENGTH, amethyst_cluster_material_id};
use scene::texture_manager::TextureManager;

/// Pixel fingerprint of the Gate 10 cluster albedo (commit 0315d42).
const ALBEDO_FINGERPRINT: u64 = 342949058642;

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

fn material(e: &Env) -> &core::material::Material {
    e.library.get(amethyst_cluster_material_id()).unwrap()
}

#[test]
fn the_five_prism_geometry_is_unchanged() {
    let geometry = block_geometry(BlockType::AmethystCluster, Orientation::Up);
    assert!(matches!(geometry, BlockGeometry::Composite(_)));
    let expected = [
        ([0.375, 0.0, 0.375], [0.625, 0.8125, 0.625]),
        ([0.125, 0.0, 0.4375], [0.3125, 0.5, 0.6875]),
        ([0.6875, 0.0, 0.3125], [0.875, 0.4375, 0.5625]),
        ([0.4375, 0.0, 0.6875], [0.6875, 0.375, 0.875]),
        ([0.3125, 0.0, 0.125], [0.5625, 0.5625, 0.3125]),
    ];
    assert_eq!(geometry.part_count(), expected.len());
    let v = |a: [f32; 3]| Vec3::new(a[0], a[1], a[2]);
    for (part, (min, max)) in geometry.parts().iter().zip(expected) {
        assert!((part.min() - v(min)).length() < 1e-6);
        assert!((part.max() - v(max)).length() < 1e-6);
    }
}

#[test]
fn strong_crystal_highlights_and_moderate_reflection() {
    let e = env();
    let m = material(&e);
    assert!((m.specular - 0.65).abs() < 1e-6, "specular {}", m.specular);
    assert!(
        (m.shininess - 110.0).abs() < 1e-6,
        "shininess {}",
        m.shininess
    );
    assert!(
        (m.reflectivity - 0.20).abs() < 1e-6,
        "reflectivity {}",
        m.reflectivity
    );
}

#[test]
fn slight_translucency_with_a_crystal_index_never_glass() {
    let e = env();
    let m = material(&e);
    assert!(
        (0.05..=0.15).contains(&m.transparency),
        "{}",
        m.transparency
    );
    assert!((m.refractive_index - 1.45).abs() < 1e-6);
    let glass = e.library.get(glass_material_id()).unwrap();
    assert!(m.transparency < glass.transparency / 5.0, "not glass");
}

#[test]
fn a_soft_violet_glow_never_a_saturated_lamp() {
    let e = env();
    let m = material(&e);
    assert!((0.4..=0.9).contains(&m.emission_strength));
    assert_eq!(m.emission_strength, AMETHYST_CLUSTER_GLOW_STRENGTH);
    let lamp = e.library.get(redstone_lamp_material_id()).unwrap();
    assert!(m.emission_strength < lamp.emission_strength / 2.0);

    // The glow comes from the crystal facets themselves: violet on average.
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    for j in 0..16 {
        for i in 0..16 {
            let uv = Vec2::new((i as f32 + 0.5) / 16.0, (j as f32 + 0.5) / 16.0);
            let c = sample_emissive(m, uv, &e.manager);
            assert!(c.r.max(c.g).max(c.b) <= 0.9, "never blown out: {c:?}");
            r += c.r;
            g += c.g;
            b += c.b;
        }
    }
    assert!(b > r && r > g && g > 0.0, "violet glow: {r} {g} {b}");
}

#[test]
fn the_albedo_is_unchanged() {
    let e = env();
    let faces = material(&e).face_textures.unwrap();
    let t = e
        .manager
        .get(faces.texture_for_face(Face::PositiveZ))
        .unwrap();
    let sum: u64 = t
        .pixels()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let [r, g, b] = [c.r, c.g, c.b].map(|v| (v * 255.0).round() as u64);
            (i as u64 + 1) * (r * 65536 + g * 256 + b)
        })
        .sum();
    assert_eq!(sum, ALBEDO_FINGERPRINT);
    assert_eq!(
        material(&e).emissive_texture,
        Some(faces.texture_for_face(Face::PositiveZ)),
        "the facets are their own emissive mask"
    );
}

#[test]
fn the_catalog_entry_and_focus_are_unchanged() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::AmethystCluster)
        .collect();
    assert_eq!(entries.len(), 1, "no duplication");
    let entry = &entries[0];
    assert_eq!(entry.display_name, "Amethyst cluster");
    assert_eq!(entry.kind, SampleKind::PartialGeometry);
    assert_eq!(entry.material_id, amethyst_cluster_material_id());
    assert_eq!(entry.position, IVec3::new(AMETHYST_X, 1, SHAPES_Z));
    assert_eq!(entry.orientation, Orientation::Up);
    let centre = Vec3::new(AMETHYST_X as f32 + 0.5, 1.5, SHAPES_Z as f32 + 0.5);
    assert!((entry.focus_point - centre).length() < 1e-6);

    let scene = CatalogScene::new();
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let mut s = CatalogScene::new();
    while s.selected().block_type != BlockType::AmethystCluster {
        s.select_next();
    }
    assert_eq!(s.focus_target(), centre);
}
