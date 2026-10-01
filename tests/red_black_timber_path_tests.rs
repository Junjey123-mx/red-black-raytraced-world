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
use scene::expansion::lower_ground_cell;
use scene::fortress::is_family_brick;
use scene::orientation::Orientation;
use scene::world::WorldScene;
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

fn ground(scene: &WorldScene, x: i32, z: i32) -> Option<IVec3> {
    lower_ground_cell(scene.world(), x, z, -40, 0)
}

/// The whole walk: Gate 14 landing, the corinto path, the gate corridor
/// and the courtyard up to the keep door.
fn walk_points(scene: &WorldScene) -> Vec<Vec3> {
    let r = scene.inverted_route().layout;
    let mut points = vec![eye(IVec3::new(r.x, r.platform_y, r.platform_z_max))];
    points.push(eye(scene.expansion_layout().red_black_path_anchor));
    for c in &scene.red_black_approach().main {
        points.push(eye(*c));
    }
    let l = scene.fortress_layout();
    points.extend(
        l.main_route()
            .iter()
            .take_while(|c| c.x < l.central_keep.min_x)
            .map(|c| eye(*c)),
    );
    points
}

// 1
#[test]
fn the_route_is_continuous() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    let main = &scene.red_black_approach().main;
    assert_eq!(p.planks.len() + p.accents.len(), main.len());
    // Every cell steps onto a neighbouring path cell (one column over,
    // at most one course up or down).
    for c in main {
        let next = main.iter().any(|o| {
            o != c && (o.x - c.x).abs() + (o.z - c.z).abs() == 1 && (o.y - c.y).abs() <= 1
        });
        assert!(next, "{c:?} is isolated");
    }
    // Two wide along both legs.
    let cols: HashSet<(i32, i32)> = p.columns().into_iter().collect();
    for c in main {
        let partner = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .any(|(dx, dz)| cols.contains(&(c.x + dx, c.z + dz)));
        assert!(partner, "{c:?} is a one-wide neck");
    }
    println!(
        "GATE17_5 path: planks={} accents={} borders={} fences={} identity_voxels={} voxels={}",
        p.planks.len(),
        p.accents.len(),
        p.borders.len(),
        p.fences.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn the_materials_are_corinto_timber_with_dark_borders() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    assert!(
        p.planks.len() * 10 >= (p.planks.len() + p.accents.len()) * 9,
        "planks dominate the path"
    );
    for c in &p.planks {
        let b = scene.world().get(*c).unwrap();
        assert_eq!(b.block_type(), BlockType::RedBlackWoodPlanks);
        assert_eq!(b.orientation(), Orientation::Down);
        assert_eq!(
            ground(&scene, c.x, c.z),
            Some(*c),
            "{c:?} is the ground of its column"
        );
    }
    for c in &p.accents {
        assert!(is_family_brick(scene.world().get(*c).unwrap().block_type()));
    }
    assert!(!p.borders.is_empty());
    assert!(!p.fences.is_empty(), "selected edges carry fence posts");
    for c in &p.borders {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks
        );
        // Ground of its column (the lobe is a hollow shell, so nothing need
        // stand above it): nothing below but its own post.
        let below = IVec3::new(c.x, c.y - 1, c.z);
        assert!(
            !scene.world().contains(below) || p.fences.contains(&below),
            "{c:?}"
        );
    }
    for c in &p.fences {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackFence
        );
        assert!(
            p.borders.contains(&IVec3::new(c.x, c.y + 1, c.z)),
            "{c:?} stands on a border"
        );
    }
}

// 3
#[test]
fn the_path_ends_at_the_gate() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let p = &scene.red_black_identity().path;
    let last = *scene.red_black_approach().main.last().unwrap();
    assert_eq!(
        last.x + 1,
        l.gate.base.x,
        "the last cell meets the gate wall"
    );
    let gate_rows: Vec<i32> = l.gate.columns().iter().map(|(_, z)| *z).collect();
    let forecourt: Vec<&IVec3> = p
        .planks
        .iter()
        .filter(|c| c.x == l.gate.base.x - 1)
        .collect();
    assert!(
        forecourt.iter().any(|c| gate_rows.contains(&c.z)),
        "a plank lies right in front of the gate"
    );
    // The anchor end starts at the Gate 14 landing.
    let first = scene.red_black_approach().main[0];
    let anchor = scene.expansion_layout().red_black_path_anchor;
    assert!((first.x - anchor.x).abs() <= 1 && (first.z - anchor.z).abs() <= 1);
}

// 4
#[test]
fn there_is_no_dead_end() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    let cols: HashSet<(i32, i32)> = p.columns().into_iter().collect();
    let start = (p.planks[0].x, p.planks[0].z);
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some((x, z)) = stack.pop() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (x + dx, z + dz);
            if cols.contains(&n) && seen.insert(n) {
                stack.push(n);
            }
        }
    }
    assert_eq!(seen.len(), cols.len(), "the path is one piece");
}

// 5
#[test]
fn the_path_runs_both_ways() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    walk_both_ways(&scene, &walk_points(&scene));
}

// 6
#[test]
fn fences_do_not_block_the_path() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    let cols: HashSet<(i32, i32)> = p.columns().into_iter().collect();
    for f in &p.fences {
        assert!(!cols.contains(&(f.x, f.z)), "{f:?} stands on the path");
        assert!(is_camera_solid(scene.world().get(*f).unwrap()));
    }
    for c in p.planks.iter().chain(p.accents.iter()) {
        for dy in 1..=2 {
            assert!(
                !scene.world().contains(IVec3::new(c.x, c.y - dy, c.z)),
                "{c:?}"
            );
        }
        assert!(clear(&scene, eye(*c)), "{c:?}");
    }
}

// 7
#[test]
fn no_noclip_is_needed() {
    let scene = WorldScene::new();
    let state = CollisionState::new();
    assert!(state.collision_enabled());
    let points = walk_points(&scene);
    walk(&scene, &points);
    let src = std::fs::read_to_string("src/scene/red_black_identity.rs").unwrap();
    assert!(!src.contains("noclip"));
}

// 8
#[test]
fn the_path_is_distinct_from_the_terrain_by_block_identity() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    let cols: HashSet<(i32, i32)> = p.columns().into_iter().collect();
    let pad = scene.fortress_layout().footprint;
    // Around the path: no other ground cell is corinto timber, so the
    // planks alone mark the route.
    for x in 15..pad.min_x {
        for z in 0..16 {
            if cols.contains(&(x, z)) {
                continue;
            }
            if let Some(g) = ground(&scene, x, z) {
                assert_ne!(
                    scene.world().get(g).unwrap().block_type(),
                    BlockType::RedBlackWoodPlanks,
                    "({x},{z})"
                );
            }
        }
    }
}
