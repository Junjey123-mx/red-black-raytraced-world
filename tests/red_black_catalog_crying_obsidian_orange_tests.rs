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
use core::material::{AlphaMode, MaterialId};
use core::math::{IVec3, Vec2};
use core::texture::CpuTexture;
use renderer::emission::sample_emissive;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    CRYING_OBSIDIAN_GLOW_STRENGTH, crying_obsidian_crimson_material_id,
    crying_obsidian_orange_material_id,
};
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

fn id() -> MaterialId {
    crying_obsidian_orange_material_id()
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

fn mask_of(e: &Env, id: MaterialId) -> &CpuTexture {
    let m = e.library.get(id).unwrap();
    e.manager
        .get(m.emissive_texture.expect("emissive mask"))
        .unwrap()
}

/// The crack footprint: every texel the emissive mask lights.
fn cracks(mask: &CpuTexture) -> Vec<bool> {
    mask.pixels().iter().map(|c| rgb8(c) != [0, 0, 0]).collect()
}

fn luma(c: &core::color::Color) -> f32 {
    0.299 * c.r + 0.587 * c.g + 0.114 * c.b
}

#[test]
fn the_block_type_exists_and_is_a_full_cube() {
    assert!(format!("{:?}", BlockType::CryingObsidianOrange).contains("CryingObsidianOrange"));
    for o in [Orientation::Up, Orientation::Down, Orientation::South] {
        assert!(matches!(
            block_geometry(BlockType::CryingObsidianOrange, o),
            BlockGeometry::FullCube
        ));
    }
}

#[test]
fn the_albedo_emissive_mask_and_material_resolve() {
    let e = env();
    let m = e.library.get(id()).expect("MaterialId resolves");
    let faces = m.face_textures.expect("albedo");
    let first = faces.texture_for_face(Face::PositiveZ);
    for face in FACES {
        assert_eq!(
            faces.texture_for_face(face),
            first,
            "one pattern on every face"
        );
    }
    let a = e.manager.get(first).expect("albedo loaded");
    let mask = e
        .manager
        .get(m.emissive_texture.expect("mask"))
        .expect("mask loaded");
    assert_eq!((a.width(), a.height()), (16, 16));
    assert_eq!((mask.width(), mask.height()), (16, 16));
    assert!(a.pixels().iter().all(|c| c.a == 1.0));
}

#[test]
fn the_material_is_opaque_reflective_and_glowing() {
    let e = env();
    let m = e.library.get(id()).unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.refractive_index, 1.0);
    assert_eq!(m.alpha_mode, AlphaMode::Ignore);
    assert!((m.specular - 0.30).abs() < 1e-6);
    assert!((m.shininess - 60.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.12).abs() < 1e-6, "moderate reflection");
    assert!(m.emission_strength > 0.0);
    assert_eq!(m.emission_strength, CRYING_OBSIDIAN_GLOW_STRENGTH);
    assert!((1.0..=1.8).contains(&m.emission_strength));
}

#[test]
fn only_the_cracks_emit_and_the_body_stays_dark() {
    let e = env();
    let (a, mask) = (albedo_of(&e, id()), mask_of(&e, id()));
    let crack = cracks(mask);
    let lit = crack.iter().filter(|c| **c).count();
    assert!(
        (30..=100).contains(&lit),
        "localized energy, not a lit cube: {lit}"
    );

    let mut body = Vec::new();
    for (k, (texel, is_crack)) in a.pixels().iter().zip(&crack).enumerate() {
        if *is_crack {
            let peak = texel.r.max(texel.g).max(texel.b);
            assert!(peak > 0.38, "texel {k}: cracks are bright ({peak})");
        } else {
            assert_eq!(rgb8(&mask.pixels()[k]), [0, 0, 0], "body texel {k} emits");
            body.push(luma(texel));
        }
    }
    let mean = body.iter().sum::<f32>() / body.len() as f32;
    assert!(mean < 0.12, "near-black obsidian body: {mean}");

    // Through the renderer's own emissive path.
    let material = e.library.get(id()).unwrap();
    for (k, is_crack) in crack.iter().enumerate() {
        let uv = Vec2::new(
            ((k % 16) as f32 + 0.5) / 16.0,
            ((k / 16) as f32 + 0.5) / 16.0,
        );
        let c = sample_emissive(material, uv, &e.manager);
        assert_eq!(*is_crack, c.r + c.g + c.b > 0.0, "texel {k}");
    }
    // The one-texel frame is solid obsidian.
    for k in 0..16 {
        for (x, y) in [(k, 0), (k, 15), (0, k), (15, k)] {
            assert!(!crack[y * 16 + x], "frame ({x},{y})");
        }
    }
}

#[test]
fn the_cracks_and_their_glow_are_warm_orange() {
    let e = env();
    let (a, mask) = (albedo_of(&e, id()), mask_of(&e, id()));
    for (texel, glow) in a.pixels().iter().zip(mask.pixels()) {
        if rgb8(glow) != [0, 0, 0] {
            // The crack itself: warm orange, neither crimson nor yellow.
            assert!(texel.r > texel.g && texel.g > texel.b, "orange {texel:?}");
            let ratio = texel.g / texel.r;
            assert!((0.35..0.6).contains(&ratio), "crack {ratio}");
            assert!(texel.b < 0.02, "warm {texel:?}");
            // Its glow: the deeper orange of the Gate 10.5 card suits, so the
            // lit crack never drifts to yellow.
            let ratio = glow.g / glow.r;
            assert!((0.2..0.35).contains(&ratio), "glow {ratio}");
            assert!(glow.b < 0.02, "warm {glow:?}");
        }
    }
}

#[test]
fn it_is_one_official_catalog_entry_with_a_valid_focus() {
    let matching: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::CryingObsidianOrange)
        .collect();
    assert_eq!(matching.len(), 1);
    let entry = &matching[0];
    assert_eq!(entry.display_name, "Crying obsidian (orange)");
    assert_eq!(entry.kind, SampleKind::Emissive);
    assert_eq!(entry.material_id, id());
    let scene = CatalogScene::new();
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let mut cells = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -24..12 {
                if scene
                    .world()
                    .get(IVec3::new(x, y, z))
                    .is_some_and(|b| b.material_id() == id())
                {
                    cells += 1;
                }
            }
        }
    }
    assert_eq!(cells, 1);

    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    let c = entry.position;
    assert!((p.x - (c.x as f32 + 0.5)).abs() < 1e-6);
    assert!((p.z - (c.z as f32 + 0.5)).abs() < 1e-6);
    let mut s = CatalogScene::new();
    while s.selected().block_type != BlockType::CryingObsidianOrange {
        s.select_next();
    }
    assert_eq!(s.focus_target(), p);
}

#[test]
fn it_reuses_the_crimson_crack_footprint_and_body() {
    let e = env();
    let crimson = crying_obsidian_crimson_material_id();
    assert_ne!(id(), crimson);
    assert_eq!(
        cracks(mask_of(&e, id())),
        cracks(mask_of(&e, crimson)),
        "same cracks"
    );

    let (own, base) = (albedo_of(&e, id()), albedo_of(&e, crimson));
    let crack = cracks(mask_of(&e, crimson));
    for (k, (a, b)) in own.pixels().iter().zip(base.pixels()).enumerate() {
        if crack[k] {
            assert_ne!(rgb8(a), rgb8(b), "texel {k}: the crack is recoloured");
        } else {
            assert_eq!(
                rgb8(a),
                rgb8(b),
                "texel {k}: the obsidian body is identical"
            );
        }
    }
    // Same relative brightness along the cracks: only the palette moved.
    for (k, (a, b)) in own.pixels().iter().zip(base.pixels()).enumerate() {
        if crack[k] {
            let (pa, pb) = (a.r.max(a.g).max(a.b), b.r);
            assert!(
                (pa / pb - 255.0 / 210.0).abs() < 0.1 || pa > 0.95,
                "texel {k}: {pa} vs {pb}"
            );
        }
    }
}

#[test]
fn it_shares_the_crimson_material_response() {
    let e = env();
    let own = e.library.get(id()).unwrap();
    let base = e
        .library
        .get(crying_obsidian_crimson_material_id())
        .unwrap();
    assert_eq!(
        (
            own.specular,
            own.shininess,
            own.reflectivity,
            own.transparency,
            own.refractive_index
        ),
        (
            base.specular,
            base.shininess,
            base.reflectivity,
            base.transparency,
            base.refractive_index
        )
    );
    assert_eq!(own.emission_strength, base.emission_strength);
}
