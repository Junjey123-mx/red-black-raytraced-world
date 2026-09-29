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
use core::math::{IVec3, Vec2};
use core::texture::CpuTexture;
use renderer::emission::sample_emissive;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{CatalogScene, CatalogTextures, catalog_entries, catalog_materials};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    ORANGE_GLOW_STRENGTH, orange_club_material_id, orange_spade_material_id,
};
use scene::texture_manager::TextureManager;

/// Albedo tiers of the family (Bright, Medium, Dark outline, light accent).
const TIERS: [[u8; 3]; 4] = [[255, 122, 0], [233, 79, 0], [165, 54, 0], [255, 222, 170]];

/// (block, material, catalog name, fingerprint of the Gate 10 albedo pixels).
fn blocks() -> Vec<(BlockType, MaterialId, &'static str, u64)> {
    vec![
        (
            BlockType::OrangeClub,
            orange_club_material_id(),
            "Orange club",
            173947623631,
        ),
        (
            BlockType::OrangeSpade,
            orange_spade_material_id(),
            "Orange spade",
            197471139422,
        ),
    ]
}

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

fn rgb8(c: &core::color::Color) -> [u8; 3] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    ]
}

fn albedo<'a>(e: &'a Env, id: MaterialId) -> &'a CpuTexture {
    let f = e.library.get(id).unwrap().face_textures.unwrap();
    e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap()
}

fn mask<'a>(e: &'a Env, id: MaterialId) -> &'a CpuTexture {
    let m = e.library.get(id).unwrap();
    e.manager
        .get(m.emissive_texture.expect("emissive mask"))
        .expect("mask loaded")
}

/// Texels of the glowing symbol: its body, bevel and light accent. The Dark
/// outline stays an unlit edge next to the body; only the thin parts that
/// are nothing but outline at 16 px (a stem, a foot) glow, so the lit
/// symbol keeps its whole silhouette.
fn glows(a: &CpuTexture, x: usize, y: usize) -> bool {
    let body = |x: i32, y: i32| {
        (0..16).contains(&x)
            && (0..16).contains(&y)
            && [TIERS[0], TIERS[1], TIERS[3]]
                .contains(&rgb8(&a.texel(x as usize, y as usize).unwrap()))
    };
    let (xi, yi) = (x as i32, y as i32);
    if body(xi, yi) {
        return true;
    }
    rgb8(&a.texel(x, y).unwrap()) == TIERS[2]
        && ![(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .any(|(dx, dy)| body(xi + dx, yi + dy))
}

#[test]
fn every_block_of_the_family_has_an_emissive_mask() {
    let e = env();
    for (block, id, name, _) in blocks() {
        let m = e.library.get(id).unwrap();
        let mask_id = m.emissive_texture.expect(name);
        let t = e.manager.get(mask_id).expect(name);
        assert_eq!((t.width(), t.height()), (16, 16), "{name}");
        assert!(t.pixels().iter().all(|c| c.a == 1.0), "{name}");
        let faces = m.face_textures.unwrap();
        assert_ne!(mask_id, faces.texture_for_face(Face::PositiveZ), "{name}");
        assert!(matches!(
            block_geometry(block, Orientation::Up),
            BlockGeometry::FullCube
        ));
    }
}

#[test]
fn the_emission_strength_is_positive_and_shared_by_the_family() {
    let e = env();
    assert!(
        ORANGE_GLOW_STRENGTH >= 1.2 && ORANGE_GLOW_STRENGTH <= 2.5,
        "{}",
        ORANGE_GLOW_STRENGTH
    );
    for (_, id, name, _) in blocks() {
        let m = e.library.get(id).unwrap();
        assert!(m.emission_strength > 0.0, "{name}");
        assert_eq!(m.emission_strength, ORANGE_GLOW_STRENGTH, "{name}");
    }
}

#[test]
fn the_symbol_emits() {
    let e = env();
    for (_, id, name, _) in blocks() {
        let (a, m) = (albedo(&e, id), mask(&e, id));
        let mut lit = 0;
        for y in 0..16 {
            for x in 0..16 {
                if glows(a, x, y) {
                    assert_ne!(rgb8(&m.texel(x, y).unwrap()), [0, 0, 0], "{name} ({x},{y})");
                    lit += 1;
                }
            }
        }
        assert!(lit >= 30, "{name}: {lit} glowing texels");
        // Sampled through the existing emissive path, not only on disk: the
        // body texels (Bright tier) glow when rendered.
        let material = e.library.get(id).unwrap();
        for y in 0..16 {
            for x in 0..16 {
                if rgb8(&a.texel(x, y).unwrap()) == TIERS[0] {
                    let uv = Vec2::new((x as f32 + 0.5) / 16.0, (y as f32 + 0.5) / 16.0);
                    let c = sample_emissive(material, uv, &e.manager);
                    assert!(c.r + c.g + c.b > 0.3, "{name} ({x},{y}) glows: {c:?}");
                }
            }
        }
    }
}

#[test]
fn the_dark_background_outline_and_joints_do_not_emit() {
    let e = env();
    for (_, id, name, _) in blocks() {
        let (a, m) = (albedo(&e, id), mask(&e, id));
        let mut dark = 0;
        for y in 0..16 {
            for x in 0..16 {
                if !glows(a, x, y) {
                    assert_eq!(rgb8(&m.texel(x, y).unwrap()), [0, 0, 0], "{name} ({x},{y})");
                    dark += 1;
                }
            }
        }
        // Never a whole-cube lamp: most of the face stays unlit.
        assert!(dark >= 150, "{name}: only {dark} unlit texels");
        let material = e.library.get(id).unwrap();
        for uv in [
            Vec2::new(0.02, 0.02),
            Vec2::new(0.98, 0.5),
            Vec2::new(0.5, 0.98),
        ] {
            let c = sample_emissive(material, uv, &e.manager);
            assert_eq!(rgb8(&c), [0, 0, 0], "{name} frame at {uv:?}");
        }
    }
}

#[test]
fn the_glow_is_warm_orange_amber() {
    let e = env();
    for (_, id, name, _) in blocks() {
        let m = mask(&e, id);
        let (mut r, mut g, mut b, mut n) = (0.0, 0.0, 0.0, 0.0);
        for c in m.pixels().iter().filter(|c| rgb8(c) != [0, 0, 0]) {
            // Warm: red leads, green is clearly present, blue nearly absent.
            assert!(c.r > c.g && c.g > c.b, "{name}: {c:?}");
            assert!(c.b < 0.05, "{name}: {c:?} is not warm");
            r += c.r;
            g += c.g;
            b += c.b;
            n += 1.0;
        }
        let (r, g, b) = (r / n, g / n, b / n);
        let ratio = g / r;
        assert!(
            (0.2..0.55).contains(&ratio),
            "{name}: orange-amber (green/red {ratio}), neither crimson nor yellow"
        );
        assert!(b < 0.03, "{name}: {b}");
    }
}

#[test]
fn no_crimson_dominant_texel_is_left() {
    let e = env();
    for (_, id, name, _) in blocks() {
        for c in mask(&e, id)
            .pixels()
            .iter()
            .filter(|c| rgb8(c) != [0, 0, 0])
        {
            // Crimson glow has green below a quarter of red.
            assert!(c.g >= c.r * 0.2, "{name}: crimson-like texel {c:?}");
        }
    }
}

#[test]
fn the_thin_stem_and_foot_glow_as_part_of_the_symbol() {
    let e = env();
    for (_, id, name, _) in blocks() {
        let m = mask(&e, id);
        // Row 13 is the flared foot of both the club and the spade.
        let foot = (0..16)
            .filter(|x| rgb8(&m.texel(*x, 13).unwrap()) != [0, 0, 0])
            .count();
        assert!(foot >= 4, "{name}: the foot glows ({foot})");
        assert_ne!(rgb8(&m.texel(7, 12).unwrap()), [0, 0, 0], "{name}: stem");
    }
}

#[test]
fn the_glow_never_washes_out_the_texture() {
    let e = env();
    for (_, id, name, _) in blocks() {
        let m = mask(&e, id);
        let strength = e.library.get(id).unwrap().emission_strength;
        for c in m.pixels() {
            // The glow alone stays within displayable range, so the lit
            // albedo still shapes the symbol instead of one clipped flat
            // colour.
            let emitted = c.r.max(c.g).max(c.b) * strength;
            assert!(emitted <= 1.1, "{name} over-bright: {emitted}");
        }
        // Body and bevel glow with different intensities.
        let mut levels: Vec<[u8; 3]> = m.pixels().iter().map(rgb8).collect();
        levels.sort_unstable();
        levels.dedup();
        assert!(levels.len() >= 3, "{name}: black + at least two glow tiers");
    }
}

#[test]
fn the_blocks_stay_opaque_with_their_gate_10_albedo() {
    let e = env();
    for (_, id, name, fingerprint) in blocks() {
        let m = e.library.get(id).unwrap();
        assert_eq!(m.transparency, 0.0, "{name}");
        assert_eq!(m.refractive_index, 1.0, "{name}");
        assert_eq!(m.alpha_mode, AlphaMode::Ignore, "{name}");
        assert!(m.normal_texture.is_none(), "{name}");
        let a = albedo(&e, id);
        assert!(a.pixels().iter().all(|c| c.a == 1.0), "{name}");
        let sum: u64 = a
            .pixels()
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let [r, g, b] = rgb8(c);
                (i as u64 + 1) * (r as u64 * 65536 + g as u64 * 256 + b as u64)
            })
            .sum();
        assert_eq!(sum, fingerprint, "{name}: the Gate 10 albedo is untouched");
    }
}

#[test]
fn block_types_and_catalog_entries_are_preserved() {
    let scene = CatalogScene::new();
    for (block, id, name, _) in blocks() {
        let matching: Vec<_> = catalog_entries()
            .into_iter()
            .filter(|e| e.block_type == block)
            .collect();
        assert_eq!(matching.len(), 1, "{name}: no duplicate entry");
        let entry = &matching[0];
        assert_eq!(entry.display_name, name);
        assert_eq!(entry.material_id, id);
        assert_eq!(scene.world().get(entry.position), Some(&entry.block()));

        let mut cells = 0;
        for x in -1..14 {
            for y in -1..5 {
                for z in -14..12 {
                    if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                        if b.material_id() == id {
                            cells += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cells, 1, "{name}");

        let mut s = CatalogScene::new();
        while s.selected().block_type != block {
            s.select_next();
        }
        assert_eq!(s.focus_target(), entry.focus_point, "{name} focus");
    }
}
