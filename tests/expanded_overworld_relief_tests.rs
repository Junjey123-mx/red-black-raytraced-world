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

use camera::world_collision::{
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::expansion::approach_columns;
use scene::world::WorldScene;
use std::collections::BTreeSet;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Columns paved or marked by the castle approach (C179), covered by its
/// own tests.
fn paved(scene: &WorldScene, x: i32, z: i32) -> bool {
    let a = scene.overworld_approach();
    a.cells()
        .iter()
        .chain(a.accents.iter())
        .chain(scene.expansion_scenery().upper.cells().iter())
        .any(|c| c.x == x && c.z == z)
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

#[test]
fn the_expanded_surface_is_not_flat() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let outside_pad: Vec<i32> = scene
        .overworld_expansion()
        .columns
        .iter()
        .filter(|c| !l.overworld_castle_pad.contains(c.x, c.z))
        .map(|c| c.surface_y)
        .collect();
    let heights: BTreeSet<i32> = outside_pad.iter().copied().collect();
    assert!(heights.len() >= 3, "{heights:?}");
    let r = scene.overworld_relief();
    println!(
        "GATE15 relief: terraces={} dirt_cuts={} stone_exposures={} rises={} voxels={}",
        r.terraces.len(),
        r.dirt_cuts.len(),
        r.stone_exposures.len(),
        r.rises.len(),
        scene.world().len()
    );
    assert!(
        !r.terraces.is_empty()
            && !r.dirt_cuts.is_empty()
            && !r.stone_exposures.is_empty()
            && !r.rises.is_empty()
    );
}

#[test]
fn there_are_several_height_levels_including_terraces() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let s = l.castle_pad_surface_y;
    let heights: BTreeSet<i32> = scene
        .overworld_expansion()
        .columns
        .iter()
        .map(|c| c.surface_y)
        .collect();
    assert!(
        heights.contains(&(s - 1)) && heights.contains(&s) && heights.contains(&(s + 1)),
        "{heights:?}"
    );
    // The ring steps down: every terrace cell is the grass top of its column.
    for cell in &scene.overworld_relief().terraces {
        if paved(&scene, cell.x, cell.z) {
            continue;
        }
        assert_eq!(scene.column_top(cell.x, cell.z), Some(cell.y));
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::Grass)
        );
    }
}

#[test]
fn stone_is_exposed_on_the_flanks() {
    let scene = WorldScene::new();
    let r = scene.overworld_relief();
    assert!(r.stone_exposures.len() >= 5, "{}", r.stone_exposures.len());
    for cell in &r.stone_exposures {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::Stone)
        );
        let c = scene.overworld_expansion().column(cell.x, cell.z).unwrap();
        assert!(c.ring, "exposure on an interior column {cell:?}");
        // On the flank: a face of the cell is open to the air.
        let open = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
            !scene
                .world()
                .contains(IVec3::new(cell.x + dx, cell.y, cell.z + dz))
        });
        assert!(open, "{cell:?} is not visible");
    }
}

#[test]
fn dirt_is_exposed_by_the_cuts() {
    let scene = WorldScene::new();
    let r = scene.overworld_relief();
    assert!(r.dirt_cuts.len() >= 3, "{}", r.dirt_cuts.len());
    for cell in &r.dirt_cuts {
        if paved(&scene, cell.x, cell.z) {
            continue;
        }
        assert_eq!(scene.column_top(cell.x, cell.z), Some(cell.y));
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::Dirt)
        );
        assert!(
            !scene
                .expansion_layout()
                .overworld_castle_pad
                .contains(cell.x, cell.z)
        );
    }
}

#[test]
fn grass_tops_still_face_up() {
    let scene = WorldScene::new();
    let cuts: BTreeSet<(i32, i32)> = scene
        .overworld_relief()
        .dirt_cuts
        .iter()
        .map(|c| (c.x, c.z))
        .collect();
    for c in &scene.overworld_expansion().columns {
        if paved(&scene, c.x, c.z) {
            continue;
        }
        let top = scene.column_top(c.x, c.z).unwrap();
        assert_eq!(top, c.surface_y, "({}, {})", c.x, c.z);
        let b = scene.world().get(IVec3::new(c.x, top, c.z)).unwrap();
        assert_eq!(b.orientation(), scene::orientation::Orientation::Up);
        if cuts.contains(&(c.x, c.z)) {
            assert_eq!(b.block_type(), BlockType::Dirt);
        } else {
            assert_eq!(b.block_type(), BlockType::Grass);
        }
    }
    let l = scene.expansion_layout();
    for c in scene
        .overworld_expansion()
        .columns
        .iter()
        .filter(|c| l.overworld_castle_pad.contains(c.x, c.z))
    {
        assert_eq!(c.surface_y, l.castle_pad_surface_y);
    }
}

#[test]
fn a_navigable_corridor_leads_to_the_pad() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let corridor = approach_columns(l);
    assert!(corridor.len() >= 20);
    // Untouched by the relief and walkable: the eye 1.5 above every top is
    // clear, and the tops differ by at most one between neighbours.
    let r = scene.overworld_relief();
    for (x, z) in &corridor {
        for cell in r
            .terraces
            .iter()
            .chain(r.dirt_cuts.iter())
            .chain(r.rises.iter())
        {
            assert!(
                !(cell.x == *x && cell.z == *z),
                "relief on the corridor at ({x}, {z})"
            );
        }
        let top = scene.column_top(*x, *z).unwrap();
        assert!(
            clear(
                &scene,
                v(*x as f32 + 0.5, top as f32 + 1.5 + 0.5, *z as f32 + 0.5)
            ),
            "corridor blocked at ({x}, {z})"
        );
    }
    let last = corridor.last().unwrap();
    assert!(
        l.overworld_castle_pad.contains(last.0 + 1, last.1)
            || l.overworld_castle_pad.contains(last.0, last.1 + 1)
            || l.overworld_castle_pad.contains(last.0 + 1, last.1 + 1)
            || (last.0 + 1 == l.overworld_castle_pad.min_x)
    );
}

#[test]
fn no_column_stands_two_above_all_its_neighbours() {
    let scene = WorldScene::new();
    let cols = &scene.overworld_expansion().columns;
    let height = |x: i32, z: i32| {
        cols.iter()
            .find(|c| c.x == x && c.z == z)
            .map(|c| c.surface_y)
            .or_else(|| scene.column_top(x, z))
    };
    for c in cols {
        let n: Vec<i32> = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter_map(|(dx, dz)| height(c.x + dx, c.z + dz))
            .collect();
        assert!(!n.is_empty());
        assert!(
            !n.iter().all(|h| c.surface_y >= h + 2),
            "spike at ({}, {}): {} over {n:?}",
            c.x,
            c.z,
            c.surface_y
        );
        // No pit either: never two below every neighbour.
        assert!(
            !n.iter().all(|h| c.surface_y + 2 <= *h),
            "pit at ({}, {})",
            c.x,
            c.z
        );
    }
}

#[test]
fn the_surface_is_collision_safe() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    // Every top has two open cells above it (head room for the camera);
    // the approach's markers (C179) stand on their own columns.
    for c in &scene.overworld_expansion().columns {
        if paved(&scene, c.x, c.z) {
            continue;
        }
        for dy in 1..=2 {
            assert!(
                !scene
                    .world()
                    .contains(IVec3::new(c.x, c.surface_y + dy, c.z)),
                "({}, {}) covered",
                c.x,
                c.z
            );
        }
    }
    // A walk across the pad from its west edge to its east edge, one cell
    // above the grass, is never blocked.
    let y = l.castle_pad_surface_y as f32 + 2.0;
    let z = (l.overworld_castle_pad.min_z + l.overworld_castle_pad.max_z) as f32 / 2.0 + 0.5;
    let from = v(l.overworld_castle_pad.min_x as f32 + 0.5, y, z);
    let to = v(l.overworld_castle_pad.max_x as f32 + 0.5, y, z);
    let reached = resolve_camera_motion(
        scene.world(),
        from,
        to - from,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    );
    assert!((reached - to).length() < 1e-3, "{reached:?}");
}
