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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
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
    CameraCollisionConfig, CollisionState, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
use scene::light::Light;
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
use scene::red_black_timber::is_red_black_timber;
use scene::world::{WorldScene, world_lights};
use std::collections::HashSet;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn cfg() -> CameraCollisionConfig {
    CameraCollisionConfig::default()
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(scene.world(), p, &cfg(), &is_camera_solid)
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(scene.world(), from, delta, &cfg(), &is_camera_solid)
}

/// The inverted viewer's eye under a ground cell.
fn eye(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 - 1.5, c.z as f32 + 0.5)
}

fn step(scene: &WorldScene, from: Vec3, target: Vec3) -> Vec3 {
    let direct = resolve(scene, from, target - from);
    if (direct - target).length() < 1e-3 {
        return direct;
    }
    let (f, t) = (from, target);
    for corner in [
        v(t.x, f.y, t.z),
        v(f.x, t.y, f.z),
        v(t.x, f.y, f.z),
        v(f.x, f.y, t.z),
        v(t.x, t.y, f.z),
        v(f.x, t.y, t.z),
    ] {
        let mid = resolve(scene, from, corner - from);
        if (mid - corner).length() < 1e-3 {
            let end = resolve(scene, mid, target - mid);
            if (end - target).length() < 1e-3 {
                return end;
            }
        }
    }
    direct
}

fn walk(scene: &WorldScene, points: &[Vec3]) {
    let mut position = points[0];
    for (i, target) in points.iter().enumerate().skip(1) {
        let reached = step(scene, position, *target);
        assert!(
            (reached - *target).length() < 1e-3,
            "segment {i}: stuck at {reached:?} short of {target:?}"
        );
        position = reached;
    }
}

fn walk_both_ways(scene: &WorldScene, points: &[Vec3]) {
    walk(scene, points);
    let mut back = points.to_vec();
    back.reverse();
    walk(scene, &back);
}

/// Every fortress cell: the Gate 17 stages, the pad row and the timber layer.
fn fortress_body(scene: &WorldScene) -> Vec<IVec3> {
    let l = scene.fortress_layout();
    let pad = l.footprint;
    let mut cells: HashSet<IVec3> = scene.fortress_cells().into_iter().collect();
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            cells.insert(IVec3::new(x, l.levels.ground_y, z));
        }
    }
    cells.extend(scene.red_black_identity().timber.cells());
    cells.into_iter().collect()
}

// 1
#[test]
fn timber_is_present_in_the_fortress() {
    let scene = WorldScene::new();
    let t = &scene.red_black_identity().timber;
    let l = scene.fortress_layout();
    assert!(t.planks.len() >= 40, "{}", t.planks.len());
    assert!(!t.balconies.is_empty() && !t.rails.is_empty());
    for c in &t.planks {
        let b = scene.world().get(*c).unwrap();
        assert_eq!(b.block_type(), BlockType::RedBlackWoodPlanks);
        assert_eq!(b.orientation(), Orientation::Down);
        assert!(l.footprint.contains(c.x, c.z));
    }
    for c in &t.rails {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackFence
        );
        assert!(
            scene.world().contains(IVec3::new(c.x, c.y + 1, c.z)),
            "{c:?} floats"
        );
    }
    for c in &t.balconies {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
        // Attached to the keep wall beside it.
        let wall = IVec3::new(
            c.x,
            c.y,
            if c.z < l.central_keep.min_z {
                c.z + 1
            } else {
                c.z - 1
            },
        );
        assert!(scene.world().contains(wall), "{c:?} is not attached");
    }
    // The hall floor and the keep roof are timber.
    let h = l.hall;
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            for y in [l.levels.ground_y, l.levels.keep_roof_y] {
                let bt = scene.world().get(IVec3::new(x, y, z)).unwrap().block_type();
                assert!(
                    bt == BlockType::RedBlackWoodPlanks || !is_dark_structure(bt),
                    "({x},{y},{z}) {bt:?}"
                );
            }
        }
    }
    println!(
        "GATE17_5 timber: planks={} balconies={} rails={} identity_voxels={} voxels={}",
        t.planks.len(),
        t.balconies.len(),
        t.rails.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn dark_stone_stays_the_majority() {
    let scene = WorldScene::new();
    let body = fortress_body(&scene);
    let kind = |c: &IVec3| scene.world().get(*c).unwrap().block_type();
    let dark = body.iter().filter(|c| is_dark_structure(kind(c))).count();
    let timber = body.iter().filter(|c| is_red_black_timber(kind(c))).count();
    let accents = body.len() - dark - timber;
    let n = body.len() as f32;
    println!(
        "GATE17_5 fortress balance: cells={} dark={dark} ({:.0}%) timber={timber} ({:.0}%) accents={accents} ({:.0}%)",
        body.len(),
        dark as f32 / n * 100.0,
        timber as f32 / n * 100.0,
        accents as f32 / n * 100.0
    );
    assert!(dark as f32 / n >= 0.55, "dark {dark} of {n}");
    assert!(dark > timber * 2 && dark > accents * 2);
    assert!(timber as f32 / n >= 0.08, "timber {timber} of {n}");
}

// 3
#[test]
fn critical_supports_are_unchanged() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let timber: HashSet<IVec3> = scene
        .red_black_identity()
        .timber
        .cells()
        .into_iter()
        .collect();
    // Curtain wall, tower shells, keep walls and pillars keep their stone.
    for c in &scene.fortress_foundation().walls {
        assert!(!timber.contains(c), "{c:?} curtain");
    }
    for t in scene.fortress_towers() {
        for c in t.shell.iter().chain(t.band.iter()) {
            assert!(!timber.contains(c), "{c:?} tower");
        }
    }
    let k = scene.fortress_keep();
    for c in k.shell.iter().chain(k.pillars.iter()) {
        assert!(!timber.contains(c), "{c:?} keep");
    }
    // The upper floor's beams stay structural: basalt, or the corinto log
    // beams the final balance pass (C220) lays.
    for c in &k.upper_floor {
        if c.z == l.hall.min_z || c.z == l.hall.max_z {
            assert!(
                matches!(
                    scene.world().get(*c).unwrap().block_type(),
                    BlockType::SmoothBasalt | BlockType::RedBlackLog
                ),
                "{c:?}"
            );
        }
    }
    for c in &scene.fortress_foundation().footings {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks,
            "{c:?}"
        );
    }
}

// 4
#[test]
fn paths_and_rooms_stay_traversable() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let l = scene.fortress_layout();
    for route in &l.interior_routes {
        let pts: Vec<Vec3> = route.iter().map(|c| eye(*c)).collect();
        walk_both_ways(&scene, &pts);
    }
    let mut approach: Vec<Vec3> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| eye(*c))
        .collect();
    approach.extend(l.main_route().iter().map(|c| eye(*c)));
    walk_both_ways(&scene, &approach);
    for (x, z) in &l.courtyard {
        assert!(
            clear(&scene, eye(IVec3::new(*x, l.levels.ground_y, *z))),
            "({x},{z})"
        );
    }
}

// 5
#[test]
fn every_symbol_is_preserved() {
    let scene = WorldScene::new();
    let timber: HashSet<IVec3> = scene
        .red_black_identity()
        .timber
        .cells()
        .into_iter()
        .collect();
    let mut symbols = 0;
    for t in scene.fortress_towers() {
        for (c, bt) in &t.crests {
            assert_eq!(scene.world().get(*c).unwrap().block_type(), *bt);
            assert!(!timber.contains(c));
            symbols += 1;
        }
    }
    for (c, bt) in scene
        .fortress_keep()
        .symbols()
        .iter()
        .chain(scene.fortress_hall().crest.iter())
    {
        assert_eq!(scene.world().get(*c).unwrap().block_type(), *bt);
        symbols += 1;
    }
    assert_eq!(symbols, 21);
    for c in scene.fortress_bricks().cells() {
        assert!(is_family_brick(scene.world().get(c).unwrap().block_type()));
    }
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        assert!(scene.fortress_tower(f).is_some());
    }
    for c in timber {
        for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            let _ = scene
                .world()
                .get(n)
                .is_some_and(|b| is_symbol(b.block_type()));
        }
    }
}

// 6
#[test]
fn no_point_light_is_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
}

// 7
#[test]
fn no_portal_geometry_is_changed() {
    let scene = WorldScene::new();
    let portal = scene.portal();
    assert_eq!(portal.frame.len(), 18);
    assert_eq!(portal.core.len(), 12);
    for c in &portal.core {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PortalCoreDarkCrimson
        );
    }
    let route = scene.inverted_route();
    let timber: HashSet<IVec3> = scene
        .red_black_identity()
        .timber
        .cells()
        .into_iter()
        .collect();
    for (c, _) in &route.treads {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::WoodStairs
        );
        assert!(!timber.contains(c));
    }
    for c in route.layout.platform_cells() {
        assert_eq!(
            scene.world().get(c).unwrap().block_type(),
            BlockType::DeepslateBricks
        );
    }
    let src = std::fs::read_to_string("src/scene/red_black_identity.rs").unwrap();
    assert!(!src.contains("PortalCore") && !src.contains("portal_crossing"));
}
