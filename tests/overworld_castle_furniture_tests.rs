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
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::castle::seat_faces;
use scene::orientation::Orientation;
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

fn eye(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 + 2.0, c.z as f32 + 0.5)
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

#[test]
fn chairs_are_wood_stairs_facing_a_table() {
    let scene = WorldScene::new();
    let f = scene.castle_furniture();
    let c = scene.castle_layout();
    assert!(
        f.chairs.len() >= 4 && f.chairs.len() <= 8,
        "{}",
        f.chairs.len()
    );
    let tops: Vec<IVec3> = f.tables.iter().chain(f.benches.iter()).copied().collect();
    for (cell, o) in &f.chairs {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.orientation(), *o);
        assert_ne!(*o, Orientation::Up);
        assert!(c.hall.contains(cell.x, cell.z));
        let (dx, dz) = seat_faces(*o);
        let facing = IVec3::new(cell.x + dx, cell.y, cell.z + dz);
        assert!(
            tops.contains(&facing),
            "{cell:?} faces {facing:?}, not a table"
        );
        // Standing on a floor (planks) or the upper deck.
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
    }
    let ground = f
        .chairs
        .iter()
        .filter(|(cell, _)| cell.y == c.levels.base_y)
        .count();
    assert!(ground >= 4, "{ground} chairs around the hall table");
    println!(
        "GATE16 furniture: chairs={} tables={} benches={} castle_voxels={} voxels={}",
        f.chairs.len(),
        f.tables.len(),
        f.benches.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn tables_and_benches_use_existing_blocks() {
    let scene = WorldScene::new();
    let f = scene.castle_furniture();
    assert!(f.tables.len() >= 2 && !f.benches.is_empty());
    for cell in f.tables.iter().chain(f.benches.iter()) {
        let b = scene.world().get(*cell).unwrap();
        assert!(
            matches!(
                b.block_type(),
                BlockType::DoubleWoodSlab | BlockType::WoodPlanks
            ),
            "{cell:?}"
        );
        assert!(is_camera_solid(b));
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
    }
    // Every furniture block is one of the certified catalog types.
    for cell in f.cells() {
        let t = scene.world().get(cell).unwrap().block_type();
        assert!(matches!(
            t,
            BlockType::WoodStairs
                | BlockType::DoubleWoodSlab
                | BlockType::WoodPlanks
                | BlockType::Fence
        ));
    }
    let src = std::fs::read_to_string("src/scene/block_type.rs").unwrap();
    assert!(
        !src.contains("Chair") && !src.contains("Table"),
        "a furniture BlockType was added"
    );
}

#[test]
fn the_walkway_remains_open() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = c.levels.ground_y;
    // The corridor row inside the door, wall to wall, two cells high.
    for x in c.hall.min_x..=c.hall.max_x {
        for dy in 1..=2 {
            let cell = IVec3::new(x, g + dy, c.hall.min_z);
            assert!(
                !scene.world().contains(cell),
                "{cell:?} blocks the corridor"
            );
        }
    }
    let row: Vec<Vec3> = (c.hall.min_x..=c.hall.max_x)
        .map(|x| eye(IVec3::new(x, g, c.hall.min_z)))
        .collect();
    walk_both_ways(&scene, &row);
    // The whole canonical route, both ways.
    let route: Vec<Vec3> = c.interior_route.iter().map(|r| eye(*r)).collect();
    walk_both_ways(&scene, &route);
}

#[test]
fn the_door_and_the_stairs_are_not_obstructed() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = scene.castle_furniture();
    let d = c.keep_door.base;
    for dy in 0..2 {
        let inside = IVec3::new(d.x + 1, d.y + dy, d.z);
        let outside = IVec3::new(d.x - 1, d.y + dy, d.z);
        assert!(!scene.world().contains(inside) && !scene.world().contains(outside));
    }
    let s = scene.castle_stairs();
    for (tread, _) in &s.treads {
        for dy in 1..=2 {
            assert!(
                !scene
                    .world()
                    .contains(IVec3::new(tread.x, tread.y + dy, tread.z))
            );
        }
        assert!(
            !f.cells().iter().any(|m| m.x == tread.x && m.z == tread.z),
            "furniture in the stair column"
        );
    }
    for dy in 1..=2 {
        assert!(
            !scene
                .world()
                .contains(IVec3::new(s.landing.x, s.landing.y + dy, s.landing.z))
        );
    }
    assert!(clear(&scene, eye(s.landing)));
}

#[test]
fn windows_are_not_blocked_by_furniture() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    for g in &scene.castle_windows().glass {
        if !c.is_keep_shell(g.x, g.z) {
            continue;
        }
        // The cell just inside the window, at the same height, is open.
        let (dx, dz) = if g.x == c.keep.min_x {
            (1, 0)
        } else if g.x == c.keep.max_x {
            (-1, 0)
        } else if g.z == c.keep.min_z {
            (0, 1)
        } else {
            (0, -1)
        };
        let inside = IVec3::new(g.x + dx, g.y, g.z + dz);
        assert!(!scene.world().contains(inside), "{inside:?} blocks {g:?}");
    }
}
