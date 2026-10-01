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
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
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
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use scene::block_shape_factory::{block_geometry, canonical_geometry};
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::orientation::Orientation;
use scene::red_black_timber::{
    PRE_TIMBER_OFFICIAL_COUNT, RED_BLACK_TIMBER, RED_BLACK_TIMBER_DIR, TIMBER_OFFICIAL_COUNT,
    is_red_black_timber, overworld_counterpart, timber_contract,
};
use std::collections::HashSet;

const ORIENTATIONS: [Orientation; 6] = [
    Orientation::Up,
    Orientation::Down,
    Orientation::North,
    Orientation::South,
    Orientation::East,
    Orientation::West,
];

// 1
#[test]
fn the_five_new_block_types_exist() {
    assert_eq!(
        RED_BLACK_TIMBER,
        [
            BlockType::RedBlackLeaves,
            BlockType::RedBlackLog,
            BlockType::RedBlackWoodPlanks,
            BlockType::RedBlackFence,
            BlockType::RedBlackWoodDoor,
        ]
    );
    for t in RED_BLACK_TIMBER {
        assert!(is_red_black_timber(t));
        assert_eq!(t.family(), BlockFamily::RedBlackTimber);
    }
    assert_eq!(
        BlockFamily::RedBlackTimber.label(),
        "Red-Black Organic / Timber"
    );
    assert!(
        BlockFamily::RedBlackTimber > BlockFamily::StructuralRedBlack,
        "catalog order"
    );
}

// 2
#[test]
fn no_discriminant_or_registry_entry_is_duplicated() {
    // Every registered block once; the five timber blocks are distinct from
    // every pre-existing identity.
    let all: HashSet<BlockType> = BlockType::ALL
        .iter()
        .copied()
        .chain(RED_BLACK_TIMBER)
        .collect();
    assert_eq!(
        all.len(),
        PRE_TIMBER_OFFICIAL_COUNT + RED_BLACK_TIMBER.len()
    );
    let discriminants: HashSet<usize> = RED_BLACK_TIMBER.iter().map(|t| *t as usize).collect();
    assert_eq!(discriminants.len(), 5);
    let older: Vec<BlockType> = BlockType::ALL
        .iter()
        .copied()
        .filter(|t| !is_red_black_timber(*t))
        .collect();
    assert_eq!(older.len(), PRE_TIMBER_OFFICIAL_COUNT);
    for t in older {
        assert!(!discriminants.contains(&(t as usize)), "{t:?}");
    }
    let ids: Vec<u32> = RED_BLACK_TIMBER
        .iter()
        .flat_map(|t| timber_contract(*t).unwrap().material_ids)
        .map(|m| format!("{m:?}"))
        .map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, vec![64, 65, 66, 67, 68, 69]);
    let src: String = [
        "src/scene/overworld_blocks.rs",
        "src/scene/material_gallery.rs",
        "src/scene/scene.rs",
    ]
    .iter()
    .map(|p| std::fs::read_to_string(p).unwrap())
    .collect();
    for id in &ids {
        assert!(
            !src.contains(&format!("MaterialId::new({id})")),
            "material id {id} already taken"
        );
    }
}

// 3
#[test]
fn each_block_reuses_its_overworld_geometry() {
    for t in RED_BLACK_TIMBER {
        let c = timber_contract(t).unwrap();
        assert_eq!(overworld_counterpart(t), Some(c.counterpart));
        assert_eq!(
            canonical_geometry(t),
            canonical_geometry(c.counterpart),
            "{t:?}"
        );
        for o in ORIENTATIONS {
            assert_eq!(
                block_geometry(t, o),
                block_geometry(c.counterpart, o),
                "{t:?} {o:?}"
            );
        }
    }
    assert_eq!(overworld_counterpart(BlockType::Stone), None);
    assert!(timber_contract(BlockType::WoodDoor).is_none());
}

// 4
#[test]
fn texture_and_material_paths_are_deterministic() {
    assert_eq!(
        RED_BLACK_TIMBER_DIR,
        "assets/textures/red_black_maze/timber"
    );
    let expect = [
        (BlockType::RedBlackLeaves, vec!["leaves.png"]),
        (BlockType::RedBlackLog, vec!["log/end.png", "log/side.png"]),
        (BlockType::RedBlackWoodPlanks, vec!["wood_planks.png"]),
        (BlockType::RedBlackFence, vec!["wood_planks.png"]),
        (
            BlockType::RedBlackWoodDoor,
            vec![
                "wood_door/bottom.png",
                "wood_door/top.png",
                "wood_door/edge_bottom.png",
                "wood_door/edge_top.png",
            ],
        ),
    ];
    for (t, files) in expect {
        let c = timber_contract(t).unwrap();
        let wanted: Vec<String> = files
            .iter()
            .map(|f| format!("{RED_BLACK_TIMBER_DIR}/{f}"))
            .collect();
        assert_eq!(c.texture_paths, wanted, "{t:?}");
        // A different file from every Overworld texture: nothing is shared.
        for p in &c.texture_paths {
            assert!(!p.contains("assets/textures/overworld"), "{p}");
        }
        assert_eq!(
            c.material_ids.len(),
            if t == BlockType::RedBlackWoodDoor {
                2
            } else {
                1
            }
        );
    }
}

// 5
#[test]
fn the_family_is_not_emissive_by_default() {
    for t in RED_BLACK_TIMBER {
        assert!(!timber_contract(t).unwrap().emissive, "{t:?}");
    }
}

// 6
#[test]
fn only_the_door_is_marked_camera_passable() {
    for t in RED_BLACK_TIMBER {
        let c = timber_contract(t).unwrap();
        assert_eq!(c.camera_passable, t == BlockType::RedBlackWoodDoor, "{t:?}");
    }
}

// 7
#[test]
fn the_fence_remains_partial_geometry() {
    let c = timber_contract(BlockType::RedBlackFence).unwrap();
    assert!(c.partial);
    let g = canonical_geometry(BlockType::RedBlackFence);
    assert!(!g.is_full_cube());
    assert_eq!(
        g.part_count(),
        canonical_geometry(BlockType::Fence).part_count()
    );
    assert!(
        timber_contract(BlockType::RedBlackWoodDoor)
            .unwrap()
            .partial
    );
    assert!(!canonical_geometry(BlockType::RedBlackWoodDoor).is_full_cube());
}

// 8
#[test]
fn the_leaves_preserve_the_cutout_contract() {
    let c = timber_contract(BlockType::RedBlackLeaves).unwrap();
    assert!(c.cutout && !c.partial);
    assert!(canonical_geometry(BlockType::RedBlackLeaves).is_full_cube());
    // The door keeps its window holes; solid wood stays opaque.
    assert!(timber_contract(BlockType::RedBlackWoodDoor).unwrap().cutout);
    for t in [
        BlockType::RedBlackLog,
        BlockType::RedBlackWoodPlanks,
        BlockType::RedBlackFence,
    ] {
        assert!(!timber_contract(t).unwrap().cutout, "{t:?}");
    }
}

// 9
#[test]
fn the_log_keeps_end_grain_and_bark_faces() {
    let c = timber_contract(BlockType::RedBlackLog).unwrap();
    assert_eq!(c.texture_paths.len(), 2, "end grain and bark");
    assert!(c.texture_paths[0].ends_with("log/end.png"));
    assert!(c.texture_paths[1].ends_with("log/side.png"));
    assert!(canonical_geometry(BlockType::RedBlackLog).is_full_cube());
}

// 10
#[test]
fn the_catalog_closes_at_forty_four_after_the_gate() {
    assert_eq!(PRE_TIMBER_OFFICIAL_COUNT, 39);
    assert_eq!(TIMBER_OFFICIAL_COUNT, 44);
    assert!(
        OFFICIAL_BLOCK_COUNT == PRE_TIMBER_OFFICIAL_COUNT
            || OFFICIAL_BLOCK_COUNT == TIMBER_OFFICIAL_COUNT
    );
    assert_eq!(BlockType::ALL.len(), OFFICIAL_BLOCK_COUNT);
}
