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
use scene::expansion::{approach_columns, ground_cell};
use scene::overworld::HOUSE_FOOTPRINT;
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

/// Moves from `from` toward `target` with collision on: straight, or, when
/// the straight line cuts a corner, the way a walker turns it (one axis
/// first, then the rest).
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

/// Eye above a paved cell.
fn over(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 + 2.0, c.z as f32 + 0.5)
}

/// The route from the porch to the pad center as eye positions.
fn route(scene: &WorldScene) -> Vec<Vec3> {
    let l = scene.expansion_layout();
    let mut points: Vec<Vec3> = scene
        .path()
        .cells
        .iter()
        .take_while(|c| **c != l.overworld_path_anchor)
        .map(|c| over(*c))
        .collect();
    points.push(over(l.overworld_path_anchor));
    for c in &scene.overworld_approach().main {
        points.push(over(*c));
    }
    let pad = l.overworld_castle_pad;
    let y = l.castle_pad_surface_y as f32 + 2.0;
    points.push(v(
        pad.min_x as f32 + 0.5,
        y,
        l.overworld_path_target.z as f32 + 0.5,
    ));
    points.push(v(
        (pad.min_x + pad.max_x) as f32 / 2.0 + 0.5,
        y,
        (pad.min_z + pad.max_z) as f32 / 2.0 + 0.5,
    ));
    points
}

#[test]
fn the_path_connects_the_old_area_to_the_castle_pad() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let a = scene.overworld_approach();
    assert!(a.main.len() >= 30, "{}", a.main.len());
    let first = a.main[0];
    let anchor = l.overworld_path_anchor;
    assert!(
        (first.x - anchor.x).abs() <= 1
            && (first.z - anchor.z).abs() <= 1
            && (first.y - anchor.y).abs() <= 1,
        "{first:?} vs {anchor:?}"
    );
    for (i, c) in a.main.iter().enumerate() {
        let t = scene.world().get(*c).unwrap().block_type();
        assert!(
            matches!(t, BlockType::Cobblestone | BlockType::Stone),
            "{c:?} is {t:?}"
        );
        assert_eq!(ground_cell(scene.world(), c.x, c.z, 40, -10), Some(*c));
        let connected = a.main.iter().enumerate().any(|(j, o)| {
            j != i && (o.x - c.x).abs() <= 1 && (o.z - c.z).abs() <= 1 && (o.y - c.y).abs() <= 1
        });
        assert!(connected, "{c:?} is an isolated paving stone");
    }
    let last = *a.main.last().unwrap();
    assert!(
        last.x + 1 == l.overworld_castle_pad.min_x
            || l.overworld_castle_pad.contains(last.x + 1, last.z)
    );
    assert!(!a.forecourt.is_empty());
    for c in &a.forecourt {
        assert!(
            !l.overworld_castle_pad.contains(c.x, c.z),
            "forecourt inside the pad at {c:?}"
        );
        assert_eq!(
            scene.world().get(*c).map(|b| b.block_type()),
            Some(BlockType::Cobblestone)
        );
    }
    println!(
        "GATE15 approach: main={} forecourt={} accents={:?} voxels={}",
        a.main.len(),
        a.forecourt.len(),
        a.accents,
        scene.world().len()
    );
}

#[test]
fn the_route_is_continuous_with_collision_on() {
    let scene = WorldScene::new();
    walk(&scene, &route(&scene));
}

#[test]
fn the_main_segments_are_at_least_two_wide() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let cols: Vec<(i32, i32)> = scene
        .overworld_approach()
        .main
        .iter()
        .map(|c| (c.x, c.z))
        .collect();
    let corridor = approach_columns(l);
    assert_eq!(
        cols.len(),
        corridor.len(),
        "some corridor column was not paved"
    );
    let anchor = l.overworld_path_anchor;
    for x in (anchor.x + 1)..l.overworld_castle_pad.min_x {
        assert!(
            cols.contains(&(x, anchor.z)) && cols.contains(&(x, anchor.z - 1)),
            "x = {x}"
        );
    }
    for z in (l.overworld_path_target.z - 1)..(anchor.z - 1) {
        assert!(
            cols.contains(&(l.overworld_castle_pad.min_x - 1, z))
                && cols.contains(&(l.overworld_castle_pad.min_x - 2, z)),
            "z = {z}"
        );
    }
}

#[test]
fn the_pond_is_not_blocked() {
    let scene = WorldScene::new();
    assert_eq!(scene.pond().water.len(), 26);
    for c in &scene.pond().water {
        assert_eq!(
            scene.world().get(*c).map(|b| b.block_type()),
            Some(BlockType::Water)
        );
    }
    let a = scene.overworld_approach();
    for c in a.cells().iter().chain(a.accents.iter()) {
        assert!(!scene::overworld::pond_basin_columns().contains(&(c.x, c.z)));
        assert!(c.z <= 14, "{c:?} strays south into the pond/cutaway rows");
    }
}

#[test]
fn the_house_is_not_blocked() {
    let scene = WorldScene::new();
    let a = scene.overworld_approach();
    for c in a.cells().iter().chain(a.accents.iter()) {
        assert!(!HOUSE_FOOTPRINT.grown(1).contains(c.x, c.z), "{c:?}");
    }
    let inside = resolve(&scene, v(8.5, 7.5, 13.5), v(0.0, 0.0, -3.0));
    assert!(inside.z < 12.0);
    for c in &scene.path().cells {
        assert_eq!(
            scene.world().get(*c).map(|b| b.block_type()),
            Some(BlockType::Cobblestone)
        );
    }
}

#[test]
fn the_portal_is_not_blocked() {
    let scene = WorldScene::new();
    assert_eq!(
        (scene.portal().frame.len(), scene.portal().core.len()),
        (18, 12)
    );
    let a = scene.overworld_approach();
    for c in a.cells().iter().chain(a.accents.iter()) {
        assert!(!scene.portal().frame.contains(c) && !scene.portal().core.contains(c));
        assert!(!scene.upper_descent().treads.iter().any(|(t, _)| t == c));
        assert!(!scene.rhombus().cutaway.contains_column(c.x, c.z));
    }
    let to = resolve(&scene, v(16.5, -7.5, 16.5), v(0.0, 0.0, -3.0));
    assert!(to.z < 14.0, "{to:?}");
    assert!(is_position_clear(
        scene.world(),
        v(16.5, -7.5, 13.5),
        &CameraCollisionConfig::default(),
        &is_camera_solid
    ));
}

#[test]
fn the_route_works_in_reverse() {
    let scene = WorldScene::new();
    let mut points = route(&scene);
    points.reverse();
    walk(&scene, &points);
}
