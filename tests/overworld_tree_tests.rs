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
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
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

use core::math::IVec3;
use scene::block_type::BlockType;
use scene::material_gallery::leaves_material_id;
use scene::overworld::{
    HOUSE_FOOTPRINT, MAX_TRUNK_HEIGHT, MIN_TREE_SPACING, MIN_TRUNK_HEIGHT, PATH_RESERVE,
    TREE_COUNT, is_reserved_column, pond_basin_columns,
};
use scene::overworld_blocks::log_material_id;
use scene::terrain::generator::footprint_contains;
use scene::world::WorldScene;
use std::collections::HashSet;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn three_to_five_trees_are_planted() {
    let scene = WorldScene::new();
    assert!((3..=5).contains(&TREE_COUNT));
    assert!(
        (3..=5).contains(&scene.trees().len()),
        "{} trees",
        scene.trees().len()
    );
    assert_eq!(scene.trees().len(), TREE_COUNT);
}

#[test]
fn every_trunk_stands_on_grass_never_on_water_or_sand() {
    let scene = WorldScene::new();
    for tree in scene.trees() {
        assert_eq!(
            block_type(&scene, tree.base),
            Some(BlockType::Grass),
            "{:?}",
            tree.base
        );
        assert!(footprint_contains(scene.config(), tree.base.x, tree.base.z));
        // The trunk starts right above the grass, nothing sits under it.
        assert_eq!(
            tree.trunk[0],
            IVec3::new(tree.base.x, tree.base.y + 1, tree.base.z)
        );
    }
}

#[test]
fn trunks_are_logs_of_a_valid_height_with_leaves_above() {
    let scene = WorldScene::new();
    let mut heights = HashSet::new();
    for tree in scene.trees() {
        assert!((MIN_TRUNK_HEIGHT..=MAX_TRUNK_HEIGHT).contains(&tree.trunk_height));
        assert!((3..=5).contains(&tree.trunk_height));
        assert_eq!(tree.trunk.len() as i32, tree.trunk_height);
        heights.insert(tree.trunk_height);
        for (i, cell) in tree.trunk.iter().enumerate() {
            assert_eq!(
                *cell,
                IVec3::new(tree.base.x, tree.base.y + 1 + i as i32, tree.base.z)
            );
            let block = scene.world().get(*cell).unwrap();
            assert_eq!(block.block_type(), BlockType::Log);
            assert_eq!(block.material_id(), log_material_id());
        }
        assert!(tree.leaves.len() >= 30, "{} leaves", tree.leaves.len());
        let top = tree.trunk.last().unwrap().y;
        for cell in &tree.leaves {
            let block = scene.world().get(*cell).unwrap();
            assert_eq!(block.block_type(), BlockType::Leaves);
            assert_eq!(block.material_id(), leaves_material_id());
            assert!(
                cell.y >= top - 1 && cell.y <= top + 2,
                "{cell:?} vs top {top}"
            );
            assert!((cell.x - tree.base.x).abs() <= 2 && (cell.z - tree.base.z).abs() <= 2);
        }
        // Something leafy sits directly above the trunk.
        assert!(
            tree.leaves
                .contains(&IVec3::new(tree.base.x, top + 1, tree.base.z))
        );
    }
    // Not every trunk has the same height.
    assert!(heights.len() >= 2, "{heights:?}");
}

#[test]
fn canopies_are_compact_but_not_identical() {
    let scene = WorldScene::new();
    let shapes: Vec<HashSet<(i32, i32, i32)>> = scene
        .trees()
        .iter()
        .map(|t| {
            let top = t.trunk.last().unwrap().y;
            t.leaves
                .iter()
                .map(|c| (c.x - t.base.x, c.y - top, c.z - t.base.z))
                .collect()
        })
        .collect();
    for (i, a) in shapes.iter().enumerate() {
        for b in shapes.iter().skip(i + 1) {
            assert_ne!(a, b, "two identical canopies");
        }
    }
}

#[test]
fn trees_never_overlap_each_other_or_the_terrain() {
    let scene = WorldScene::new();
    let trees = scene.trees();
    let mut occupied: HashSet<IVec3> = HashSet::new();
    for tree in trees {
        for cell in tree.trunk.iter().chain(tree.leaves.iter()) {
            assert!(occupied.insert(*cell), "cell {cell:?} used twice");
        }
    }
    for (i, a) in trees.iter().enumerate() {
        for b in trees.iter().skip(i + 1) {
            let dx = (a.base.x - b.base.x).abs();
            let dz = (a.base.z - b.base.z).abs();
            assert!(
                dx >= MIN_TREE_SPACING || dz >= MIN_TREE_SPACING,
                "{:?} vs {:?}",
                a.base,
                b.base
            );
        }
    }
    // Leaves only fill air: every leaf cell is above the terrain column top
    // that existed before the tree (its neighbors' grass stays intact).
    for tree in trees {
        for cell in &tree.leaves {
            if cell.x != tree.base.x || cell.z != tree.base.z {
                let below_ground = scene.column_top(cell.x, cell.z).unwrap() >= cell.y
                    && block_type(&scene, IVec3::new(cell.x, cell.y - 1, cell.z))
                        .map(|b| {
                            matches!(
                                b,
                                BlockType::Grass
                                    | BlockType::Dirt
                                    | BlockType::Stone
                                    | BlockType::Sand
                            )
                        })
                        .unwrap_or(false)
                    && cell.y <= tree.base.y;
                assert!(!below_ground, "leaf inside the ground at {cell:?}");
            }
        }
    }
}

#[test]
fn trees_keep_out_of_the_pond_house_and_path_reservations() {
    let scene = WorldScene::new();
    let basin = pond_basin_columns();
    for tree in scene.trees() {
        let (x, z) = (tree.base.x, tree.base.z);
        assert!(!is_reserved_column(scene.pond(), x, z));
        assert!(
            !HOUSE_FOOTPRINT.grown(2).contains(x, z),
            "trunk at ({x}, {z}) too close to the house"
        );
        assert!(!PATH_RESERVE.contains(x, z));
        for &(bx, bz) in &basin {
            assert!(
                (bx - x).abs() > 2 || (bz - z).abs() > 2,
                "trunk at ({x}, {z}) by the pond"
            );
        }
        // No leaf hangs over the water or the house footprint.
        for cell in &tree.leaves {
            assert!(
                !basin.contains(&(cell.x, cell.z)),
                "leaf over the pond at {cell:?}"
            );
            assert!(
                !HOUSE_FOOTPRINT.contains(cell.x, cell.z),
                "leaf over the house at {cell:?}"
            );
        }
    }
}

#[test]
fn tree_placement_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.trees(), b.trees());
}
