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
use scene::castle::{GATE_WIDTH, PASSAGE_HEIGHT};
use scene::orientation::Orientation;
use scene::overworld_blocks::{wood_door_bottom_material_id, wood_door_top_material_id};
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

/// Eye points from the approach, through the gate, into the courtyard.
fn entrance(scene: &WorldScene) -> Vec<Vec3> {
    let c = scene.castle_layout();
    let end = c.keep.min_x;
    c.interior_route
        .iter()
        .take_while(|r| r.x < end)
        .map(|r| eye(*r))
        .collect()
}

/// Columns where a later stage (C193) stands a fence post, rail or lamp.
fn detailed(scene: &WorldScene, x: i32, z: i32) -> bool {
    scene
        .castle_details()
        .cells()
        .iter()
        .any(|c| c.x == x && c.z == z)
}

#[test]
fn the_door_is_visible_in_the_gate() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = scene.castle_gatehouse();
    assert_eq!(g.door.len(), (GATE_WIDTH * PASSAGE_HEIGHT) as usize);
    let mut cells = g.door.clone();
    cells.sort_by_key(|c| (c.z, c.y));
    let mut expected = c.gate.cells();
    expected.sort_by_key(|c| (c.z, c.y));
    assert_eq!(cells, expected);
    for cell in &g.door {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodDoor);
        assert_eq!(b.orientation(), Orientation::West);
        let expected = if cell.y == c.gate.base.y {
            wood_door_bottom_material_id()
        } else {
            wood_door_top_material_id()
        };
        assert_eq!(b.material_id(), expected);
    }
    // Framed: masonry above and beside, roof over the passage.
    let lintel = IVec3::new(c.gate.base.x, c.gate.base.y + PASSAGE_HEIGHT, c.gate.base.z);
    assert!(scene.world().contains(lintel));
    assert_eq!(
        g.roof.len(),
        (c.gatehouse.width() * c.gatehouse.depth()) as usize
    );
    for r in &g.roof {
        assert_eq!(
            scene.world().get(*r).unwrap().block_type(),
            BlockType::WoodPlanks
        );
        assert_eq!(r.y, c.levels.wall_top_y + 1);
    }
    println!(
        "GATE16 gatehouse: posts={} walls={} roof={} door={} rail={} castle_voxels={} voxels={}",
        g.posts.len(),
        g.walls.len(),
        g.roof.len(),
        g.door.len(),
        g.rail.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_door_is_passable_and_the_walls_solid() {
    let scene = WorldScene::new();
    let g = scene.castle_gatehouse();
    for cell in &g.door {
        let b = scene.world().get(*cell).unwrap();
        assert!(is_camera_passable(b.block_type()));
        assert!(!is_camera_solid(b));
    }
    for cell in g
        .posts
        .iter()
        .chain(g.walls.iter())
        .chain(g.roof.iter())
        .chain(g.rail.iter())
    {
        let b = scene.world().get(*cell).unwrap();
        assert!(is_camera_solid(b), "{cell:?}");
    }
    for cell in &g.posts {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Log
        );
    }
    for cell in &g.rail {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Fence
        );
    }
    // The eye inside the door cell is clear; one cell into the jamb it is not.
    let c = scene.castle_layout();
    let d = c.gate.base;
    assert!(clear(
        &scene,
        v(d.x as f32 + 0.5, d.y as f32 + 1.0, d.z as f32 + 1.0)
    ));
    assert!(!clear(
        &scene,
        v(d.x as f32 + 0.5, d.y as f32 + 1.0, d.z as f32 - 0.5)
    ));
    assert!(!clear(
        &scene,
        v(
            d.x as f32 + 0.5,
            d.y as f32 + 1.0,
            d.z as f32 + GATE_WIDTH as f32 + 0.5
        )
    ));
}

#[test]
fn the_approach_continues_through_the_gate() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let mut points: Vec<Vec3> = scene
        .overworld_approach()
        .main
        .iter()
        .map(|c| eye(*c))
        .collect();
    points.extend(entrance(&scene));
    walk_both_ways(&scene, &points);
}

#[test]
fn the_courtyard_is_reachable_without_noclip() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let mut points = entrance(&scene);
    let g = c.levels.ground_y;
    // Every courtyard column, from the gate.
    let last = *points.last().unwrap();
    for (x, z) in &c.courtyard {
        if detailed(&scene, *x, *z) {
            continue;
        }
        let target = v(*x as f32 + 0.5, g as f32 + 2.0, *z as f32 + 0.5);
        assert!(clear(&scene, target), "({x},{z})");
        let mut route = points.clone();
        route.push(target);
        let _ = last;
        walk(&scene, &route);
    }
    points.push(eye(IVec3::new(c.courtyard[0].0, g, c.courtyard[0].1)));
    walk_both_ways(&scene, &points);
}

#[test]
fn the_gatehouse_stays_within_its_block() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = scene.castle_gatehouse();
    for cell in g.cells() {
        assert!(c.gatehouse.contains(cell.x, cell.z), "{cell:?}");
        assert!(cell.y > c.levels.ground_y && cell.y <= c.levels.wall_top_y + 2);
    }
    // The passage keeps its head room (two cells) under lintel and roof.
    for (x, z) in c.gatehouse_passage() {
        for dy in 1..=2 {
            let cell = IVec3::new(x, c.levels.ground_y + dy, z);
            let open = !scene.world().contains(cell) || g.door.contains(&cell);
            assert!(open, "{cell:?}");
        }
    }
}
