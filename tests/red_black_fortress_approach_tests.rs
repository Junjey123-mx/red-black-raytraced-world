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

use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::expansion::{fortress_approach_columns, lower_ground_cell};
use scene::orientation::Orientation;
use scene::world::WorldScene;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(
        scene.world(),
        from,
        delta,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
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

/// The inverted viewer's eye under a ground cell.
fn under(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 - 1.5, c.z as f32 + 0.5)
}

/// From the exit clearance behind the membrane, down the inverted flight,
/// over the landing and along the new path to the pad center.
fn route(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(under(*t));
    }
    let l = scene.inverted_route().layout;
    points.push(under(IVec3::new(l.x, l.platform_y, l.platform_z_max)));
    let e = scene.expansion_layout();
    points.push(under(e.red_black_path_anchor));
    for c in &scene.red_black_approach().main {
        points.push(under(*c));
    }
    let pad = e.red_black_fortress_pad;
    let y = e.fortress_pad_bottom_y as f32 - 1.5;
    points.push(v(
        pad.min_x as f32 + 0.5,
        y,
        e.red_black_path_target.z as f32 + 0.5,
    ));
    // Gate 17 built the fortress here: the walk ends in its courtyard,
    // just past the gate corridor, instead of the built-over pad centre.
    let fortress = scene.fortress_layout();
    points.push(v(
        fortress.gatehouse.max_x as f32 + 1.5,
        y,
        e.red_black_path_target.z as f32 + 0.5,
    ));
    points
}

#[test]
fn the_landing_is_connected_to_the_fortress_pad() {
    let scene = WorldScene::new();
    let a = scene.red_black_approach();
    let l = scene.expansion_layout();
    assert!(a.main.len() >= 20, "{}", a.main.len());
    let first = a.main[0];
    let anchor = l.red_black_path_anchor;
    assert!(
        (first.x - anchor.x).abs() <= 1
            && (first.z - anchor.z).abs() <= 1
            && (first.y - anchor.y).abs() <= 1,
        "{first:?} vs {anchor:?}"
    );
    for (i, c) in a.main.iter().enumerate() {
        let t = scene.world().get(*c).unwrap().block_type();
        assert!(
            matches!(
                t,
                BlockType::PolishedBlackstoneBricks
                    | BlockType::SmoothBasalt
                    | BlockType::RedBlackDeepslateBricksCrimson
                    | BlockType::RedBlackDeepslateBricksOrange
                    | BlockType::RedBlackDeepslateBricksViolet
            ),
            "{c:?} is {t:?}"
        );
        assert_eq!(
            lower_ground_cell(scene.world(), c.x, c.z, -30, -10),
            Some(*c)
        );
        let connected = a.main.iter().enumerate().any(|(j, o)| {
            j != i && (o.x - c.x).abs() <= 1 && (o.z - c.z).abs() <= 1 && (o.y - c.y).abs() <= 1
        });
        assert!(connected, "{c:?} is isolated");
    }
    let last = *a.main.last().unwrap();
    assert!(last.x + 1 == l.red_black_fortress_pad.min_x);
    assert_eq!(a.accents.len(), 2);
    println!(
        "GATE15 lower approach: main={} accents={:?} voxels={}",
        a.main.len(),
        a.accents,
        scene.world().len()
    );
}

#[test]
fn the_canonical_portal_path_is_unchanged() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    assert_eq!(route.treads.len(), 10);
    for (cell, o) in &route.treads {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::WoodStairs)
        );
        assert_eq!(*o, Orientation::Down);
    }
    for cell in route.layout.platform_cells() {
        assert_eq!(
            scene.world().get(cell).map(|b| b.block_type()),
            Some(BlockType::DeepslateBricks)
        );
    }
    for cell in route.layout.exit_clearance() {
        assert!(!scene.world().contains(cell));
    }
    let a = scene.red_black_approach();
    for c in &a.main {
        assert!(
            !route.treads.iter().any(|(t, _)| t == c)
                && !route.layout.platform_cells().contains(c)
                && !route.supports.contains(c)
        );
    }
}

#[test]
fn no_noclip_is_needed_from_the_portal_to_the_pad() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    walk(&scene, &route(&scene));
}

#[test]
fn the_path_is_two_wide_where_practical() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let cols: Vec<(i32, i32)> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| (c.x, c.z))
        .collect();
    assert_eq!(cols.len(), fortress_approach_columns(l).len());
    let anchor = l.red_black_path_anchor;
    for x in anchor.x..l.red_black_fortress_pad.min_x {
        assert!(
            cols.contains(&(x, anchor.z + 1)) && cols.contains(&(x, anchor.z + 2)),
            "x = {x}"
        );
    }
    for z in (anchor.z + 3)..=(l.red_black_path_target.z + 1) {
        assert!(
            cols.contains(&(l.red_black_fortress_pad.min_x - 1, z))
                && cols.contains(&(l.red_black_fortress_pad.min_x - 2, z)),
            "z = {z}"
        );
    }
}

#[test]
fn no_focal_symbol_is_destroyed() {
    let scene = WorldScene::new();
    let counts: Vec<(usize, usize)> = scene
        .families()
        .iter()
        .map(|f| (f.symbols.len(), f.accents.len()))
        .collect();
    assert_eq!(counts, vec![(25, 12), (21, 6), (23, 10)]);
    for family in scene.families() {
        for cell in family.cells() {
            assert!(scene.world().contains(cell));
            assert!(
                !scene.red_black_approach().main.contains(&cell),
                "{cell:?} of {:?} paved",
                family.family
            );
        }
    }
    for cell in &scene.lower_surface().cells {
        assert!(scene.world().contains(*cell));
    }
}

#[test]
fn the_lower_route_works_in_reverse() {
    let scene = WorldScene::new();
    let mut points = route(&scene);
    points.reverse();
    walk(&scene, &points);
}

#[test]
fn the_paving_keeps_down_semantics() {
    let scene = WorldScene::new();
    let a = scene.red_black_approach();
    for c in a.main.iter().chain(a.accents.iter()) {
        let b = scene.world().get(*c).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        // Ground: nothing below (world -Y), the viewer's room is clear.
        assert!(!scene.world().contains(IVec3::new(c.x, c.y - 1, c.z)));
        assert!(clear(&scene, under(*c)), "{c:?}");
    }
    assert!(a.accents.iter().all(|c| a.main.contains(c)));
}
