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
use scene::overworld::{build_house, carve_pond, furnish_house, lay_path, plant_trees};
use scene::rhombus::connected_component;
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;

fn area(scene: &WorldScene, y: i32) -> usize {
    (-6..30)
        .flat_map(|z| (-6..30).map(move |x| (x, z)))
        .filter(|&(x, z)| scene.world().contains(IVec3::new(x, y, z)))
        .count()
}

#[test]
fn the_upper_and_lower_halves_form_one_connected_mass() {
    let scene = WorldScene::new();
    let s = scene.silhouette();
    let main = connected_component(scene.world(), s.tip);
    assert_eq!(
        main.len(),
        scene.world().len(),
        "cells outside the main component"
    );
    assert_eq!(s.main_component, scene.world().len());
    // From the grass under a tree to the tip: one piece.
    let grass = scene.trees()[0].base;
    assert!(main.contains(&grass));
    assert!(main.contains(&s.tip));
    for cell in scene
        .portal()
        .core
        .iter()
        .chain(scene.portal().frame.iter())
    {
        assert!(main.contains(cell));
    }
    for (cell, _) in scene
        .inverted_route()
        .treads
        .iter()
        .chain(scene.upper_descent().treads.iter())
    {
        assert!(main.contains(cell));
    }
}

#[test]
fn the_lower_tip_exists_and_nothing_hangs_below_it() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let tip = scene.silhouette().tip;
    assert_eq!(tip, IVec3::new(r.center_x, r.lower_tip_y, r.center_z));
    assert!(scene.world().contains(tip));
    // The tip is the lowest surface cell of the inverted world: a dark
    // ground block (or deepslate when nothing else claimed it).
    let tip_block = scene.world().get(tip).unwrap();
    assert!(!matches!(
        tip_block.block_type(),
        BlockType::Grass | BlockType::Water | BlockType::Leaves
    ));
    for z in -6..30 {
        for x in -6..30 {
            for y in (r.lower_tip_y - 6)..r.lower_tip_y {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x}, {y}, {z}) below the tip"
                );
            }
        }
    }
    assert!((1..=4).contains(&area(&scene, r.lower_tip_y)));
}

#[test]
fn the_cutaway_remains_open_and_the_portal_stays_visible() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let p = r.portal;
    // Upper cutaway: air above the shelf in the quadrant (except the
    // descent's exit stair and its footing).
    let descent: Vec<IVec3> = scene
        .upper_descent()
        .treads
        .iter()
        .map(|(c, _)| *c)
        .chain(scene.upper_descent().footings.iter().copied())
        .collect();
    let mut open = 0;
    for z in r.cutaway.min_z..24 {
        for x in r.cutaway.min_x..24 {
            for y in (r.shelf_y + 1)..=r.upper_surface_reference {
                let cell = IVec3::new(x, y, z);
                if !descent.contains(&cell) {
                    assert!(!scene.world().contains(cell), "{cell:?} fills the cutaway");
                    open += 1;
                }
            }
            for y in r.lower_tip_y..r.shelf_y {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x}, {y}, {z}) fills the lower cutaway"
                );
            }
        }
    }
    assert!(open > 500);
    for x in (p.min_x + 1)..p.max_x {
        for y in (p.min_y + 1)..p.max_y {
            for z in (p.wall_z + 1)..=(p.wall_z + 3) {
                assert!(!scene.world().contains(IVec3::new(x, y, z)));
            }
        }
    }
    assert!(
        scene
            .silhouette()
            .filled_voids
            .iter()
            .all(|c| !r.cutaway.contains_column(c.x, c.z))
    );
}

#[test]
fn no_floating_masses_and_no_enclosed_voids_remain() {
    let scene = WorldScene::new();
    let s = scene.silhouette();
    assert!(
        s.removed_floating.len() <= 12,
        "{} floating cells removed",
        s.removed_floating.len()
    );
    assert!(
        s.filled_voids.len() <= 40,
        "{} voids filled",
        s.filled_voids.len()
    );
    for cell in &s.removed_floating {
        assert!(!scene.world().contains(*cell));
    }
    for cell in &s.filled_voids {
        assert!(scene.world().contains(*cell));
    }
    // Any remaining empty cell has at least one empty neighbor (no void).
    let r = scene.rhombus();
    for z in 0..24 {
        for x in 0..24 {
            for y in r.lower_tip_y..=r.upper_surface_reference {
                let c = IVec3::new(x, y, z);
                if scene.world().contains(c) {
                    continue;
                }
                let enclosed = [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ]
                .iter()
                .all(|(dx, dy, dz)| scene.world().contains(IVec3::new(x + dx, y + dy, z + dz)));
                assert!(!enclosed, "void at {c:?}");
            }
        }
    }
}

#[test]
fn the_silhouette_narrows_at_the_waist_and_at_the_tip() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let surface = area(&scene, r.upper_taper_top);
    let waist = area(&scene, r.waist_y);
    let widest = area(&scene, r.lower_widest_y);
    let tip = area(&scene, r.lower_tip_y);
    assert!(waist < surface, "waist {waist} vs surface {surface}");
    assert!(waist < widest, "waist {waist} vs widest {widest}");
    assert!(tip < widest / 10, "tip {tip} vs widest {widest}");
    assert!(area(&scene, r.lower_tip_y + 2) > tip);
    // Width through the center along x: wide, narrow, wide, point.
    let width = |y: i32| {
        (-6..30)
            .filter(|&x| scene.world().contains(IVec3::new(x, y, r.center_z)))
            .count()
    };
    assert!(width(r.waist_y) < width(r.upper_taper_top));
    assert!(width(r.waist_y) < width(r.lower_widest_y));
    assert!(width(r.lower_tip_y) <= 4);
}

#[test]
fn the_overworld_surface_is_unchanged_outside_the_cut_and_the_pit() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let config = TerrainConfig::official();
    let mut reference = VoxelWorld::new();
    generate_terrain(&config, &mut reference);
    let pond = carve_pond(&config, &mut reference);
    build_house(&config, &mut reference);
    furnish_house(&config, &mut reference);
    let trees = plant_trees(&config, &mut reference, &pond);
    lay_path(&config, &mut reference, &pond, &trees);
    let dug: Vec<IVec3> = scene.upper_descent().dug.clone();
    // Gate 15 grows the east lobe: its cells are new by design.
    let approach = scene.overworld_approach();
    let scenery = scene.expansion_scenery().upper.cells();
    let grown = |cell: &IVec3| {
        scene.overworld_expansion().cells.contains(cell)
            || approach.cells().contains(cell)
            || approach.accents.contains(cell)
            || scenery.contains(cell)
    };
    for z in 0..24 {
        for x in 0..24 {
            if r.cutaway.contains_column(x, z) {
                continue;
            }
            for y in 0..=24 {
                let cell = IVec3::new(x, y, z);
                if dug.contains(&cell)
                    || scene.upper_descent().treads.iter().any(|(c, _)| *c == cell)
                    || grown(&cell)
                {
                    continue;
                }
                assert_eq!(
                    scene.world().get(cell),
                    reference.get(cell),
                    "{cell:?} changed"
                );
            }
        }
    }
}
