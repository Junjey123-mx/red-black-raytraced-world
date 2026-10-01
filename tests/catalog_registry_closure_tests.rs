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

use camera::diagnostic::DiagnosticCameraState;
use core::hit::Face;
use core::material::AlphaMode;
use core::math::{IVec3, Vec3};
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{
    CatalogScene, CatalogTextures, FOCUS_DISTANCE, SampleKind, catalog_entries, catalog_materials,
    official_entries,
};
use scene::material_gallery::gallery_camera;
use scene::material_library::MaterialLibrary;
use scene::texture_manager::TextureManager;
use std::collections::HashSet;

const FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];

/// The blocks whose shape is not a full cube (Gate 06/07 partial geometry).
const PARTIAL: [BlockType; 7] = [
    BlockType::WoodStairs,
    BlockType::Fence,
    BlockType::WoodDoor,
    BlockType::PortalCoreDarkCrimson,
    BlockType::AmethystCluster,
    BlockType::RedBlackFence,
    BlockType::RedBlackWoodDoor,
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

#[test]
fn the_official_count_is_forty_four() {
    // Gate 17.5 raised the official closure from 39 to 44 (Red-Black timber).
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(BlockType::ALL.len(), OFFICIAL_BLOCK_COUNT);
    assert_eq!(official_entries().len(), OFFICIAL_BLOCK_COUNT);
    // Every official block is the one sample of its type; only the two
    // optical diagnostics are extra.
    assert_eq!(catalog_entries().len(), OFFICIAL_BLOCK_COUNT + 2);
    let diagnostics: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| !e.is_official())
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        diagnostics,
        vec![SampleKind::Control, SampleKind::Reflective]
    );
}

#[test]
fn official_block_types_are_unique_and_grouped_by_family() {
    let unique: HashSet<_> = BlockType::ALL.iter().collect();
    assert_eq!(unique.len(), OFFICIAL_BLOCK_COUNT);
    // Families appear in catalog order, each as one contiguous run.
    let families: Vec<BlockFamily> = BlockType::ALL.iter().map(|t| t.family()).collect();
    assert!(families.windows(2).all(|w| w[0] <= w[1]), "{families:?}");
    let counts = |f: BlockFamily| families.iter().filter(|x| **x == f).count();
    assert_eq!(
        counts(BlockFamily::OverworldTerrain) + counts(BlockFamily::OverworldArchitecture),
        17
    );
    assert_eq!(counts(BlockFamily::Portal), 2);
    let red_black = [
        BlockFamily::RedBlackBase,
        BlockFamily::Crimson,
        BlockFamily::Orange,
        BlockFamily::Violet,
        BlockFamily::StructuralRedBlack,
    ];
    assert_eq!(red_black.iter().map(|f| counts(*f)).sum::<usize>(), 20);
    assert_eq!(counts(BlockFamily::RedBlackTimber), 5);
    for (rank, t) in BlockType::ALL.iter().enumerate() {
        assert_eq!(t.catalog_rank(), rank);
    }
}

#[test]
fn catalog_entries_are_unique_one_per_official_block() {
    let official = official_entries();
    let types: Vec<BlockType> = official.iter().map(|e| e.block_type).collect();
    assert_eq!(
        types,
        BlockType::ALL.to_vec(),
        "registry order, no gaps, no repeats"
    );
    let ids: HashSet<_> = official.iter().map(|e| e.material_id).collect();
    assert_eq!(ids.len(), OFFICIAL_BLOCK_COUNT, "no shared MaterialId");
}

#[test]
fn display_names_are_unique() {
    let names: Vec<_> = catalog_entries().iter().map(|e| e.display_name).collect();
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len());
    assert!(names.iter().all(|n| !n.is_empty()));
}

#[test]
fn every_official_material_id_resolves() {
    let e = env();
    for entry in official_entries() {
        assert!(
            e.library.contains(entry.material_id),
            "{}",
            entry.display_name
        );
        for (_, block) in &entry.extra_blocks {
            assert!(
                e.library.contains(block.material_id()),
                "{}",
                entry.display_name
            );
        }
    }
}

#[test]
fn every_official_geometry_resolves() {
    for entry in official_entries() {
        let geometry = block_geometry(entry.block_type, entry.orientation);
        let partial = PARTIAL.contains(&entry.block_type);
        assert_eq!(!geometry.is_full_cube(), partial, "{}", entry.display_name);
        if !partial {
            assert!(
                matches!(geometry, BlockGeometry::FullCube),
                "{}",
                entry.display_name
            );
        }
        for part in geometry.parts() {
            assert!(part.is_within_unit_cell(), "{}", entry.display_name);
        }
    }
}

#[test]
fn every_official_texture_resolves() {
    let e = env();
    let mut checked = 0;
    for entry in official_entries() {
        let mut ids = vec![entry.material_id];
        ids.extend(entry.extra_blocks.iter().map(|(_, b)| b.material_id()));
        for id in ids {
            let m = e.library.get(id).unwrap();
            let faces = m
                .face_textures
                .unwrap_or_else(|| panic!("{} albedo", entry.display_name));
            for face in FACES {
                let t = e
                    .manager
                    .get(faces.texture_for_face(face))
                    .expect(entry.display_name);
                assert!(t.width() > 0 && t.height() > 0);
            }
            for extra in [m.emissive_texture, m.normal_texture].into_iter().flatten() {
                assert!(e.manager.get(extra).is_some(), "{}", entry.display_name);
            }
        }
        checked += 1;
    }
    assert_eq!(checked, OFFICIAL_BLOCK_COUNT);
}

#[test]
fn every_focus_point_is_finite_and_on_its_sample() {
    for entry in official_entries() {
        let p = entry.focus_point;
        assert!(
            p.x.is_finite() && p.y.is_finite() && p.z.is_finite(),
            "{}",
            entry.display_name
        );
        let c = Vec3::new(
            entry.position.x as f32 + 0.5,
            entry.position.y as f32 + 0.5,
            entry.position.z as f32 + 0.5,
        );
        assert!((p - c).length() <= 0.5 + 1e-6, "{}", entry.display_name);
    }
}

#[test]
fn no_duplicate_entries() {
    let entries = catalog_entries();
    for (i, a) in entries.iter().enumerate() {
        for b in &entries[i + 1..] {
            assert_ne!(a, b);
            if a.is_official() && b.is_official() {
                assert_ne!(a.block_type, b.block_type);
            }
        }
    }
}

#[test]
fn no_duplicate_positions() {
    let mut cells: HashSet<IVec3> = HashSet::new();
    for entry in catalog_entries() {
        assert!(cells.insert(entry.position), "{}", entry.display_name);
        for (cell, _) in &entry.extra_blocks {
            assert!(cells.insert(*cell), "{} extra cell", entry.display_name);
        }
    }
    // Every sample really stands in the world at its cell.
    let scene = CatalogScene::new();
    for entry in catalog_entries() {
        assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    }
}

#[test]
fn selection_walks_all_official_blocks_family_by_family() {
    let mut s = CatalogScene::new();
    let mut seen = Vec::new();
    for _ in 0..s.len() {
        if s.selected().is_official() {
            seen.push(s.selected().block_type);
            assert!(s.label().contains(s.selected().block_type.family().label()));
        }
        s.select_next();
    }
    assert_eq!(s.selected_index(), 0, "a full cycle wraps to the start");
    assert_eq!(seen, BlockType::ALL.to_vec());
}

#[test]
fn reset_returns_to_the_overview_after_focusing_any_block() {
    let home = gallery_camera(4.0 / 3.0);
    let initial = DiagnosticCameraState::from_pose(home.position, home.target);
    let mut s = CatalogScene::new();
    for _ in 0..s.len() {
        let mut view = initial;
        s.focus_camera(&mut view);
        assert_eq!(view.target, s.focus_target());
        assert!((view.distance - FOCUS_DISTANCE).abs() < 1e-5);
        view.reset();
        assert_eq!(view, initial);
        s.select_next();
    }
}

#[test]
fn the_advanced_material_systems_are_still_present() {
    let e = env();
    let by_type = |t: BlockType| {
        let entry = official_entries()
            .into_iter()
            .find(|x| x.block_type == t)
            .unwrap();
        e.library.get(entry.material_id).unwrap().clone()
    };
    let glass = by_type(BlockType::Glass);
    assert!(glass.transparency > 0.5 && (glass.refractive_index - 1.5).abs() < 1e-6);
    let water = by_type(BlockType::Water);
    assert!(water.transparency > 0.5 && (water.refractive_index - 1.33).abs() < 1e-6);
    assert_eq!(by_type(BlockType::Leaves).alpha_mode, AlphaMode::Cutout);
    for t in [
        BlockType::RedstoneLampLit,
        BlockType::PortalCoreDarkCrimson,
        BlockType::CrimsonHeart,
        BlockType::CryingObsidianViolet,
        BlockType::AmethystCluster,
    ] {
        let m = by_type(t);
        assert!(
            m.emissive_texture.is_some() && m.emission_strength > 0.0,
            "{t:?}"
        );
    }
    for t in [
        BlockType::DeepslateBricks,
        BlockType::RedBlackDeepslateBricksCrimson,
        BlockType::RedBlackDeepslateBricksOrange,
        BlockType::RedBlackDeepslateBricksViolet,
    ] {
        assert!(by_type(t).normal_texture.is_some(), "{t:?}");
    }
    let amethyst = by_type(BlockType::AmethystCluster);
    assert!(amethyst.transparency > 0.0 && (amethyst.refractive_index - 1.45).abs() < 1e-6);
    // The mirror diagnostic still demonstrates reflection.
    let mirror = catalog_entries()
        .into_iter()
        .find(|x| x.kind == SampleKind::Reflective)
        .unwrap();
    assert!(e.library.get(mirror.material_id).unwrap().reflectivity > 0.3);
}

#[test]
fn red_black_variants_never_collide() {
    let e = env();
    let groups: [&[BlockType]; 4] = [
        &[
            BlockType::CryingObsidianCrimson,
            BlockType::CryingObsidianOrange,
            BlockType::CryingObsidianViolet,
        ],
        &[
            BlockType::RedBlackDeepslateBricksCrimson,
            BlockType::RedBlackDeepslateBricksOrange,
            BlockType::RedBlackDeepslateBricksViolet,
        ],
        &[BlockType::CrimsonHeart, BlockType::PurpleHeart],
        &[
            BlockType::OrangeClub,
            BlockType::PurpleClub,
            BlockType::OrangeSpade,
            BlockType::PurpleSpade,
            BlockType::CrimsonDiamond,
            BlockType::PurpleDiamond,
        ],
    ];
    let official = official_entries();
    for group in groups {
        let entries: Vec<_> = group
            .iter()
            .map(|t| official.iter().find(|x| x.block_type == *t).unwrap())
            .collect();
        let materials: HashSet<_> = entries.iter().map(|x| x.material_id).collect();
        let positions: HashSet<_> = entries.iter().map(|x| x.position).collect();
        let albedos: HashSet<_> = entries
            .iter()
            .map(|x| {
                e.library
                    .get(x.material_id)
                    .unwrap()
                    .face_textures
                    .unwrap()
                    .texture_for_face(Face::PositiveZ)
            })
            .collect();
        assert_eq!(materials.len(), group.len(), "{group:?}");
        assert_eq!(positions.len(), group.len(), "{group:?}");
        assert_eq!(albedos.len(), group.len(), "{group:?}");
    }
}
