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
use scene::overworld_blocks::orange_club_material_id;
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
        .get(orange_club_material_id())
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
fn the_orange_club_block_type_exists() {
    assert!(format!("{:?}", BlockType::OrangeClub).contains("OrangeClub"));
}

#[test]
fn it_is_a_full_cube_in_every_orientation() {
    for o in [Orientation::Up, Orientation::Down, Orientation::South] {
        assert!(matches!(
            block_geometry(BlockType::OrangeClub, o),
            BlockGeometry::FullCube
        ));
    }
}

#[test]
fn the_material_and_every_face_texture_resolve() {
    let e = env();
    assert!(e.library.contains(orange_club_material_id()));
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
    let m = e.library.get(orange_club_material_id()).unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert!(m.normal_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    assert!((m.specular - 0.06).abs() < 1e-6, "specular {}", m.specular);
    assert!(
        (m.shininess - 8.0).abs() < 1e-6,
        "shininess {}",
        m.shininess
    );
    assert!(
        (m.reflectivity - 0.01).abs() < 1e-6,
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
        .find(|e| e.block_type == BlockType::OrangeClub)
        .expect("in the catalog");
    assert_eq!(entry.display_name, "Orange club");
    assert_eq!(entry.kind, SampleKind::PlainBlock);
    assert_eq!(entry.material_id, orange_club_material_id());
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
    while s.selected().block_type != BlockType::OrangeClub {
        s.select_next();
    }
    assert_eq!(s.focus_target(), p);
}

#[test]
fn there_is_exactly_one_sample_of_this_block() {
    let count = catalog_entries()
        .iter()
        .filter(|e| e.block_type == BlockType::OrangeClub)
        .count();
    assert_eq!(count, 1);

    let scene = CatalogScene::new();
    let mut uses = 0;
    for x in -1..14 {
        for y in -1..5 {
            for z in -14..12 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if b.material_id() == orange_club_material_id() {
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

/// Family tiers (Bright, Medium, Dark) and the controlled light accent.
const TIERS: [[u8; 3]; 4] = [[255, 122, 0], [233, 79, 0], [165, 54, 0], [255, 222, 170]];

fn rgb8(c: &core::color::Color) -> [u8; 3] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    ]
}

/// Texels painted with the symbol palette (the rest is the dark masonry).
fn symbol_mask(t: &core::texture::CpuTexture) -> [[bool; 16]; 16] {
    let mut m = [[false; 16]; 16];
    for (y, row) in m.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            *cell = TIERS.contains(&rgb8(&t.texel(x, y).unwrap()));
        }
    }
    m
}

fn symbol_mean(t: &core::texture::CpuTexture) -> (f32, f32, f32) {
    let m = symbol_mask(t);
    let (mut r, mut g, mut b, mut n) = (0.0, 0.0, 0.0, 0.0);
    for y in 0..16 {
        for x in 0..16 {
            if m[y][x] {
                let c = t.texel(x, y).unwrap();
                r += c.r;
                g += c.g;
                b += c.b;
                n += 1.0;
            }
        }
    }
    (r / n, g / n, b / n)
}

fn symbol_rows(m: &[[bool; 16]; 16]) -> Vec<usize> {
    (0..16).filter(|y| m[*y].iter().any(|v| *v)).collect()
}

fn width(row: &[bool; 16]) -> usize {
    row.iter().filter(|v| **v).count()
}

fn runs(row: &[bool; 16]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut x = 0;
    while x < 16 {
        if row[x] {
            let start = x;
            while x < 16 && row[x] {
                x += 1;
            }
            out.push((start, x - 1));
        } else {
            x += 1;
        }
    }
    out
}

fn assert_symmetric(m: &[[bool; 16]; 16]) {
    for (y, row) in m.iter().enumerate() {
        for x in 0..8 {
            assert_eq!(row[x], row[15 - x], "mirror symmetric at ({x},{y})");
        }
    }
}

#[test]
fn the_symbol_is_centred_and_dominant_on_a_dark_masonry_ground() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    let m = symbol_mask(t);
    let count = m.iter().flatten().filter(|v| **v).count();
    assert!(
        (70..=140).contains(&count),
        "symbol covers the face: {count}"
    );
    // The one-texel frame stays dark: the symbol never touches the edge.
    for k in 0..16 {
        assert!(!m[0][k] && !m[15][k] && !m[k][0] && !m[k][15]);
    }
    let mut ground = Vec::new();
    for y in 0..16 {
        for x in 0..16 {
            if !m[y][x] {
                ground.push(t.texel(x, y).unwrap());
            }
        }
    }
    let luma = ground
        .iter()
        .map(|c| 0.299 * c.r + 0.587 * c.g + 0.114 * c.b)
        .sum::<f32>()
        / ground.len() as f32;
    assert!(luma < 0.15, "dark Red-Black ground: {luma}");
}

#[test]
fn it_is_a_reinterpreted_block_texture_not_a_flat_pasted_icon() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    // Re-drawn at the catalog's 16x16 pixel-art scale (the guide is 64x64).
    assert_eq!((t.width(), t.height()), (16, 16));
    // Shaded like a block: outline (Dark), body (Bright), bevel (Medium).
    let m = symbol_mask(t);
    for tier in &TIERS[..3] {
        let n = (0..16)
            .flat_map(|y| (0..16).map(move |x| (x, y)))
            .filter(|(x, y)| m[*y][*x] && rgb8(&t.texel(*x, *y).unwrap()) == *tier)
            .count();
        assert!(n >= 5, "tier {tier:?} used {n} times");
    }
    // The ground is masonry (several tones), not one flat fill.
    let mut ground: Vec<[u8; 3]> = (0..16)
        .flat_map(|y| (0..16).map(move |x| (x, y)))
        .filter(|(x, y)| !m[*y][*x])
        .map(|(x, y)| rgb8(&t.texel(x, y).unwrap()))
        .collect();
    ground.sort_unstable();
    ground.dedup();
    assert!(ground.len() >= 3, "masonry tones: {}", ground.len());
}

#[test]
fn the_palette_is_warm_orange_not_the_original_red() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    let (r, g, b) = symbol_mean(t);
    assert!(r > 0.6, "bright warm symbol: {r}");
    let ratio = g / r;
    assert!(
        (0.28..0.6).contains(&ratio),
        "orange (green/red {ratio}), not red"
    );
    assert!(b < 0.05, "no blue in warm orange: {b}");
    for [r, g, b] in &TIERS[..3] {
        assert!(
            *g as f32 / *r as f32 > 0.28 && *b == 0,
            "orange tier {r} {g} {b}"
        );
    }
}

#[test]
fn the_symbol_is_a_recognisable_club() {
    let e = env();
    let m = symbol_mask(texture(&e, Face::PositiveZ));
    assert_symmetric(&m);
    let rows = symbol_rows(&m);
    let top = rows[0];
    let bottom = *rows.last().unwrap();
    assert_eq!(runs(&m[top]).len(), 1, "one top lobe");
    // The top lobe is separated from the two side lobes by notches.
    let notched = rows.iter().filter(|y| runs(&m[**y]).len() == 3).count();
    assert!(notched >= 2, "three lobes with notches between them");
    let widest = rows.iter().map(|y| width(&m[*y])).max().unwrap();
    assert!(
        width(&m[top]) < widest / 2,
        "the top lobe is narrower than the pair"
    );
    // A narrow stem and a wider flared foot.
    assert!(
        rows.iter().any(|y| *y > top + 5 && width(&m[*y]) == 2),
        "stem"
    );
    assert!(width(&m[bottom]) >= 4, "flared foot");
}

#[test]
fn no_original_red_survives_in_the_symbol() {
    let e = env();
    let t = texture(&e, Face::PositiveZ);
    // The club.png reds (#D21F2B, #7A1018) are gone.
    for red in [[210u8, 31, 43], [122, 16, 24]] {
        assert!(t.pixels().iter().all(|c| rgb8(c) != red), "{red:?}");
    }
}
