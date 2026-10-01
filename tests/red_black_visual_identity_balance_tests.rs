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
    CameraCollisionConfig, CollisionState, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::expansion::lower_ground_cell;
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
use scene::light::Light;
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

/// The fortress body: Gate 17 cells, the pad row and every Gate 17.5
/// fortress stage (timber, gatehouse, fourth tower, balance).
fn body(scene: &WorldScene) -> Vec<IVec3> {
    let l = scene.fortress_layout();
    let id = scene.red_black_identity();
    let mut cells: HashSet<IVec3> = scene.fortress_cells().into_iter().collect();
    for z in l.footprint.min_z..=l.footprint.max_z {
        for x in l.footprint.min_x..=l.footprint.max_x {
            cells.insert(IVec3::new(x, l.levels.ground_y, z));
        }
    }
    cells.extend(id.timber.cells());
    cells.extend(id.gatehouse.added_cells());
    cells.extend(id.gatehouse.corridor_floor.iter().copied());
    cells.extend(id.fourth_tower.cells());
    cells.extend(id.balance.cells());
    cells.into_iter().collect()
}

fn is_magic(t: BlockType) -> bool {
    is_family_brick(t)
        || is_symbol(t)
        || matches!(
            t,
            BlockType::CryingObsidianCrimson
                | BlockType::CryingObsidianOrange
                | BlockType::CryingObsidianViolet
                | BlockType::BuddingAmethyst
                | BlockType::AmethystCluster
        )
}

// 1
#[test]
fn the_timber_ratio_is_bounded() {
    let scene = WorldScene::new();
    let cells = body(&scene);
    let kind = |c: &IVec3| scene.world().get(*c).unwrap().block_type();
    let n = cells.len() as f32;
    let dark = cells.iter().filter(|c| is_dark_structure(kind(c))).count() as f32;
    let timber = cells
        .iter()
        .filter(|c| is_red_black_timber(kind(c)))
        .count() as f32;
    let magic = cells.iter().filter(|c| is_magic(kind(c))).count() as f32;
    println!(
        "GATE17_5 balance: cells={} dark={:.0}% timber={:.0}% magic={:.0}% other={:.0}% balance_cells={}",
        cells.len(),
        dark / n * 100.0,
        timber / n * 100.0,
        magic / n * 100.0,
        (n - dark - timber - magic) / n * 100.0,
        scene.red_black_identity().balance.cells().len()
    );
    assert!(
        timber / n >= 0.15 && timber / n <= 0.25,
        "timber {:.2}",
        timber / n
    );
    assert!(magic / n <= 0.25, "magic {:.2}", magic / n);
}

// 2
#[test]
fn dark_stone_remains_dominant() {
    let scene = WorldScene::new();
    let cells = body(&scene);
    let kind = |c: &IVec3| scene.world().get(*c).unwrap().block_type();
    let n = cells.len() as f32;
    let dark = cells.iter().filter(|c| is_dark_structure(kind(c))).count() as f32;
    let timber = cells
        .iter()
        .filter(|c| is_red_black_timber(kind(c)))
        .count() as f32;
    assert!(dark / n >= 0.55 && dark / n <= 0.72, "dark {:.2}", dark / n);
    assert!(dark > timber * 3.0);
    // The balance pass only re-lays: every cell it touched is timber now.
    let b = &scene.red_black_identity().balance;
    assert!(!b.hoardings.is_empty() && !b.walkways.is_empty() && !b.beams.is_empty());
    for c in b.hoardings.iter().chain(b.walkways.iter()) {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
    }
    for c in &b.beams {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackLog
        );
    }
}

// 3
#[test]
fn all_three_families_are_present() {
    let scene = WorldScene::new();
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        let t = scene.fortress_tower(f).unwrap();
        for (c, bt) in &t.crests {
            assert_eq!(scene.world().get(*c).unwrap().block_type(), *bt);
        }
    }
    let cells = body(&scene);
    for want in [
        BlockType::RedBlackDeepslateBricksCrimson,
        BlockType::RedBlackDeepslateBricksOrange,
        BlockType::RedBlackDeepslateBricksViolet,
        BlockType::CrimsonHeart,
        BlockType::OrangeClub,
        BlockType::PurpleSpade,
    ] {
        assert!(
            cells
                .iter()
                .any(|c| scene.world().get(*c).unwrap().block_type() == want),
            "{want:?}"
        );
    }
    // The balance pass moved no accent: its cells were all dark before.
    for c in scene.red_black_identity().balance.cells() {
        assert!(!scene.fortress_bricks().cells().contains(&c));
    }
}

// 4
#[test]
fn the_path_material_is_distinct() {
    let scene = WorldScene::new();
    let p = &scene.red_black_identity().path;
    let cols: HashSet<(i32, i32)> = p.columns().into_iter().collect();
    for c in &p.planks {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
    }
    // Terrain ≠ path: outside the fortress, ground timber exists only on the path.
    let pad = scene.fortress_layout().footprint;
    for x in 15..pad.min_x {
        for z in 0..16 {
            if cols.contains(&(x, z)) {
                continue;
            }
            if let Some(g) = lower_ground_cell(scene.world(), x, z, -40, 0) {
                assert_ne!(
                    scene.world().get(g).unwrap().block_type(),
                    BlockType::RedBlackWoodPlanks,
                    "({x},{z})"
                );
            }
        }
    }
}

// 5
#[test]
fn the_gatehouse_exists() {
    let scene = WorldScene::new();
    let g = &scene.red_black_identity().gatehouse;
    assert_eq!(g.door.len(), 4);
    for c in &g.door {
        assert!(is_camera_passable(
            scene.world().get(*c).unwrap().block_type()
        ));
    }
    assert!(!g.hood.is_empty() && !g.corridor_floor.is_empty());
}

// 6
#[test]
fn four_towers_exist() {
    let scene = WorldScene::new();
    assert_eq!(scene.fortress_towers().len() + 1, 4);
    let t4 = &scene.red_black_identity().fourth_tower;
    assert_eq!(t4.finials.len(), 4);
    assert_eq!(t4.crests.len(), 2);
}

// 7
#[test]
fn trees_exist() {
    let scene = WorldScene::new();
    let n = scene.red_black_identity().grove.trees.len();
    assert!((4..=7).contains(&n), "{n}");
    for t in &scene.red_black_identity().grove.trees {
        assert!(!t.trunk.is_empty() && !t.leaves.is_empty());
    }
}

// 8
#[test]
fn no_new_point_light_exists() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    for p in [
        "src/scene/red_black_identity.rs",
        "src/scene/red_black_timber.rs",
    ] {
        let src = std::fs::read_to_string(p).unwrap();
        assert!(
            !src.contains("PointLight") && !src.contains("emission"),
            "{p}"
        );
    }
}

// 9
#[test]
fn the_canonical_route_is_clear() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let l = scene.fortress_layout();
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    points.extend(scene.red_black_approach().main.iter().map(|c| eye(*c)));
    points.extend(l.main_route().iter().map(|c| eye(*c)));
    walk_both_ways(&scene, &points);
    for route in &l.interior_routes[1..] {
        let pts: Vec<Vec3> = route.iter().map(|c| eye(*c)).collect();
        walk_both_ways(&scene, &pts);
    }
    for c in &scene.red_black_identity().balance.walkways {
        assert!(clear(&scene, eye(*c)), "{c:?}");
    }
}
