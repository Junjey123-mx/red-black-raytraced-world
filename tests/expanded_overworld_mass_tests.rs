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

use core::math::IVec3;
use scene::block_type::BlockType;
use scene::expansion::{UPPER_STRATA_DEPTH, VOXEL_BUDGET_SOFT_MAX, VOXEL_BUDGET_TARGET};
use scene::rhombus::connected_component;
use scene::world::WorldScene;

#[test]
fn the_expanded_mass_is_one_connected_piece() {
    let scene = WorldScene::new();
    let main = connected_component(scene.world(), scene.silhouette().tip);
    assert_eq!(main.len(), scene.world().len());
    // Every seam column touches an island cell face to face.
    let e = scene.overworld_expansion();
    let seams = e.columns.iter().filter(|c| c.seam).count();
    assert!(seams >= 10, "{seams} seam columns");
    for c in e.columns.iter().filter(|c| c.seam) {
        let touches = (c.surface_y - UPPER_STRATA_DEPTH..=c.surface_y + 1).any(|y| {
            scene.world().contains(IVec3::new(c.x - 1, y, c.z))
                && !e.cells.contains(&IVec3::new(c.x - 1, y, c.z))
        });
        assert!(
            touches,
            "seam column ({}, {}) hangs free of the island",
            c.x, c.z
        );
    }
}

#[test]
fn the_castle_pad_is_fully_supported() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let e = scene.overworld_expansion();
    let pad = l.overworld_castle_pad;
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let c = e
                .column(x, z)
                .unwrap_or_else(|| panic!("pad column ({x}, {z}) missing"));
            assert_eq!(c.surface_y, l.castle_pad_surface_y);
            for d in 0..(UPPER_STRATA_DEPTH - 1) {
                assert!(scene.world().contains(IVec3::new(x, c.surface_y - d, z)));
            }
        }
    }
    // Gate 16 builds the castle on the pad: its ground row is the foundation.
    pad_is_castle_ground(&scene);
}

#[test]
fn the_strata_are_present_in_order() {
    let scene = WorldScene::new();
    let e = scene.overworld_expansion();
    let mut deepslate = 0;
    // The relief pass (C178) exposes stone on some flanks and cuts the
    // grass of a few columns; those cells are checked by its own tests.
    let relief = scene.overworld_relief();
    // Terraced, cut and raised columns have their profile shifted by one
    // (checked by the relief tests); the others keep the full sequence.
    let approach = scene.overworld_approach();
    let touched = |c: &scene::expansion::ExpansionColumn| {
        relief
            .dirt_cuts
            .iter()
            .chain(relief.terraces.iter())
            .chain(relief.rises.iter())
            .chain(approach.cells().iter())
            .chain(approach.accents.iter())
            .any(|d| d.x == c.x && d.z == c.z)
            || scene.castle_layout().footprint.contains(c.x, c.z)
    };
    for c in &e.columns {
        if touched(c) {
            continue;
        }
        let t = |d: i32| {
            scene
                .world()
                .get(IVec3::new(c.x, c.surface_y - d, c.z))
                .map(|b| b.block_type())
        };
        assert_eq!(t(0), Some(BlockType::Grass), "({}, {})", c.x, c.z);
        for d in 1..=2 {
            let cell = IVec3::new(c.x, c.surface_y - d, c.z);
            if !relief.stone_exposures.contains(&cell) {
                assert_eq!(t(d), Some(BlockType::Dirt), "{cell:?}");
            }
        }
        if c.ring {
            assert_eq!(t(3), Some(BlockType::Stone));
            assert_eq!(t(4), Some(BlockType::Deepslate));
            deepslate += 1;
        }
    }
    assert!(deepslate >= 12, "{deepslate} deepslate ring cells");
}

#[test]
fn the_top_of_every_new_column_is_grass_facing_up() {
    let scene = WorldScene::new();
    let cuts = &scene.overworld_relief().dirt_cuts;
    // Columns paved or marked by the castle approach (C179) are covered by
    // its own tests.
    let approach = scene.overworld_approach();
    let scenery = scene.expansion_scenery().upper.cells();
    let paved = |x: i32, z: i32| {
        approach
            .cells()
            .iter()
            .chain(approach.accents.iter())
            .chain(scenery.iter())
            .any(|c| c.x == x && c.z == z)
            || scene.castle_layout().footprint.contains(x, z)
    };
    for c in &scene.overworld_expansion().columns {
        if paved(c.x, c.z) {
            continue;
        }
        let top = scene.column_top(c.x, c.z).unwrap();
        assert_eq!(top, c.surface_y, "({}, {})", c.x, c.z);
        let b = scene.world().get(IVec3::new(c.x, top, c.z)).unwrap();
        let cut = cuts.iter().any(|d| d.x == c.x && d.z == c.z);
        assert_eq!(
            b.block_type(),
            if cut {
                BlockType::Dirt
            } else {
                BlockType::Grass
            }
        );
        assert_eq!(b.orientation(), scene::orientation::Orientation::Up);
    }
}

#[test]
fn the_new_terrain_has_relief() {
    let scene = WorldScene::new();
    let heights: std::collections::BTreeSet<i32> = scene
        .overworld_expansion()
        .columns
        .iter()
        .map(|c| c.surface_y)
        .collect();
    assert!(heights.len() >= 2, "{heights:?}");
    let l = scene.expansion_layout();
    // The pad is level; the ring around it is not all at the same height.
    let ring: std::collections::BTreeSet<i32> = scene
        .overworld_expansion()
        .columns
        .iter()
        .filter(|c| !l.overworld_castle_pad.contains(c.x, c.z))
        .map(|c| c.surface_y)
        .collect();
    assert!(ring.len() >= 2, "{ring:?}");
}

#[test]
fn no_floating_slab_and_a_ragged_outline() {
    let scene = WorldScene::new();
    let e = scene.overworld_expansion();
    let l = scene.expansion_layout();
    // Every cell has a neighbour in the world (no isolated cell), and the
    // east edge is not one straight line.
    for cell in &e.cells {
        let n = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ]
        .iter()
        .filter(|(dx, dy, dz)| {
            scene
                .world()
                .contains(IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz))
        })
        .count();
        assert!(n >= 1, "{cell:?} floats");
    }
    let last_x: std::collections::BTreeSet<i32> = (l.overworld_extension.min_z
        ..=l.overworld_extension.max_z)
        .filter_map(|z| e.columns.iter().filter(|c| c.z == z).map(|c| c.x).max())
        .collect();
    assert!(last_x.len() >= 2, "straight east edge: {last_x:?}");
    // No rectangular platform: the corner rows are shorter than the middle.
    let east_end = |z: i32| {
        e.columns
            .iter()
            .filter(|c| c.z == z)
            .map(|c| c.x)
            .max()
            .unwrap()
    };
    assert!(east_end(l.overworld_extension.min_z) < east_end(l.overworld_extension.min_z + 4));
    assert!(east_end(l.overworld_extension.max_z) < east_end(l.overworld_extension.max_z - 4));
}

#[test]
fn the_original_landmarks_are_preserved() {
    let scene = WorldScene::new();
    let e = scene.overworld_expansion();
    let ext = scene.expansion_layout().overworld_extension;
    // Nothing new outside the lobe rows; nothing new touched the house,
    // the pond, the trees, the portal, the upper stairs or the path.
    for cell in &e.cells {
        assert!(cell.z >= ext.min_z && cell.z <= ext.max_z);
        assert!(cell.x >= 19);
    }
    let protected: Vec<IVec3> = scene
        .house()
        .walls
        .iter()
        .chain(scene.house().floor.iter())
        .chain(scene.pond().water.iter())
        .chain(
            scene
                .trees()
                .iter()
                .flat_map(|t| t.trunk.iter().chain(t.leaves.iter())),
        )
        .chain(scene.portal().frame.iter())
        .chain(scene.portal().core.iter())
        .chain(scene.upper_descent().treads.iter().map(|(c, _)| c))
        .chain(scene.path().cells.iter())
        .copied()
        .collect();
    for cell in &protected {
        assert!(scene.world().contains(*cell), "{cell:?} lost");
        assert!(!e.cells.contains(cell));
    }
    assert_eq!(scene.trees().len(), 4);
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(
        (scene.portal().frame.len(), scene.portal().core.len()),
        (18, 12)
    );
    assert_eq!(scene.upper_descent().treads.len(), 12);
}

#[test]
fn the_voxel_budget_is_respected() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    println!(
        "GATE15 voxels after the upper mass: {n} (+{} cells)",
        scene.overworld_expansion().cells.len()
    );
    assert!(n > 6783);
    // The structural masses meet the target; the restrained scenery (C183)
    // may add a few dozen cells on top, always under the soft maximum.
    let scenery = scene.expansion_scenery();
    let decoration = scenery.upper.cells().len() + scenery.lower.added_cells().len();
    // Gate 16's castle has its own budget on top of the Gate 15 masses.
    let castle = scene.castle_voxels();
    assert!(
        n - decoration - castle <= VOXEL_BUDGET_TARGET,
        "{n} - {decoration} - {castle} over the target"
    );
    assert!(
        n - castle <= VOXEL_BUDGET_SOFT_MAX,
        "{n} over the soft maximum"
    );
}

/// Gate 16 built the castle on the pad: its ground row is the castle
/// foundation, level with the pad surface.
fn pad_is_castle_ground(scene: &WorldScene) {
    let l = scene.expansion_layout();
    let pad = l.overworld_castle_pad;
    assert_eq!(scene.castle_layout().footprint, pad);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let ground = IVec3::new(x, l.castle_pad_surface_y, z);
            let t = scene.world().get(ground).map(|b| b.block_type());
            assert!(
                matches!(
                    t,
                    Some(
                        BlockType::DeepslateBricks
                            | BlockType::Cobblestone
                            | BlockType::Stone
                            | BlockType::WoodPlanks
                    )
                ),
                "({x},{z}) is {t:?}"
            );
            assert!(scene.world().contains(IVec3::new(x, ground.y - 1, z)));
        }
    }
}
