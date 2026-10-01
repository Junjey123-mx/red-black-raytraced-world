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

use camera::world_collision::{
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::expansion::{VOXEL_BUDGET_SOFT_MAX, VOXEL_BUDGET_TARGET, lower_bottom_y};
use scene::orientation::Orientation;
use scene::rhombus::connected_component;
use scene::world::WorldScene;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
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
fn the_lower_mass_is_one_connected_piece() {
    let scene = WorldScene::new();
    let main = connected_component(scene.world(), scene.silhouette().tip);
    assert_eq!(main.len(), scene.world().len());
    let e = scene.red_black_expansion();
    assert!(!e.underside.is_empty() && !e.walls.is_empty());
    // The underside continues the rim of the existing lower mass: the
    // westernmost lower columns sit at the rim level (or one further, where
    // the C181 relief pushed them) next to island cells.
    let l = scene.expansion_layout();
    let rim = lower_bottom_y(l, l.fortress_terrace_min_x - 1);
    let mut touching = 0;
    for c in e.columns.iter().filter(|c| c.x < l.fortress_terrace_min_x) {
        assert!(c.bottom_y == rim || c.bottom_y == rim - 1, "{c:?}");
        let west = IVec3::new(c.x - 1, rim, c.z);
        if scene.world().contains(west) && !e.cells().contains(&west) {
            touching += 1;
        }
    }
    assert!(touching >= 4, "{touching} rim columns touch the old mass");
    println!(
        "GATE15 lower: columns={} underside={} walls={} voxels={}",
        e.columns.len(),
        e.underside.len(),
        e.walls.len(),
        scene.world().len()
    );
}

#[test]
fn the_fortress_pad_is_supported_and_open_below() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let c = scene
                .red_black_expansion()
                .column(x, z)
                .unwrap_or_else(|| panic!("pad column ({x}, {z}) missing"));
            assert_eq!(c.bottom_y, l.fortress_pad_bottom_y);
            assert!(scene.world().contains(IVec3::new(x, c.bottom_y, z)));
        }
    }
    // Gate 17 built the fortress on it: the pad row is its foundation.
    pad_is_fortress_ground(&scene);
}

#[test]
fn the_portal_route_is_untouched() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    for (cell, o) in &route.treads {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::WoodStairs)
        );
        assert_eq!(*o, Orientation::Down);
    }
    // Dug cells stay open, except those the route then filled with a tread.
    for cell in &route.dug {
        if route.treads.iter().any(|(t, _)| t == cell) {
            continue;
        }
        assert!(!scene.world().contains(*cell), "{cell:?} refilled");
    }
    for cell in &route.supports {
        assert!(scene.world().contains(*cell));
    }
    assert_eq!(
        (scene.portal().frame.len(), scene.portal().core.len()),
        (18, 12)
    );
    let e = scene.red_black_expansion();
    for cell in e.cells() {
        assert!(cell.x >= 17, "{cell:?} reaches into the portal tunnel area");
        assert!(!scene.portal().frame.contains(&cell) && !scene.portal().core.contains(&cell));
    }
    let to = resolve_camera_motion(
        scene.world(),
        v(16.5, -7.5, 16.5),
        v(0.0, 0.0, -3.0),
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    );
    assert!(to.z < 14.0);
}

#[test]
fn the_family_focal_compositions_are_preserved() {
    let scene = WorldScene::new();
    let counts: Vec<(usize, usize, usize)> = scene
        .families()
        .iter()
        .map(|f| (f.symbols.len(), f.accents.len(), f.structures.len()))
        .collect();
    assert_eq!(counts, vec![(25, 12, 0), (21, 6, 4), (23, 10, 4)]);
    for family in scene.families() {
        for cell in family.cells() {
            assert!(
                scene.world().contains(cell),
                "{cell:?} of {:?} gone",
                family.family
            );
        }
    }
    let e = scene.red_black_expansion();
    for cell in e.cells() {
        assert!(!scene.families().iter().any(|f| f.cells().contains(&cell)));
    }
}

#[test]
fn the_lower_tip_is_preserved() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let tip = IVec3::new(r.center_x, r.lower_tip_y, r.center_z);
    assert!(scene.world().contains(tip));
    assert_eq!(scene.silhouette().tip, tip);
    let l = scene.expansion_layout();
    let rim = lower_bottom_y(l, l.fortress_terrace_min_x - 1);
    for cell in scene.red_black_expansion().cells() {
        assert!(cell.y >= rim, "{cell:?} below the rim");
    }
    assert_eq!(scene.world().bounds().unwrap().min.y, r.lower_tip_y);
}

#[test]
fn no_floating_slab_and_down_orientation() {
    let scene = WorldScene::new();
    let e = scene.red_black_expansion();
    for cell in e.cells() {
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down, "{cell:?}");
        // Structure, or one of the surface blocks the C181 pass lays over it.
        assert!(
            matches!(
                b.block_type(),
                BlockType::Deepslate
                    | BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::Mycelium
                    | BlockType::NetherWartBlock
                    | BlockType::RedBlackDeepslateBricksCrimson
                    | BlockType::RedBlackDeepslateBricksOrange
                    | BlockType::RedBlackDeepslateBricksViolet
            ),
            "{cell:?} is {:?}",
            b.block_type()
        );
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
    // The terrace steps one cell per column from the rim to the pad.
    let l = scene.expansion_layout();
    for x in l.fortress_terrace_min_x..=l.fortress_terrace_max_x {
        assert_eq!(lower_bottom_y(l, x), lower_bottom_y(l, x + 1) - 1);
    }
    assert_eq!(
        lower_bottom_y(l, l.red_black_fortress_pad.min_x),
        l.fortress_pad_bottom_y
    );
}
#[test]
fn the_voxel_budget_is_respected() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    println!("GATE15 voxels after the lower mass: {n}");
    assert!(
        n - scene.architecture_voxels() <= VOXEL_BUDGET_SOFT_MAX,
        "{n} over the soft maximum"
    );
    assert!(
        n - scene.architecture_voxels() <= VOXEL_BUDGET_TARGET + 150,
        "{n} far over the target"
    );
}

/// Gate 17 built the fortress on the lower pad: its ground row is the dark
/// foundation, level with the pad surface, on the Gate 15 shell columns.
fn pad_is_fortress_ground(scene: &WorldScene) {
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    assert_eq!(scene.fortress_layout().footprint, pad);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let ground = IVec3::new(x, l.fortress_pad_bottom_y, z);
            let b = scene
                .world()
                .get(ground)
                .unwrap_or_else(|| panic!("({x},{z}) missing"));
            assert_eq!(b.orientation(), Orientation::Down, "({x},{z})");
            assert!(
                matches!(
                    b.block_type(),
                    BlockType::PolishedBlackstoneBricks
                        | BlockType::SmoothBasalt
                        | BlockType::Deepslate
                        | BlockType::NetherWartBlock
                        | BlockType::Mycelium
                ),
                "({x},{z}) is {:?}",
                b.block_type()
            );
            assert!(scene.red_black_expansion().column(x, z).is_some());
        }
    }
}
