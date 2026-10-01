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
use core::math::Vec3;
use scene::block_shape_factory::block_geometry;
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{
    CatalogScene, CatalogTextures, FOCUS_DISTANCE, RED_BLACK_TIMBER_ROW_Z, SampleKind,
    catalog_entries, catalog_materials, official_entries,
};
use scene::material_gallery::gallery_camera;
use scene::red_black_timber::{PRE_TIMBER_OFFICIAL_COUNT, RED_BLACK_TIMBER, timber_contract};
use scene::texture_manager::TextureManager;
use std::collections::HashSet;

/// The Gate 17 catalog (39 identities), in registry order.
const OLD_39: [BlockType; 39] = [
    BlockType::Grass,
    BlockType::Dirt,
    BlockType::Stone,
    BlockType::Cobblestone,
    BlockType::Sand,
    BlockType::Water,
    BlockType::Deepslate,
    BlockType::Log,
    BlockType::Leaves,
    BlockType::DeepslateBricks,
    BlockType::WoodPlanks,
    BlockType::DoubleWoodSlab,
    BlockType::WoodStairs,
    BlockType::Fence,
    BlockType::Glass,
    BlockType::RedstoneLampLit,
    BlockType::WoodDoor,
    BlockType::PortalFrameRedObsidian,
    BlockType::PortalCoreDarkCrimson,
    BlockType::Mycelium,
    BlockType::SmoothBasalt,
    BlockType::BuddingAmethyst,
    BlockType::AmethystCluster,
    BlockType::CrimsonHeart,
    BlockType::CrimsonDiamond,
    BlockType::CryingObsidianCrimson,
    BlockType::NetherWartBlock,
    BlockType::OrangeClub,
    BlockType::OrangeSpade,
    BlockType::CryingObsidianOrange,
    BlockType::PurpleHeart,
    BlockType::PurpleDiamond,
    BlockType::PurpleClub,
    BlockType::PurpleSpade,
    BlockType::CryingObsidianViolet,
    BlockType::RedBlackDeepslateBricksCrimson,
    BlockType::RedBlackDeepslateBricksOrange,
    BlockType::RedBlackDeepslateBricksViolet,
    BlockType::PolishedBlackstoneBricks,
];

// 1
#[test]
fn there_are_forty_four_unique_official_block_types() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(BlockType::ALL.len(), 44);
    let unique: HashSet<BlockType> = BlockType::ALL.iter().copied().collect();
    assert_eq!(unique.len(), 44);
    assert_eq!(official_entries().len(), 44);
    for t in RED_BLACK_TIMBER {
        assert!(BlockType::ALL.contains(&t), "{t:?}");
    }
}

// 2
#[test]
fn no_catalog_entry_is_duplicated() {
    let official = official_entries();
    let types: Vec<BlockType> = official.iter().map(|e| e.block_type).collect();
    assert_eq!(types, BlockType::ALL.to_vec());
    let names: HashSet<&str> = catalog_entries().iter().map(|e| e.display_name).collect();
    assert_eq!(names.len(), catalog_entries().len());
    let ids: HashSet<_> = official.iter().map(|e| e.material_id).collect();
    assert_eq!(ids.len(), 44, "one material per official block");
    let cells: HashSet<_> = catalog_entries()
        .iter()
        .flat_map(|e| std::iter::once(e.position).chain(e.extra_blocks.iter().map(|(c, _)| *c)))
        .collect();
    assert_eq!(
        cells.len(),
        catalog_entries()
            .iter()
            .map(|e| 1 + e.extra_blocks.len())
            .sum::<usize>()
    );
}

// 3
#[test]
fn the_five_new_variants_are_selectable() {
    let mut s = CatalogScene::new();
    let mut seen = Vec::new();
    for _ in 0..s.len() {
        if RED_BLACK_TIMBER.contains(&s.selected().block_type) && s.selected().is_official() {
            seen.push(s.selected().block_type);
            assert!(
                s.label().contains("Red-Black Organic / Timber"),
                "{}",
                s.label()
            );
            assert!(
                s.label().ends_with(&format!("/ {}", s.len())),
                "{}",
                s.label()
            );
        }
        s.select_next();
    }
    assert_eq!(seen, RED_BLACK_TIMBER.to_vec());
    // Backwards too: the last official entry before the diagnostics is the door.
    let mut s = CatalogScene::new();
    s.select_previous();
    s.select_previous();
    s.select_previous();
    assert_eq!(s.selected().block_type, BlockType::RedBlackWoodDoor);
}

// 4
#[test]
fn focus_works_on_every_new_variant() {
    let home = gallery_camera(4.0 / 3.0);
    let initial = DiagnosticCameraState::from_pose(home.position, home.target);
    let mut s = CatalogScene::new();
    let mut focused = 0;
    for _ in 0..s.len() {
        if RED_BLACK_TIMBER.contains(&s.selected().block_type) {
            let mut view = initial;
            s.focus_camera(&mut view);
            assert_eq!(view.target, s.focus_target());
            assert!((view.distance - FOCUS_DISTANCE).abs() < 1e-5);
            let e = s.selected();
            let c = Vec3::new(
                e.position.x as f32 + 0.5,
                e.position.y as f32 + 0.5,
                e.position.z as f32 + 0.5,
            );
            assert!((e.focus_point - c).length() <= 1.0, "{}", e.display_name);
            view.reset();
            assert_eq!(view, initial, "reset after focusing {}", e.display_name);
            focused += 1;
        }
        s.select_next();
    }
    assert_eq!(focused, 5);
}

// 5
#[test]
fn the_labels_are_correct() {
    let names: Vec<(&str, BlockType)> = official_entries()
        .iter()
        .filter(|e| RED_BLACK_TIMBER.contains(&e.block_type))
        .map(|e| (e.display_name, e.block_type))
        .collect();
    assert_eq!(
        names,
        vec![
            ("Red-Black leaves", BlockType::RedBlackLeaves),
            ("Red-Black log", BlockType::RedBlackLog),
            ("Red-Black wood planks", BlockType::RedBlackWoodPlanks),
            ("Red-Black fence", BlockType::RedBlackFence),
            ("Red-Black wood door", BlockType::RedBlackWoodDoor),
        ]
    );
}

// 6
#[test]
fn the_category_and_sample_kinds_are_correct() {
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
    for e in official_entries()
        .iter()
        .filter(|e| RED_BLACK_TIMBER.contains(&e.block_type))
    {
        assert_eq!(e.block_type.family(), BlockFamily::RedBlackTimber);
        assert_eq!(e.position.z, RED_BLACK_TIMBER_ROW_Z, "{}", e.display_name);
        let contract = timber_contract(e.block_type).unwrap();
        assert_eq!(e.material_id, contract.material_ids[0]);
        for (_, b) in &e.extra_blocks {
            assert!(contract.material_ids.contains(&b.material_id()));
        }
        let expected = match e.block_type {
            BlockType::RedBlackLeaves => SampleKind::Cutout,
            BlockType::RedBlackFence | BlockType::RedBlackWoodDoor => SampleKind::PartialGeometry,
            _ => SampleKind::PlainBlock,
        };
        assert_eq!(e.kind, expected, "{}", e.display_name);
        let m = library.get(e.material_id).expect("resolved material");
        assert!(m.emissive_texture.is_none(), "{}", e.display_name);
        let faces = m.face_textures.expect("albedo");
        assert!(
            manager
                .get(faces.texture_for_face(Face::PositiveY))
                .is_some()
        );
        assert_eq!(
            m.alpha_mode == AlphaMode::Cutout,
            contract.cutout,
            "{}",
            e.display_name
        );
        assert_eq!(
            !block_geometry(e.block_type, e.orientation).is_full_cube(),
            contract.partial,
            "{}",
            e.display_name
        );
    }
    let door = official_entries()
        .into_iter()
        .find(|e| e.block_type == BlockType::RedBlackWoodDoor)
        .unwrap();
    assert_eq!(door.extra_blocks.len(), 1, "the door's upper half");
}

// 7
#[test]
fn the_old_thirty_nine_are_unchanged() {
    assert_eq!(PRE_TIMBER_OFFICIAL_COUNT, 39);
    assert_eq!(
        &BlockType::ALL[..39],
        &OLD_39[..],
        "the first 39 identities keep their order"
    );
    let names: Vec<&str> = official_entries()
        .iter()
        .take(39)
        .map(|e| e.display_name)
        .collect();
    for n in [
        "Grass",
        "Leaves",
        "WoodDoor",
        "Portal core",
        "Crimson heart",
        "Polished blackstone bricks",
    ] {
        assert!(names.contains(&n), "{n}");
    }
    for (rank, t) in OLD_39.iter().enumerate() {
        assert_eq!(t.catalog_rank(), rank);
    }
    // The Gate 17 rows are where they were.
    let leaves = official_entries()
        .into_iter()
        .find(|e| e.block_type == BlockType::Leaves)
        .unwrap();
    assert_eq!(leaves.position.z, scene::material_gallery::BACK_Z);
}

// 8
#[test]
fn diagnostics_are_not_counted_as_official() {
    let all = catalog_entries();
    assert_eq!(all.len(), 46);
    let diagnostics: Vec<SampleKind> = all
        .iter()
        .filter(|e| !e.is_official())
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        diagnostics,
        vec![SampleKind::Control, SampleKind::Reflective]
    );
    assert_eq!(
        all.iter().filter(|e| e.is_official()).count(),
        OFFICIAL_BLOCK_COUNT
    );
    assert!(
        all[..44].iter().all(|e| e.is_official()),
        "diagnostics close the list"
    );
}
