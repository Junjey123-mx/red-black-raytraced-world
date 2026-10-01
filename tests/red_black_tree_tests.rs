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
use scene::fortress::is_symbol;
use scene::light::Light;
use scene::orientation::Orientation;
use scene::red_black_identity::{
    RED_BLACK_TREES_MAX, RED_BLACK_TREES_MIN, TREE_FOCAL_CLEARANCE, TREE_SPACING,
};
use scene::red_black_timber::{red_black_leaves_material_id, red_black_log_material_id};
use scene::world::{WorldScene, world_lights};

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

fn tree_cells(scene: &WorldScene) -> Vec<IVec3> {
    scene.red_black_identity().grove.cells()
}

// 1
#[test]
fn the_tree_count_is_in_range() {
    let scene = WorldScene::new();
    let trees = &scene.red_black_identity().grove.trees;
    assert!(
        trees.len() >= RED_BLACK_TREES_MIN && trees.len() <= RED_BLACK_TREES_MAX,
        "{}",
        trees.len()
    );
    for (i, a) in trees.iter().enumerate() {
        for b in &trees[i + 1..] {
            let d = (a.base.x - b.base.x).abs().max((a.base.z - b.base.z).abs());
            assert!(d >= TREE_SPACING, "{:?} {:?}", a.base, b.base);
        }
        assert!(a.trunk_height >= 4, "{}", a.trunk_height);
        assert!(a.leaves.len() >= 8, "{} leaves", a.leaves.len());
    }
    for t in trees {
        println!(
            "GATE17_5 tree base={:?} trunk={} leaves={}",
            t.base,
            t.trunk_height,
            t.leaves.len()
        );
    }
    println!(
        "GATE17_5 grove: trees={} cells={} identity_voxels={} voxels={}",
        trees.len(),
        tree_cells(&scene).len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn trees_are_built_from_the_new_log_and_leaves() {
    let scene = WorldScene::new();
    for t in &scene.red_black_identity().grove.trees {
        for c in &t.trunk {
            let b = scene.world().get(*c).unwrap();
            assert_eq!(b.block_type(), BlockType::RedBlackLog);
            assert_eq!(b.material_id(), red_black_log_material_id());
            assert_eq!(b.orientation(), Orientation::Down);
        }
        for c in &t.leaves {
            let b = scene.world().get(*c).unwrap();
            assert_eq!(b.block_type(), BlockType::RedBlackLeaves);
            assert_eq!(b.material_id(), red_black_leaves_material_id());
            assert_eq!(b.orientation(), Orientation::Down);
        }
        // Inverted growth: the trunk hangs from the ground toward -Y.
        assert_eq!(t.trunk[0], IVec3::new(t.base.x, t.base.y - 1, t.base.z));
        for w in t.trunk.windows(2) {
            assert_eq!(w[1].y, w[0].y - 1);
        }
        let end = t.trunk.last().unwrap().y;
        assert!(
            t.leaves.iter().all(|c| c.y <= end + 1),
            "the crown sits around the trunk end"
        );
        // Grounded: the base is the lowest cell of its column in the lower world.
        assert!(scene.world().contains(t.base));
        assert!(t.base.y < 0);
    }
}

// 3
#[test]
fn no_overworld_material_is_used() {
    let scene = WorldScene::new();
    for c in tree_cells(&scene) {
        let t = scene.world().get(c).unwrap().block_type();
        assert!(
            !matches!(t, BlockType::Log | BlockType::Leaves),
            "{c:?} is {t:?}"
        );
        assert!(c.y < 0, "{c:?} is in the Overworld");
    }
    // The Overworld trees keep their own blocks.
    for t in scene.trees() {
        for c in &t.trunk {
            assert_eq!(scene.world().get(*c).unwrap().block_type(), BlockType::Log);
        }
    }
}

// 4
#[test]
fn the_placement_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.red_black_identity().grove, b.red_black_identity().grove);
    for c in tree_cells(&a) {
        assert_eq!(a.world().get(c), b.world().get(c));
    }
    assert_eq!(a.world().len(), b.world().len());
}

// 5
#[test]
fn the_paths_keep_their_head_room() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let walk: Vec<IVec3> = scene
        .red_black_approach()
        .main
        .iter()
        .copied()
        .chain(l.interior_routes.iter().flatten().copied())
        .collect();
    let cells = tree_cells(&scene);
    for g in &walk {
        for dy in 1..=2 {
            let c = IVec3::new(g.x, g.y - dy, g.z);
            assert!(!cells.contains(&c), "{c:?} blocks the path");
        }
        assert!(clear(&scene, eye(*g)), "{g:?}");
    }
    // Clear of the portal route and every family focal cell.
    let route = scene.inverted_route();
    for c in &cells {
        assert!(!route.treads.iter().any(|(t, _)| t == c) && !route.supports.contains(c));
        for f in scene.families() {
            for fc in f.symbols.iter().chain(f.accents.iter()) {
                let far = (fc.x - c.x).abs() >= TREE_FOCAL_CLEARANCE
                    || (fc.z - c.z).abs() >= TREE_FOCAL_CLEARANCE;
                assert!(far, "{c:?} crowds {fc:?}");
            }
        }
    }
    // No tree cell touches a symbol block.
    for c in &cells {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            assert!(
                !scene
                    .world()
                    .get(n)
                    .is_some_and(|b| is_symbol(b.block_type())),
                "{c:?} touches a symbol"
            );
        }
    }
}

// 6
#[test]
fn the_gate_stays_visible_from_the_approach() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let cells = tree_cells(&scene);
    // Sight lines from every approach eye to every gate cell cross no leaf.
    for g in &scene.red_black_approach().main {
        let from = eye(*g);
        for target in l.gate.cells() {
            let to = v(
                target.x as f32 + 0.5,
                target.y as f32 + 0.5,
                target.z as f32 + 0.5,
            );
            for i in 0..=40 {
                let p = from + (to - from) * (i as f32 / 40.0);
                let cell = IVec3::new(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
                assert!(!cells.contains(&cell), "{cell:?} hides the gate from {g:?}");
            }
        }
    }
}

// 7
#[test]
fn the_collision_route_is_unchanged() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    for c in &scene.red_black_approach().main {
        points.push(eye(*c));
    }
    for c in scene.fortress_layout().main_route() {
        points.push(eye(*c));
    }
    walk_both_ways(&scene, &points);
    for c in tree_cells(&scene) {
        assert!(
            is_camera_solid(scene.world().get(c).unwrap()),
            "trees are solid like the Overworld's"
        );
    }
}

// 8
#[test]
fn no_light_is_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    let src = std::fs::read_to_string("src/scene/red_black_identity.rs").unwrap();
    assert!(!src.contains("PointLight") && !src.contains("Light::"));
    // Inside the world box: the crowns never reach past the lower tip.
    let scene = WorldScene::new();
    let tip = scene.rhombus().lower_tip_y;
    assert!(tree_cells(&scene).iter().all(|c| c.y >= tip));
    assert_eq!(scene.world().bounds().unwrap().min.y, tip);
}
