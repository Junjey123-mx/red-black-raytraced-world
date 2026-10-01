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

use camera::camera::Camera;
use camera::portal_crossing::{PortalCrossingDetector, PortalCrossingEvent, PortalVolume};
use camera::world_collision::{
    CAMERA_COLLISION_RADIUS, COLLISION_SUBSTEP, CameraCollisionConfig, CollisionState,
    NoclipToggle, is_camera_passable, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, angles_for, apply_free_fly_input,
};
use core::math::{IVec3, Vec3};
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::preview::AdaptivePreview;
use renderer::raytracer::{RenderQuality, VoxelScene};
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::light::Light;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::time::Instant;

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

fn eye_under(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 - 1.5, t.z as f32 + 0.5)
}

fn eye_over(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 + 2.0, t.z as f32 + 0.5)
}

/// A move with collision that may turn a corner like a walker does.
fn step(scene: &WorldScene, from: Vec3, target: Vec3) -> Vec3 {
    let direct = resolve(scene, from, target - from);
    if (direct - target).length() < 1e-3 {
        return direct;
    }
    for corner in [v(target.x, from.y, target.z), v(from.x, target.y, from.z)] {
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

/// Screen right of the renderer's own frame.
fn screen_right(state: &WorldFreeFlyCameraState) -> Vec3 {
    let camera: Camera = state.camera(4.0 / 3.0);
    camera.basis().right
}

fn d_direction(state: &WorldFreeFlyCameraState) -> Vec3 {
    let mut moved = *state;
    apply_free_fly_input(
        &mut moved,
        &FreeFlyInput {
            move_right: 1.0,
            delta_time: 0.1,
            ..Default::default()
        },
    );
    (moved.position - state.position).normalize()
}

fn red_black(position: Vec3, target: Vec3) -> WorldFreeFlyCameraState {
    let mut s = WorldFreeFlyCameraState::looking_at(position, target);
    s.realm = WorldRealm::RedBlack;
    s.local_up = WorldRealm::RedBlack.up();
    let (yaw, pitch) = angles_for(target - position, s.local_up);
    s.yaw = yaw;
    s.pitch = pitch;
    s
}

/// The full World update of one frame as the viewer runs it: input,
/// collision (unless noclip), portal detection, roll.
struct Rig {
    scene: WorldScene,
    state: WorldFreeFlyCameraState,
    detector: PortalCrossingDetector,
    collision: CollisionState,
}

impl Rig {
    fn new() -> Self {
        let scene = WorldScene::new();
        let detector =
            PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
        Self {
            scene,
            state: world_free_fly_camera(),
            detector,
            collision: CollisionState::new(),
        }
    }

    fn teleport(&mut self, position: Vec3, target: Vec3) {
        let pose = WorldFreeFlyCameraState::looking_at(position, target);
        self.state.position = pose.position;
        let (yaw, pitch) = angles_for(target - position, self.state.local_up);
        self.state.yaw = yaw;
        self.state.pitch = pitch;
    }

    fn frame(&mut self, input: FreeFlyInput) -> Option<PortalCrossingEvent> {
        let rolling = self.state.advance_transition(input.delta_time.max(0.0));
        let _ = rolling;
        let mut event = None;
        if input.is_active() {
            let previous = self.state.position;
            apply_free_fly_input(&mut self.state, &input);
            if input.reset {
                self.collision.reset();
            } else if self.collision.collision_enabled() {
                self.state.position =
                    resolve(&self.scene, previous, self.state.position - previous);
            }
            if input.reset {
                self.detector.reset();
            } else if self.state.transition().is_none() {
                event = self
                    .detector
                    .detect(previous, self.state.position, self.state.realm);
                if let Some(e) = event {
                    self.state.begin_transition(e);
                }
            }
        }
        event
    }

    fn fly_until(&mut self, frames: usize) -> Option<PortalCrossingEvent> {
        let mut fired = None;
        for _ in 0..frames {
            let e = self.frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 60.0,
                ..Default::default()
            });
            fired = fired.or(e);
            if fired.is_some() && self.state.transition().is_none() {
                break;
            }
        }
        fired
    }
}

// 1, 2: defaults.
#[test]
fn collision_is_on_and_noclip_off_by_default() {
    let state = CollisionState::new();
    assert!(state.collision_enabled() && !state.is_noclip());
    assert_eq!(CollisionState::default(), state);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("collision_state: CollisionState::new(),"));
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert!(COLLISION_SUBSTEP <= CAMERA_COLLISION_RADIUS);
}

// 3, 4: full cubes and partial shapes.
#[test]
fn full_cubes_block_and_partial_shapes_use_their_real_geometry() {
    let scene = WorldScene::new();
    for t in [
        BlockType::Stone,
        BlockType::Dirt,
        BlockType::Glass,
        BlockType::Log,
        BlockType::WoodPlanks,
        BlockType::Leaves,
        BlockType::SmoothBasalt,
    ] {
        let (cell, _) = scene
            .world()
            .iter()
            .find(|(_, b)| b.block_type() == t)
            .unwrap();
        assert!(
            !clear(
                &scene,
                v(
                    cell.x as f32 + 0.5,
                    cell.y as f32 + 0.5,
                    cell.z as f32 + 0.5
                )
            ),
            "{t:?}"
        );
    }
    // A South stair of the upper descent: open above its low half, solid
    // in its raised half.
    let (tread, o) = scene.upper_descent().treads[2];
    assert_eq!(o, Orientation::South);
    assert!(clear(
        &scene,
        v(
            tread.x as f32 + 0.5,
            tread.y as f32 + 0.8,
            tread.z as f32 + 0.75
        )
    ));
    assert!(!clear(
        &scene,
        v(
            tread.x as f32 + 0.5,
            tread.y as f32 + 0.8,
            tread.z as f32 + 0.25
        )
    ));
    // The porch fence rails.
    assert!(clear(&scene, v(6.9, 6.5, 13.5625 + 0.29)));
    assert!(!clear(&scene, v(6.9, 6.5, 13.5625 + 0.19)));
}

// 5-8: frame, core, door, walls.
#[test]
fn the_portal_frame_and_house_walls_are_solid_and_core_and_door_passable() {
    let scene = WorldScene::new();
    for cell in &scene.portal().frame {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    for cell in &scene.portal().core {
        assert!(!is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    for cell in &scene.house_exterior().door {
        assert!(!is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    for cell in scene
        .house()
        .walls
        .iter()
        .chain(scene.house().corners.iter())
        .chain(scene.house_exterior().glass.iter())
    {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    assert!(
        is_camera_passable(BlockType::PortalCoreDarkCrimson)
            && is_camera_passable(BlockType::WoodDoor)
    );
    assert!(
        !is_camera_passable(BlockType::PortalFrameRedObsidian)
            && !is_camera_passable(BlockType::Glass)
    );
    // Beside the door the wall stops the camera; the door lets it in.
    assert!(resolve(&scene, v(7.5, 7.5, 13.5), v(0.0, 0.0, -3.0)).z >= 13.25 - 1e-3);
    assert!(resolve(&scene, v(8.5, 7.5, 13.5), v(0.0, 0.0, -3.0)).z < 12.0);
}

// 9-11: N toggle, refusal inside solid, R reset.
#[test]
fn noclip_toggles_refuses_inside_solid_and_resets() {
    let scene = WorldScene::new();
    let mut s = CollisionState::new();
    let air = world_free_fly_camera().position;
    assert_eq!(
        s.toggle_noclip(scene.world(), air, &cfg()),
        NoclipToggle::Enabled
    );
    assert_eq!(
        s.toggle_noclip(scene.world(), v(16.5, -11.5, 10.5), &cfg()),
        NoclipToggle::RefusedInsideSolid
    );
    assert!(s.is_noclip());
    assert_eq!(
        s.toggle_noclip(scene.world(), air, &cfg()),
        NoclipToggle::Disabled
    );
    s.toggle_noclip(scene.world(), air, &cfg());
    s.reset();
    assert!(s.collision_enabled());
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("rl.is_key_pressed(KeyboardKey::KEY_N)"));
    assert!(app.contains("| N noclip | R reset"));
}

// 12, 13: A/D in both realms.
#[test]
fn a_and_d_follow_the_screen_in_both_realms() {
    for state in [
        world_free_fly_camera(),
        red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        red_black(v(16.5, -16.5, 2.5), v(16.5, -16.5, -5.0)),
    ] {
        let right = screen_right(&state);
        assert!(d_direction(&state).dot(right) > 0.999, "{:?}", state.realm);
        let mut moved = state;
        apply_free_fly_input(
            &mut moved,
            &FreeFlyInput {
                move_right: -1.0,
                delta_time: 0.1,
                ..Default::default()
            },
        );
        assert!((moved.position - state.position).normalize().dot(right) < -0.999);
    }
    let src = std::fs::read_to_string("src/camera/world_free_fly.rs").unwrap();
    assert!(!src.contains("swap") && src.contains("REFERENCE_HEADING.cross(up"));
}

// 14-16: forward/reverse crossing basis and no drift.
#[test]
fn crossings_keep_the_basis_forward_reverse_and_repeated() {
    let mut rig = Rig::new();
    let home_forward = rig.state.forward();
    let home_right = screen_right(&rig.state);
    for round in 0..4 {
        rig.teleport(v(16.5, -7.5, 17.5), v(16.5, -7.5, 14.5));
        let e = rig.fly_until(400);
        assert_eq!(
            e,
            Some(PortalCrossingEvent::OverworldToRedBlack),
            "round {round}"
        );
        assert_eq!(rig.state.realm, WorldRealm::RedBlack);
        let f = rig.state.forward();
        assert!(f.z < -0.9, "forward drifted: {f:?}");
        assert!(d_direction(&rig.state).dot(screen_right(&rig.state)) > 0.999);
        // Turn around inside the exit clearance and come back.
        rig.teleport(v(16.5, -7.5, 13.5), v(16.5, -7.5, 17.0));
        let e = rig.fly_until(400);
        assert_eq!(
            e,
            Some(PortalCrossingEvent::RedBlackToOverworld),
            "round {round}"
        );
        assert_eq!(rig.state.realm, WorldRealm::Overworld);
        assert_eq!(rig.state.local_up, WorldRealm::Overworld.up());
        assert!(d_direction(&rig.state).dot(screen_right(&rig.state)) > 0.999);
        // Back home: the exact home frame every time.
        rig.frame(FreeFlyInput {
            reset: true,
            ..Default::default()
        });
        assert!((rig.state.forward() - home_forward).length() < 1e-4);
        assert!((screen_right(&rig.state) - home_right).length() < 1e-4);
    }
}

// 17-20: old stairs removed, first stair behind the portal, Down, exit clear.
#[test]
fn the_route_geometry_is_as_specified() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let route = scene.inverted_route();
    for k in 0..7 {
        let cell = IVec3::new(r.cutaway.min_x - 1, r.shelf_y - 1 - k, r.cutaway.min_z + k);
        assert_ne!(
            scene.world().get(cell).map(|b| b.block_type()),
            Some(BlockType::WoodStairs)
        );
    }
    assert_eq!(route.first(), Some(IVec3::new(16, -6, 12)));
    assert_eq!(route.layout.portal_exit, IVec3::new(16, -7, 13));
    for (cell, o) in &route.treads {
        assert_eq!(*o, Orientation::Down);
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::WoodStairs
        );
    }
    for cell in route.layout.exit_clearance() {
        assert!(!scene.world().contains(cell));
    }
    assert!(clear(&scene, v(16.5, -7.5, 13.5)));
    assert_eq!(route.treads.len(), 10);
}

// 21-24: contiguous lower path, surface connection, reverse, no noclip.
#[test]
fn the_canonical_route_is_walkable_both_ways_without_noclip() {
    let scene = WorldScene::new();
    let mut points = Vec::new();
    let end = *scene.path().cells.last().unwrap();
    points.push(v(
        end.x as f32 + 0.5,
        end.y as f32 + 2.0,
        end.z as f32 + 0.5,
    ));
    for (t, _) in &scene.upper_descent().treads {
        points.push(eye_over(*t));
    }
    points.push(v(16.5, -7.5, 16.5));
    points.push(v(16.5, -7.5, 14.5));
    points.push(v(16.5, -7.5, 13.5));
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye_under(*t));
    }
    let l = scene.inverted_route().layout;
    points.push(eye_under(IVec3::new(l.x, l.platform_y, l.platform_z_max)));
    points.push(eye_under(IVec3::new(l.x, l.platform_y, l.platform_z_min)));
    points.push(eye_under(IVec3::new(
        l.x,
        l.platform_y,
        l.platform_z_min - 2,
    )));
    walk(&scene, &points);
    points.reverse();
    walk(&scene, &points);
    assert!(CollisionState::new().collision_enabled());
    // The landing is flush with the rim surface.
    assert!(
        scene
            .lower_surface()
            .cells
            .iter()
            .any(|c| c.y == l.platform_y
                && (c.x - l.x).abs() <= 2
                && (c.z - l.platform_z_max).abs() <= 1)
    );
}

// The full traversal through the real detector, roll and collision (route B).
#[test]
fn route_b_crosses_rolls_and_climbs_the_inverted_stairs() {
    let mut rig = Rig::new();
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    let e = rig.fly_until(400);
    assert_eq!(e, Some(PortalCrossingEvent::OverworldToRedBlack));
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, v(0.0, -1.0, 0.0));
    assert!(rig.state.background().is_sky());
    assert!(clear(&rig.scene, rig.state.position));
    // Now climb: forward (north) and Space (local up = -Y) along the flight.
    let mut deepest = rig.state.position.y;
    for _ in 0..600 {
        rig.frame(FreeFlyInput {
            move_forward: 1.0,
            move_up: 1.0,
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
        deepest = deepest.min(rig.state.position.y);
    }
    assert!(
        rig.state.position.z < 5.0,
        "did not travel north through the tunnel: {:?}",
        rig.state.position
    );
    assert!(
        deepest < -14.0,
        "did not climb toward the underside: {deepest}"
    );
    assert!(clear(&rig.scene, rig.state.position));
}

// 25: the Gate 13.6 renderer is unchanged.
#[test]
fn the_gate_13_6_renderer_is_unchanged() {
    let src = std::fs::read_to_string("src/renderer/raytracer.rs").unwrap();
    assert!(src.contains("bounds.clip_distance(ray.origin, ray.direction, max_distance)?"));
    assert!(src.contains("pub const INTERACTIVE_REFLECTION_THRESHOLD: f32 = 0.05;"));
    let parallel = std::fs::read_to_string("src/renderer/parallel.rs").unwrap();
    assert!(parallel.contains("std::thread::scope"));
    let preview = std::fs::read_to_string("src/renderer/preview.rs").unwrap();
    assert!(preview.contains("Duration::from_millis(50)"));
    let refinement = std::fs::read_to_string("src/renderer/refinement.rs").unwrap();
    assert!(
        refinement.contains("Duration::from_millis(200)")
            && refinement.contains("Duration::from_millis(25)")
    );
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(
        app.contains(".update_texture(&self.rgba)")
            && app.contains("FullRefinement::new(full_width, full_height)")
    );
    assert_eq!(AdaptivePreview::new().resolution(800, 600), (400, 300));
    // Collision never touches the per-pixel path.
    for f in [
        "src/renderer/raytracer.rs",
        "src/renderer/parallel.rs",
        "src/renderer/refinement.rs",
    ] {
        assert!(
            !std::fs::read_to_string(f)
                .unwrap()
                .contains("world_collision")
        );
    }
}

// 26-28: Catalog, crates, Raylib 3D.
#[test]
fn catalog_crates_and_raylib_are_untouched() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(official_entries().len(), 44);
    assert_eq!(CatalogScene::new().entries().len(), 46);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog = &app[app.find("impl ViewerMode for CatalogMode").unwrap()
        ..app
            .find("/// The main world: the Overworld `WorldScene`.")
            .unwrap()];
    assert!(
        !catalog.contains("collision") && !catalog.contains("noclip") && !catalog.contains("KEY_N")
    );
    assert!(
        catalog
            .contains("\"Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset\"")
    );
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = &manifest[manifest.find("[dependencies]").unwrap()..];
    let deps = &deps[..deps.find("\n[").map_or(deps.len(), |i| i)];
    assert_eq!(
        deps.lines().filter(|l| l.contains('=')).collect::<Vec<_>>(),
        vec!["raylib = \"5.5\""]
    );
    let mut code = String::new();
    for entry in walk_dir("src") {
        code.push_str(&std::fs::read_to_string(&entry).unwrap());
    }
    for token in [
        "Camera3D",
        "BeginMode3D",
        "DrawCube",
        "DrawCubeV",
        "DrawModel",
        "Shader",
        "Mesh",
        "Model",
    ] {
        assert!(!has_token(&code, token), "{token} in src");
    }
}

fn walk_dir(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk_dir(path.to_str().unwrap()));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path.to_str().unwrap().to_string());
        }
    }
    out
}

fn has_token(code: &str, token: &str) -> bool {
    let bytes = code.as_bytes();
    let mut from = 0;
    while let Some(i) = code[from..].find(token) {
        let start = from + i;
        let end = start + token.len();
        let before =
            start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let after = end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
        if !before && !after {
            return true;
        }
        from = end;
    }
    false
}

/// The geometry delta the audit reports.
#[test]
fn reportable_geometry_delta() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    println!(
        "GATE14 voxels={} treads={} dug={} supports={} platform={:?} rim_step={:?} rim_support={:?}",
        scene.world().len(),
        route.treads.len(),
        route.dug.len(),
        route.supports.len(),
        route.layout.platform_cells(),
        route.layout.rim_step,
        route.layout.rim_support
    );
    println!("GATE14 dug cells: {:?}", route.dug);
    println!(
        "GATE14 portal frame={} core={} lights={} point_lights={}",
        scene.portal().frame.len(),
        scene.portal().core.len(),
        world_lights().len(),
        world_lights()
            .iter()
            .filter(|l| matches!(l, Light::Point(_)))
            .count()
    );
    assert_eq!(
        (scene.portal().frame.len(), scene.portal().core.len()),
        (18, 12)
    );
    assert_eq!(world_lights().len(), 5);
}

/// Release benchmark of Gate 13.6's A, C, F poses plus the collision cost:
/// `cargo test --release --test gate14_traversal_closure_tests bench_release_traversal -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_traversal() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let scene = WorldScene::new();
    let materials: MaterialLibrary = world_materials(&textures);
    let lights = world_lights();
    let threads = ParallelRenderConfig::detect();
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let poses = [
        ("A_full_diorama_home", world_free_fly_camera()),
        (
            "C_portal_cutaway",
            WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
        ),
        (
            "F_just_after_crossing",
            red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
        ),
        (
            "H_inverted_flight",
            red_black(v(16.5, -9.5, 11.5), v(16.5, -12.0, 8.0)),
        ),
        (
            "I_landing_underside",
            red_black(v(16.5, -17.5, 1.5), v(12.0, -17.0, 12.0)),
        ),
    ];
    println!("BENCH workers={}", threads.workers);
    let mut rgba = Vec::new();
    for (name, state) in poses {
        let camera = state.camera(4.0 / 3.0);
        let vs = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: camera.position,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        let mut preview = AdaptivePreview::new();
        let mut fb = Framebuffer::new(400, 300);
        for _ in 0..12 {
            let (w, h) = preview.resolution(800, 600);
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            preview.record(a.elapsed());
        }
        let (w, h) = preview.resolution(800, 600);
        let mut total = Vec::new();
        for _ in 0..9 {
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            fb.write_rgba8(&mut rgba);
            total.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let mut full = Vec::new();
        let mut full_fb = Framebuffer::new(800, 600);
        for _ in 0..3 {
            let a = Instant::now();
            render_parallel(&mut full_fb, &camera, &vs, RenderQuality::Full, &threads);
            full.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let t = p50(&mut total);
        println!(
            "BENCH {name} interactive resolution={w}x{h} total_ms={t:.1} fps={:.1} full_ms={:.1}",
            1000.0 / t,
            p50(&mut full)
        );
    }
    // Collision cost: one update's worth of movement (a 60 Hz frame at
    // full speed) resolved against the world, in the open and in the tunnel.
    for (name, from, delta) in [
        (
            "open_air",
            world_free_fly_camera().position,
            v(0.0, 0.0, -4.0 / 60.0),
        ),
        (
            "tunnel",
            v(16.5, -9.5, 11.5),
            v(0.0, -4.0 / 60.0, -4.0 / 60.0),
        ),
        (
            "against_wall",
            v(16.5, -4.5, 15.3),
            v(0.0, 0.0, -4.0 / 60.0),
        ),
    ] {
        let iters = 20_000;
        let a = Instant::now();
        let mut acc = 0.0;
        for _ in 0..iters {
            acc += resolve(&scene, from, delta).x;
        }
        std::hint::black_box(acc);
        println!(
            "BENCH collision {name} per_update_us={:.2}",
            a.elapsed().as_secs_f64() * 1e6 / iters as f64
        );
    }
}
