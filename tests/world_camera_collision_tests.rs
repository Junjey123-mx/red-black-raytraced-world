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
    CAMERA_COLLISION_RADIUS, COLLISION_SUBSTEP, CameraCollisionConfig, CameraCollisionVolume,
    is_camera_solid, is_position_clear, resolve_camera_motion,
};
use camera::world_free_fly::{FreeFlyInput, WorldFreeFlyCameraState, apply_free_fly_input};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::world::{WorldScene, world_free_fly_camera};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn near(a: Vec3, b: Vec3, eps: f32) -> bool {
    (a - b).length() <= eps
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
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

/// The open cutaway air in front of the portal frame's top row (frame at
/// y = -5, z = 14): a solid wall to push against. The membrane below it is
/// camera-passable (see the passability contract), so it is not a wall.
fn portal_approach() -> Vec3 {
    v(16.5, -4.5, 16.5)
}

/// Front face of the frame row, plus the radius: where the camera stops.
const WALL_STOP: f32 = 15.0 + CAMERA_COLLISION_RADIUS;

#[test]
fn a_position_in_open_air_is_clear() {
    let scene = WorldScene::new();
    assert!(clear(&scene, world_free_fly_camera().position));
    assert!(clear(&scene, portal_approach()));
    assert!(clear(&scene, v(12.0, 30.0, 12.0)));
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert!(COLLISION_SUBSTEP <= CAMERA_COLLISION_RADIUS);
}

#[test]
fn a_position_inside_a_full_cube_is_blocked() {
    let scene = WorldScene::new();
    // Deep in the lower mass behind the portal (deepslate).
    let cell = IVec3::new(16, -12, 10);
    assert_eq!(
        scene.world().get(cell).map(|b| b.block_type()),
        Some(BlockType::Deepslate)
    );
    assert!(!clear(&scene, v(16.5, -11.5, 10.5)));
    // Just outside a cube face, further than the radius: clear only if the
    // neighbouring cell is air (here it is not: solid all around).
    assert!(!clear(&scene, v(16.9, -11.5, 10.5)));
    // The cobblestone path in front of the house door: the eye floats a
    // full cell above it, but not inside it.
    let path_start = scene.path().cells[0];
    assert_eq!(path_start, IVec3::new(8, 5, 13));
    assert!(clear(&scene, v(8.5, 6.5, 13.5)));
    assert!(!clear(&scene, v(8.5, 5.5, 13.5)));
    assert!(is_camera_solid(scene.world().get(cell).unwrap()));
}

#[test]
fn a_wall_stops_forward_movement() {
    let scene = WorldScene::new();
    let from = portal_approach();
    // Straight at the membrane (solid in this stage): stops just in front.
    let to = resolve(&scene, from, v(0.0, 0.0, -3.0));
    assert!(
        to.z >= 14.5625 + CAMERA_COLLISION_RADIUS - 1e-3,
        "went through: {to:?}"
    );
    assert!(to.z < from.z, "did not advance at all: {to:?}");
    assert!(clear(&scene, to));
    assert!((to.x - from.x).abs() < 1e-6 && (to.y - from.y).abs() < 1e-6);
}

#[test]
fn movement_parallel_to_a_wall_keeps_its_full_length() {
    let scene = WorldScene::new();
    // Hugging the frame row, moving east along it.
    let from = v(15.5, -4.5, WALL_STOP + 0.05);
    assert!(clear(&scene, from));
    let to = resolve(&scene, from, v(1.0, 0.0, 0.0));
    assert!(near(to, from + v(1.0, 0.0, 0.0), 1e-4), "{to:?}");
}

#[test]
fn a_diagonal_push_into_a_wall_slides_along_it() {
    let scene = WorldScene::new();
    let from = v(15.5, -4.5, 16.0);
    // North-east: the north component hits the frame, the east one slides.
    let to = resolve(&scene, from, v(1.0, 0.0, -1.0));
    assert!(to.x > from.x + 0.9, "east component lost: {to:?}");
    assert!(to.z >= WALL_STOP - 1e-3, "went through: {to:?}");
    assert!(to.z < from.z, "did not close the gap: {to:?}");
    assert!(clear(&scene, to));
}

#[test]
fn substeps_prevent_tunneling_through_thin_geometry() {
    let scene = WorldScene::new();
    let from = portal_approach();
    // One huge step (a very slow frame): the membrane is only 1/8 thick
    // and the frame wall one cell, but the camera still stops in front.
    for delta in [v(0.0, 0.0, -2.5), v(0.0, 0.0, -10.0), v(0.3, 0.0, -6.0)] {
        let to = resolve(&scene, from, delta);
        assert!(
            to.z >= 14.5625 + CAMERA_COLLISION_RADIUS - 1e-3,
            "{delta:?} -> {to:?}"
        );
        assert!(clear(&scene, to));
    }
    // The free-fly state itself, moved through the resolver frame by frame.
    let mut state = WorldFreeFlyCameraState::looking_at(from, v(16.5, -4.5, 14.5));
    for _ in 0..60 {
        let previous = state.position;
        apply_free_fly_input(
            &mut state,
            &FreeFlyInput {
                move_forward: 1.0,
                delta_time: 0.25,
                ..Default::default()
            },
        );
        state.position = resolve(&scene, previous, state.position - previous);
    }
    assert!(state.position.z >= WALL_STOP - 1e-3, "{:?}", state.position);
}

#[test]
fn negative_coordinates_resolve_like_positive_ones() {
    let scene = WorldScene::new();
    // Under the shelf in the lower cutaway (y < -10): air with the shelf above.
    let from = v(17.5, -12.5, 18.5);
    assert!(clear(&scene, from));
    // Up into the shelf (y = -10 cells): blocked at its underside.
    let to = resolve(&scene, from, v(0.0, 3.0, 0.0));
    assert!(to.y <= -10.0 - CAMERA_COLLISION_RADIUS + 1e-3, "{to:?}");
    assert!(to.y > from.y);
    // Cell mapping of the sphere for negative centers.
    let cells: Vec<IVec3> = CameraCollisionVolume::new(v(-0.1, -0.1, -0.1), 0.25)
        .overlapped_cells()
        .collect();
    assert!(cells.contains(&IVec3::new(-1, -1, -1)) && cells.contains(&IVec3::new(0, 0, 0)));
    assert_eq!(cells.len(), 8);
}

#[test]
fn partial_geometry_is_tested_against_its_real_shape() {
    let scene = WorldScene::new();
    // The porch fence: thin rails and a post inside a mostly empty cell.
    // The empty part of the cell is free, the rails are solid.
    let fence = IVec3::new(6, 6, 13);
    assert_eq!(
        scene.world().get(fence).map(|b| b.block_type()),
        Some(BlockType::Fence)
    );
    // Rails span z 0.4375..0.5625 of the cell (the post reaches 0.625):
    // beside the post, 0.29 past the rails is clear, 0.19 is not.
    assert!(clear(&scene, v(6.9, 6.5, 13.5625 + 0.29)));
    assert!(!clear(&scene, v(6.9, 6.5, 13.5625 + 0.19)));
    // The camera-passable membrane is not a collider even though it is a
    // thin slab in its cell.
    assert!(clear(&scene, v(16.5, -7.5, 14.5)));
    // A wooden stair of the upper descent: its raised half is solid, the
    // open half above the low step is free.
    let (tread, orientation) = scene.upper_descent().treads[2];
    assert_eq!(orientation, scene::orientation::Orientation::South);
    // South stairs: low half in the south (z 0.5..1), raised half north.
    let low_half = v(
        tread.x as f32 + 0.5,
        tread.y as f32 + 0.8,
        tread.z as f32 + 0.75,
    );
    let raised_half = v(
        tread.x as f32 + 0.5,
        tread.y as f32 + 0.8,
        tread.z as f32 + 0.25,
    );
    assert!(clear(&scene, low_half));
    assert!(!clear(&scene, raised_half));
}

#[test]
fn the_hot_path_queries_only_nearby_cells() {
    let volume = CameraCollisionVolume::new(v(16.5, -7.5, 16.5), CAMERA_COLLISION_RADIUS);
    assert!(volume.overlapped_cells().count() <= 8);
    let src = std::fs::read_to_string("src/camera/world_collision.rs").unwrap();
    assert!(!src.contains(".iter()") || !src.contains("world.iter()"));
    assert!(!src.contains("world.iter("));
    assert!(src.contains("overlapped_cells()"));
    // The app resolves once per update, in the World only.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert_eq!(app.matches("resolve_camera_motion(").count(), 1);
    let world_impl = &app[app.find("impl ViewerMode for WorldMode").unwrap()..];
    assert!(world_impl.contains("resolve_camera_motion("));
}

#[test]
fn the_reset_pose_is_clear() {
    let scene = WorldScene::new();
    let mut state = world_free_fly_camera();
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            move_forward: 1.0,
            delta_time: 0.25,
            ..Default::default()
        },
    );
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert!(clear(&scene, state.position));
    assert_eq!(state.position, world_free_fly_camera().position);
}

#[test]
fn the_catalog_viewer_has_no_collision() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog_impl = &app[app.find("impl ViewerMode for CatalogMode").unwrap()
        ..app
            .find("/// The main world: the Overworld `WorldScene`.")
            .unwrap()];
    assert!(!catalog_impl.contains("collision") && !catalog_impl.contains("resolve_camera_motion"));
    let catalog_struct =
        &app[app.find("struct CatalogMode {").unwrap()..app.find("impl CatalogMode").unwrap()];
    assert!(!catalog_struct.contains("collision"));
    assert!(
        app.contains("\"Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset\"")
    );
}
