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
use scene::orientation::Orientation;
use scene::world::WorldScene;
use std::collections::{HashSet, VecDeque};

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

/// Gate 17 final voxel count (HEAD 807e490).
const GATE_17_VOXELS: usize = 9083;
/// Soft ceiling of the Gate 17.5 budget.
const IDENTITY_BUDGET_MAX: usize = 1400;

const N6: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

// 1
#[test]
fn the_world_is_one_connected_component() {
    let scene = WorldScene::new();
    let cells: HashSet<IVec3> = scene.world().iter().map(|(c, _)| *c).collect();
    let start = *scene.house().floor.first().unwrap();
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(c) = queue.pop_front() {
        for (dx, dy, dz) in N6 {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            if cells.contains(&n) && seen.insert(n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(seen.len(), cells.len());
    let ls = &scene.red_black_identity().landscape;
    println!(
        "GATE17_5 landscape: columns={} cells={} identity_voxels={} voxels={}",
        ls.columns.len(),
        ls.cells.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn no_floating_island_is_created() {
    let scene = WorldScene::new();
    let ls = &scene.red_black_identity().landscape;
    assert!(ls.columns.len() >= 10, "{}", ls.columns.len());
    let own: HashSet<IVec3> = ls.cells.iter().copied().collect();
    // Every new column touches the existing mass or a column that does.
    let mut attached: HashSet<(i32, i32)> = HashSet::new();
    for _ in 0..4 {
        for c in &ls.cells {
            let touches = N6.iter().any(|(dx, dy, dz)| {
                let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
                (scene.world().contains(n) && !own.contains(&n))
                    || (own.contains(&n) && attached.contains(&(n.x, n.z)))
            });
            if touches {
                attached.insert((c.x, c.z));
            }
        }
    }
    for col in &ls.columns {
        assert!(attached.contains(col), "{col:?} floats");
    }
    // A shoulder, not a slab: ragged outer row, one-course relief.
    let rows: HashSet<i32> = ls.columns.iter().map(|(_, z)| *z).collect();
    assert_eq!(rows.len(), 2);
    let heights: HashSet<i32> = ls.ground.iter().map(|c| c.y).collect();
    assert!(heights.len() >= 3, "{heights:?}");
    let outer = rows.iter().min().unwrap();
    let inner = rows.iter().max().unwrap();
    let n_outer = ls.columns.iter().filter(|(_, z)| z == outer).count();
    let n_inner = ls.columns.iter().filter(|(_, z)| z == inner).count();
    assert!(n_outer < n_inner, "the outer row is ragged");
}

// 3
#[test]
fn the_new_surface_faces_minus_y() {
    let scene = WorldScene::new();
    let ls = &scene.red_black_identity().landscape;
    let trees: HashSet<(i32, i32)> = scene
        .red_black_identity()
        .grove
        .cells()
        .iter()
        .map(|c| (c.x, c.z))
        .collect();
    for c in &ls.cells {
        assert_eq!(
            scene.world().get(*c).unwrap().orientation(),
            Orientation::Down,
            "{c:?}"
        );
        assert!(c.y < 0);
    }
    for g in &ls.ground {
        let t = scene.world().get(*g).unwrap().block_type();
        assert!(
            matches!(
                t,
                BlockType::SmoothBasalt | BlockType::Mycelium | BlockType::PolishedBlackstoneBricks
            ),
            "{g:?} {t:?}"
        );
        if trees.contains(&(g.x, g.z)) {
            continue;
        }
        let below_free = !scene.world().contains(IVec3::new(g.x, g.y - 1, g.z))
            || scene
                .red_black_identity()
                .path
                .fences
                .contains(&IVec3::new(g.x, g.y - 1, g.z));
        assert!(below_free, "{g:?} is not the ground");
    }
}

// 4
#[test]
fn the_portal_is_not_affected() {
    let scene = WorldScene::new();
    let ls = &scene.red_black_identity().landscape;
    let route = scene.inverted_route();
    let r: HashSet<IVec3> = route
        .treads
        .iter()
        .map(|(c, _)| *c)
        .chain(route.supports.iter().copied())
        .chain(route.layout.platform_cells())
        .chain(route.layout.exit_clearance())
        .collect();
    for c in &ls.cells {
        assert!(!r.contains(c));
        assert!(c.x > route.layout.x, "{c:?} reaches the portal route");
    }
    assert_eq!(scene.portal().frame.len(), 18);
    assert_eq!(scene.portal().core.len(), 12);
    for c in route.layout.exit_clearance() {
        assert!(!scene.world().contains(c));
    }
}

// 5
#[test]
fn the_path_is_still_continuous() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let l = scene.fortress_layout();
    let r = scene.inverted_route().layout;
    let mut points = vec![eye(IVec3::new(r.x, r.platform_y, r.platform_z_max))];
    points.extend(scene.red_black_approach().main.iter().map(|c| eye(*c)));
    points.extend(l.main_route().iter().map(|c| eye(*c)));
    walk_both_ways(&scene, &points);
    // The shoulder beside the path's east-west leg is walkable ground too.
    for g in &scene.red_black_identity().landscape.ground {
        if g.z == scene.expansion_layout().red_black_extension.min_z - 1
            && g.x <= l.footprint.min_x - 1
        {
            let path = scene
                .red_black_approach()
                .main
                .iter()
                .find(|p| p.x == g.x && p.z == g.z + 1);
            if let Some(p) = path {
                if (p.y - g.y).abs() <= 1 {
                    walk_both_ways(&scene, &[eye(*p), eye(*g)]);
                }
            }
        }
    }
}

// 6
#[test]
fn the_tree_pads_are_supported() {
    let scene = WorldScene::new();
    for t in &scene.red_black_identity().grove.trees {
        assert!(scene.world().contains(t.base), "{:?}", t.base);
        let neighbours = N6
            .iter()
            .filter(|(dx, dy, dz)| {
                scene
                    .world()
                    .contains(IVec3::new(t.base.x + dx, t.base.y + dy, t.base.z + dz))
            })
            .count();
        assert!(neighbours >= 2, "{:?} hangs by a thread", t.base);
        assert!(!matches!(
            scene.world().get(t.base).unwrap().block_type(),
            BlockType::RedBlackLeaves | BlockType::RedBlackLog
        ));
    }
}

// 7
#[test]
fn the_fourth_tower_is_supported() {
    let scene = WorldScene::new();
    let t4 = &scene.red_black_identity().fourth_tower;
    let ground = scene.fortress_layout().levels.ground_y;
    for z in t4.bounds.min_z..=t4.bounds.max_z {
        for x in t4.bounds.min_x..=t4.bounds.max_x {
            assert!(
                scene.world().contains(IVec3::new(x, ground, z)),
                "({x},{z}) has no footing"
            );
        }
    }
    for c in t4.added_cells() {
        let attached = N6.iter().any(|(dx, dy, dz)| {
            scene
                .world()
                .contains(IVec3::new(c.x + dx, c.y + dy, c.z + dz))
        });
        assert!(attached, "{c:?}");
    }
}

// 8
#[test]
fn the_voxel_budget_holds() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    let identity = scene.identity_voxels();
    assert_eq!(
        n,
        GATE_17_VOXELS + identity,
        "cells outside the identity stages changed"
    );
    assert!(identity <= IDENTITY_BUDGET_MAX, "{identity}");
    let b = scene.world().bounds().unwrap();
    assert_eq!(b.min, IVec3::new(0, -22, 0));
    assert_eq!(b.max_exclusive, IVec3::new(37, 13, 24));
    println!("GATE17_5 voxels: gate17={GATE_17_VOXELS} now={n} identity={identity}");
}
