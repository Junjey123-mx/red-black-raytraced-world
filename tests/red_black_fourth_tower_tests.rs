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
use scene::orientation::Orientation;
use scene::red_black_identity::missing_tower_corner;
use scene::red_black_maze::Family;
use scene::world::WorldScene;

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

// 1
#[test]
fn the_fortress_has_four_towers() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    assert_eq!(l.towers().len() + 1, 4);
    assert_eq!(scene.fortress_towers().len(), 3);
    assert!(t4.bounds.width() == 3 && t4.bounds.depth() == 3);
    let f = l.footprint;
    let corners = [
        (f.min_x, f.min_z),
        (f.max_x, f.min_z),
        (f.min_x, f.max_z),
        (f.max_x, f.max_z),
    ];
    for (x, z) in corners {
        let covered =
            l.towers().iter().any(|t| t.bounds.contains(x, z)) || t4.bounds.contains(x, z);
        assert!(covered, "corner ({x},{z}) has no tower");
    }
    println!(
        "GATE17_5 towers: crimson={:?} orange={:?} violet={:?} fourth={:?} shell={} seam={} band={} crests={} finials={} cap={} replaced={} identity_voxels={} voxels={}",
        l.crimson_tower.bounds,
        l.orange_tower.bounds,
        l.violet_tower.bounds,
        t4.bounds,
        t4.shell.len(),
        t4.seam.len(),
        t4.band.len(),
        t4.crests.len(),
        t4.finials.len(),
        t4.cap.len(),
        t4.replaced.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn the_fourth_tower_bounds_are_valid() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    assert_eq!(missing_tower_corner(l), Some(t4.bounds));
    let f = l.footprint;
    assert!(
        t4.bounds.min_x >= f.min_x
            && t4.bounds.max_x <= f.max_x
            && t4.bounds.min_z >= f.min_z
            && t4.bounds.max_z <= f.max_z
    );
    for t in l.towers() {
        let a = t.bounds;
        let b = t4.bounds;
        assert!(
            a.max_x < b.min_x || b.max_x < a.min_x || a.max_z < b.min_z || b.max_z < a.min_z,
            "{:?} overlaps",
            t.family
        );
    }
    for c in t4.cells() {
        assert!(t4.bounds.contains(c.x, c.z), "{c:?}");
        assert!(c.y < l.levels.ground_y && c.y >= l.levels.max_y);
    }
}

// 3
#[test]
fn it_does_not_overlap_the_gatehouse() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    let g = l.gatehouse;
    assert!(g.max_x < t4.bounds.min_x || g.max_z < t4.bounds.min_z || t4.bounds.max_z < g.min_z);
    for c in &scene.red_black_identity().gatehouse.door {
        assert!(!t4.bounds.contains(c.x, c.z));
    }
}

// 4
#[test]
fn no_path_is_blocked() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    let mut approach: Vec<Vec3> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| eye(*c))
        .collect();
    approach.extend(l.main_route().iter().map(|c| eye(*c)));
    walk_both_ways(&scene, &approach);
    for route in &l.interior_routes[1..] {
        let pts: Vec<Vec3> = route.iter().map(|c| eye(*c)).collect();
        walk_both_ways(&scene, &pts);
    }
    for (x, z) in &l.courtyard {
        if t4.bounds.contains(*x, *z)
            && t4.is_shell(*x, *z)
            && !t4.entrance.iter().any(|e| e.x == *x && e.z == *z)
        {
            continue;
        }
        assert!(
            clear(&scene, eye(IVec3::new(*x, l.levels.ground_y, *z))),
            "({x},{z})"
        );
    }
}

// 5
#[test]
fn everything_is_down_oriented() {
    let scene = WorldScene::new();
    let t4 = &scene.red_black_identity().fourth_tower;
    assert!(!t4.added_cells().is_empty());
    for c in t4.added_cells() {
        assert_eq!(
            scene.world().get(c).unwrap().orientation(),
            Orientation::Down,
            "{c:?}"
        );
    }
    let l = scene.fortress_layout();
    // Finials crown the tower one course past its top, like the other three.
    for c in &t4.finials {
        assert_eq!(c.y, l.levels.max_y);
        assert!(
            scene.world().contains(IVec3::new(c.x, c.y + 1, c.z)),
            "{c:?} floats"
        );
    }
    for c in &t4.cap {
        assert_eq!(c.y, l.levels.tower_top_y);
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
    }
}

// 6
#[test]
fn the_route_into_the_tower_is_collision_safe() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    let g = l.levels.ground_y;
    let e = t4.entrance[0];
    for c in &t4.entrance {
        assert!(!scene.world().contains(*c), "{c:?} blocks the entrance");
    }
    let outside = IVec3::new(e.x - 1, g, e.z);
    assert!(l.courtyard.contains(&(outside.x, outside.z)));
    let (cx, cz) = t4.core;
    let pts = [
        eye(outside),
        eye(IVec3::new(e.x, g, e.z)),
        eye(IVec3::new(cx, g, cz)),
    ];
    walk_both_ways(&scene, &pts);
    for y in (l.levels.tower_top_y + 1)..=l.levels.base_y {
        assert!(
            !scene.world().contains(IVec3::new(cx, y, cz)),
            "the core is hollow"
        );
    }
    // From the gate across the courtyard to the tower.
    let gate_cell = eye(IVec3::new(l.gatehouse.max_x + 1, g, l.gate.base.z));
    let yard_x = l.central_keep.min_x - 1;
    walk_both_ways(
        &scene,
        &[
            gate_cell,
            eye(IVec3::new(yard_x, g, l.gate.base.z)),
            eye(IVec3::new(yard_x, g, outside.z)),
            eye(outside),
        ],
    );
}

// 7
#[test]
fn the_first_three_towers_are_unchanged() {
    let scene = WorldScene::new();
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        let t = scene.fortress_tower(f).unwrap();
        for c in t.shell.iter() {
            // Dark masonry, Crimson's wart, or a Gate 17 hall crest set into
            // a shared keep/tower cell.
            let bt = scene.world().get(*c).unwrap().block_type();
            assert!(
                is_dark_structure(bt) || bt == BlockType::NetherWartBlock || is_symbol(bt),
                "{c:?} {bt:?}"
            );
        }
        for c in &t.band {
            assert!(is_family_brick(scene.world().get(*c).unwrap().block_type()));
        }
        for (c, bt) in &t.crests {
            assert_eq!(scene.world().get(*c).unwrap().block_type(), *bt);
        }
        assert_eq!(t.finials.len(), 4);
    }
    // The seam: the keep's south wall is shared, not rebuilt.
    let t4 = &scene.red_black_identity().fourth_tower;
    let keep: Vec<IVec3> = scene.fortress_keep().cells();
    assert!(
        t4.seam.iter().any(|c| keep.contains(c)),
        "the tower leans on the keep wall"
    );
    for c in &t4.seam {
        assert!(!t4.shell.contains(c));
    }
}

// 8
#[test]
fn the_silhouette_is_balanced() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t4 = &scene.red_black_identity().fourth_tower;
    // The same top course and finial height as the family towers.
    let top_of = |cells: Vec<IVec3>| cells.iter().map(|c| c.y).min().unwrap();
    let t4_top = top_of(t4.cells());
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        assert_eq!(
            top_of(scene.fortress_tower(f).unwrap().cells()),
            t4_top,
            "{f:?}"
        );
    }
    assert_eq!(t4_top, l.levels.max_y);
    assert_eq!(t4.finials.len(), 4);
    // Mixed but within the three families: Crimson and Orange accents.
    let kinds: Vec<BlockType> = t4
        .band
        .iter()
        .map(|c| scene.world().get(*c).unwrap().block_type())
        .collect();
    assert!(
        kinds.contains(&BlockType::RedBlackDeepslateBricksCrimson)
            && kinds.contains(&BlockType::RedBlackDeepslateBricksOrange)
    );
    assert!(t4.crests.iter().all(|(_, t)| is_symbol(*t)));
    assert_eq!(t4.crests.len(), 2);
    // Dark majority in its own placed cells.
    let placed = t4.added_cells();
    let dark = placed
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(dark * 2 > placed.len(), "dark {dark} of {}", placed.len());
}
