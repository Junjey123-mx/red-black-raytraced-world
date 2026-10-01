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
    CameraCollisionConfig, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::light::Light;
use scene::overworld::HOUSE_FOOTPRINT;
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
fn lamps_are_present_and_restrained() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let d = scene.castle_details();
    assert!(
        d.lamps.len() >= 5 && d.lamps.len() <= 10,
        "{}",
        d.lamps.len()
    );
    for cell in &d.lamps {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::RedstoneLampLit);
        assert!(c.footprint.contains(cell.x, cell.z));
        assert!(is_camera_solid(b));
    }
    let on_gate = d
        .lamps
        .iter()
        .filter(|l| c.gatehouse.contains(l.x, l.z))
        .count();
    let on_keep = d.lamps.iter().filter(|l| c.is_keep_shell(l.x, l.z)).count();
    let on_posts = d
        .lamps
        .iter()
        .filter(|l| c.courtyard.contains(&(l.x, l.z)))
        .count();
    assert!(
        on_gate >= 2 && on_keep >= 2 && on_posts >= 1,
        "gate {on_gate} keep {on_keep} posts {on_posts}"
    );
    // Every lamp has a solid cell below or beside it: no floating lights.
    for cell in &d.lamps {
        let supported = [(0, -1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
            .iter()
            .any(|(dx, dy, dz)| {
                scene
                    .world()
                    .contains(IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz))
            });
        assert!(supported, "{cell:?}");
    }
    println!(
        "GATE16 details: lamps={} keep_merlons={} rails={} castle_voxels={} voxels={}",
        d.lamps.len(),
        d.keep_merlons.len(),
        d.rails.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_point_light_count_is_unchanged() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    let points = lights
        .iter()
        .filter(|l| !matches!(l, Light::Directional(_)))
        .count();
    assert_eq!(points, 4);
    let castle = std::fs::read_to_string("src/scene/castle.rs").unwrap();
    assert!(!castle.contains("PointLight") && !castle.contains("Light::"));
    let world = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(world.matches("PointLight::new(").count(), 0);
}

#[test]
fn the_courtyard_route_stays_clear() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = c.levels.ground_y;
    let route: Vec<Vec3> = c.interior_route.iter().map(|r| eye(*r)).collect();
    walk_both_ways(&scene, &route);
    // Details never stand on a route column.
    for cell in scene.castle_details().cells() {
        assert!(
            !c.interior_route
                .iter()
                .any(|r| r.x == cell.x && r.z == cell.z && cell.y <= r.y + 2),
            "{cell:?} on the route"
        );
    }
    // The gate row and the row to the keep door stay open across the yard.
    for x in (c.gatehouse.max_x + 1)..c.keep.min_x {
        for z in [c.gate.base.z, c.keep_door.base.z] {
            for dy in 1..=2 {
                assert!(
                    !scene.world().contains(IVec3::new(x, g + dy, z)),
                    "({x},{z}) + {dy}"
                );
            }
        }
    }
    // Merlons and rails rest on something.
    let d = scene.castle_details();
    for cell in d.keep_merlons.iter().chain(d.rails.iter()) {
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?} floats"
        );
    }
    assert!(d.keep_merlons.len() >= 4, "{}", d.keep_merlons.len());
    for m in &d.keep_merlons {
        assert_eq!(m.y, c.levels.keep_roof_y + 1);
        assert!(c.is_keep_shell(m.x, m.z));
    }
}

#[test]
fn the_details_are_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.castle_details(), b.castle_details());
    assert_eq!(a.castle_furniture(), b.castle_furniture());
    assert_eq!(a.castle_cells().len(), b.castle_cells().len());
    assert_eq!(a.world().len(), b.world().len());
    for cell in a.castle_cells() {
        assert_eq!(a.world().get(cell), b.world().get(cell));
    }
}

#[test]
fn the_house_remains_a_separate_landmark() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let house = HOUSE_FOOTPRINT.grown(3);
    for cell in scene.castle_cells() {
        assert!(c.footprint.contains(cell.x, cell.z), "{cell:?} off the pad");
        assert!(!house.contains(cell.x, cell.z));
    }
    assert!(
        c.footprint.min_x - HOUSE_FOOTPRINT.max_x >= 10,
        "the castle crowds the house"
    );
    // The house door is still reachable from the path and passable.
    let door = &scene.house_exterior().door;
    for d in door {
        assert!(is_camera_passable(
            scene.world().get(*d).unwrap().block_type()
        ));
    }
    let lower = door[0];
    let start = scene.path().start().unwrap();
    let points = [
        eye(start),
        v(
            lower.x as f32 + 0.5,
            lower.y as f32 + 0.5,
            lower.z as f32 + 1.5,
        ),
        v(
            lower.x as f32 + 0.5,
            lower.y as f32 + 0.5,
            lower.z as f32 + 0.5,
        ),
        v(
            lower.x as f32 + 0.5,
            lower.y as f32 + 0.5,
            lower.z as f32 - 1.5,
        ),
    ];
    walk_both_ways(&scene, &points);
    // The house keeps its own lamps, glass, fence and roof.
    let h = scene.house_exterior();
    assert_eq!(h.lamps.len(), 2);
    for cell in &h.lamps {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::RedstoneLampLit
        );
    }
    for cell in h.glass.iter() {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Glass
        );
    }
    for (cell, t, _) in &h.roof {
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *t);
    }
}
