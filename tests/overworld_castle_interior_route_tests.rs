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

fn reaches(scene: &WorldScene, points: &[Vec3]) -> bool {
    let mut position = points[0];
    for target in points.iter().skip(1) {
        position = step(scene, position, *target);
        if (position - *target).length() >= 1e-3 {
            return false;
        }
    }
    true
}

fn walk_both_ways(scene: &WorldScene, points: &[Vec3]) {
    walk(scene, points);
    let mut back = points.to_vec();
    back.reverse();
    walk(scene, &back);
}

/// The canonical castle route as eye points: forecourt, gate, courtyard,
/// keep door, hall, stair, upper landing, upper room.
fn castle_route(scene: &WorldScene) -> Vec<Vec3> {
    scene
        .castle_layout()
        .interior_route
        .iter()
        .map(|c| eye(*c))
        .collect()
}

#[test]
fn the_stairs_are_contiguous_wood_stairs() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let s = scene.castle_stairs();
    assert_eq!(s.treads.len(), c.stair.len());
    assert!(s.treads.len() >= 2);
    for (i, (cell, o)) in s.treads.iter().enumerate() {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.orientation(), *o);
        assert_eq!(
            *o,
            Orientation::North,
            "the climb runs south: raised half to the south"
        );
        assert!(c.hall.contains(cell.x, cell.z));
        if i > 0 {
            let (prev, _) = s.treads[i - 1];
            assert_eq!(cell.y, prev.y + 1, "one riser per tread");
            assert_eq!(
                (cell.x - prev.x).abs() + (cell.z - prev.z).abs(),
                1,
                "treads touch"
            );
        }
        // Resting on the floor or the tread below.
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
                || i > 0
        );
    }
    let (first, _) = s.treads[0];
    assert_eq!(first.y, c.levels.base_y);
    let (last, _) = *s.treads.last().unwrap();
    assert_eq!(last.y + 1, c.levels.upper_floor_y);
    assert_eq!(s.landing.y, c.levels.upper_floor_y);
    assert!(
        scene.castle_keep().upper_deck().contains(&s.landing),
        "{:?}",
        s.landing
    );
    println!(
        "GATE16 stairs: treads={} railing={} landing={:?} castle_voxels={} voxels={}",
        s.treads.len(),
        s.railing.len(),
        s.landing,
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn head_room_is_sufficient_over_every_tread() {
    let scene = WorldScene::new();
    let s = scene.castle_stairs();
    for (cell, _) in &s.treads {
        for dy in 1..=2 {
            let above = IVec3::new(cell.x, cell.y + dy, cell.z);
            assert!(!scene.world().contains(above), "{above:?} over a tread");
        }
        assert!(clear(&scene, eye(*cell)), "{cell:?}");
    }
    for dy in 1..=2 {
        assert!(
            !scene
                .world()
                .contains(IVec3::new(s.landing.x, s.landing.y + dy, s.landing.z))
        );
    }
    assert!(clear(&scene, eye(s.landing)));
    // The well is open from the hall floor to the room above.
    let c = scene.castle_layout();
    for (x, z) in &c.stair_well {
        for y in (c.levels.base_y + 2)..c.levels.keep_roof_y {
            let cell = IVec3::new(*x, y, *z);
            assert!(
                !scene.world().contains(cell) || s.treads.iter().any(|(t, _)| *t == cell),
                "{cell:?}"
            );
        }
    }
}

#[test]
fn the_interior_route_is_walkable_with_collision_on() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = castle_route(&scene);
    assert!(route.len() >= 12);
    walk(&scene, &route);
    // It ends on the upper floor, two above the deck.
    let end = *route.last().unwrap();
    assert_eq!(
        end.y,
        scene.castle_layout().levels.upper_floor_y as f32 + 2.0
    );
}

#[test]
fn the_interior_route_is_walkable_in_reverse() {
    let scene = WorldScene::new();
    let mut route = castle_route(&scene);
    route.reverse();
    walk(&scene, &route);
}

#[test]
fn railings_guard_the_well_without_blocking() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let s = scene.castle_stairs();
    assert!(!s.railing.is_empty());
    for cell in &s.railing {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Fence);
        assert!(is_camera_solid(b));
        assert_eq!(cell.y, c.levels.upper_floor_y + 1);
        assert!(c.hall.contains(cell.x, cell.z));
        assert!(
            !c.stair_well.contains(&(cell.x, cell.z)),
            "a rail in the well"
        );
        assert!(
            c.stair_well.contains(&(cell.x + 1, cell.z)),
            "the rail edges the well"
        );
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "rail floats"
        );
        assert!(
            !c.interior_route
                .iter()
                .any(|r| r.x == cell.x && r.z == cell.z && r.y + 1 == cell.y),
            "rail on the route"
        );
    }
    // Every upper-deck cell (the two carrying the rail excepted) is
    // reachable from the landing around the rail.
    let landing = eye(s.landing);
    // ... nor the ones carrying a fitting (a seat or a table).
    let railed = |cell: &IVec3| {
        scene
            .world()
            .contains(IVec3::new(cell.x, cell.y + 1, cell.z))
    };
    for cell in scene
        .castle_keep()
        .upper_deck()
        .into_iter()
        .filter(|c| !railed(c))
    {
        let target = eye(cell);
        assert!(clear(&scene, target), "{cell:?}");
        // Either along the landing row and up the west wall, or up the
        // well column to the corridor row and along it (free-fly: the open
        // well is no obstacle).
        let a = [
            landing,
            eye(IVec3::new(c.hall.min_x, cell.y, s.landing.z)),
            eye(IVec3::new(c.hall.min_x, cell.y, cell.z)),
            target,
        ];
        let b = [
            landing,
            eye(IVec3::new(s.landing.x, cell.y, c.hall.min_z)),
            eye(IVec3::new(cell.x, cell.y, c.hall.min_z)),
            target,
        ];
        assert!(
            reaches(&scene, &a) || reaches(&scene, &b),
            "{cell:?} unreachable"
        );
    }
}
