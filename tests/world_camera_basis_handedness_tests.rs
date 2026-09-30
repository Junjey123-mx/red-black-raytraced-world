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

use camera::camera::Camera;
use camera::portal_crossing::{PortalCrossingDetector, PortalCrossingEvent, PortalVolume};
use camera::world_free_fly::{
    FreeFlyInput, REFERENCE_HEADING, WorldFreeFlyCameraState, WorldRealm, angles_for,
    apply_free_fly_input, heading, heading_about, look_direction,
};
use core::math::Vec3;
use scene::world::{WorldScene, world_free_fly_camera};

const EPS: f32 = 1e-4;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn near(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-3
}

/// The renderer's own frame for the state (what the image is built from).
fn screen_basis(state: &WorldFreeFlyCameraState) -> (Vec3, Vec3, Vec3) {
    let camera: Camera = state.camera(4.0 / 3.0);
    let b = camera.basis();
    (b.forward, b.right, b.up)
}

fn assert_orthonormal(f: Vec3, r: Vec3, u: Vec3, label: &str) {
    for (name, x) in [("forward", f), ("right", r), ("up", u)] {
        assert!(
            (x.length() - 1.0).abs() < EPS,
            "{label}: |{name}| = {}",
            x.length()
        );
        assert!(
            x.x.is_finite() && x.y.is_finite() && x.z.is_finite(),
            "{label}: {name} not finite"
        );
    }
    assert!(
        f.dot(r).abs() < EPS && f.dot(u).abs() < EPS && r.dot(u).abs() < EPS,
        "{label}: not orthogonal"
    );
    // Right-handed: forward x up = right (the one convention, everywhere).
    assert!(near(f.cross(u), r), "{label}: handedness flipped");
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

/// Where a movement input takes the camera, as a unit direction.
fn move_direction(state: &WorldFreeFlyCameraState, input: FreeFlyInput) -> Vec3 {
    let mut moved = *state;
    apply_free_fly_input(&mut moved, &input);
    (moved.position - state.position).normalize()
}

/// Turns the camera by a half turn through clamped mouse looks.
fn turn_around(state: &mut WorldFreeFlyCameraState) {
    let px_per_look = 180.0;
    let per_look = px_per_look * state.mouse_sensitivity;
    let looks = (std::f32::consts::PI / per_look).round() as usize;
    for _ in 0..looks {
        state.look(px_per_look, 0.0);
    }
}

fn strafe(right: f32) -> FreeFlyInput {
    FreeFlyInput {
        move_right: right,
        delta_time: 0.1,
        ..Default::default()
    }
}

/// Crosses the portal in the given direction from the given side, running
/// the real detector and the full roll.
fn cross(
    state: &mut WorldFreeFlyCameraState,
    detector: &mut PortalCrossingDetector,
    expected: PortalCrossingEvent,
) {
    let mut fired = false;
    for _ in 0..400 {
        let previous = state.position;
        apply_free_fly_input(
            state,
            &FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 60.0,
                ..Default::default()
            },
        );
        if !fired && let Some(event) = detector.detect(previous, state.position, state.realm) {
            assert_eq!(event, expected);
            assert!(state.begin_transition(event));
            fired = true;
        }
        if fired {
            state.advance_transition(1.0 / 60.0);
            if state.transition().is_none() {
                return;
            }
        }
    }
    panic!("never completed the crossing");
}

#[test]
fn the_overworld_basis_is_orthonormal() {
    let state = world_free_fly_camera();
    let (f, r, u) = screen_basis(&state);
    assert_orthonormal(f, r, u, "overworld");
    assert!(near(f, state.forward()) && near(r, state.right()) && near(u, state.up()));
    assert!(u.y > 0.0);
}

#[test]
fn the_red_black_basis_is_orthonormal() {
    let state = red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0));
    let (f, r, u) = screen_basis(&state);
    assert_orthonormal(f, r, u, "red-black");
    assert!(near(f, state.forward()) && near(r, state.right()) && near(u, state.up()));
    assert!(u.y < 0.0);
}

#[test]
fn basis_vectors_are_pairwise_perpendicular_everywhere() {
    for (yaw, pitch) in [
        (0.0, 0.0),
        (1.0, 0.4),
        (-2.5, -0.9),
        (3.0, 1.2),
        (0.7, -1.5),
    ] {
        for up in [WorldRealm::Overworld.up(), WorldRealm::RedBlack.up()] {
            let f = look_direction(yaw, pitch, up);
            let r = f.cross(f.cross(up).normalize().cross(f));
            let u = f.cross(up).normalize().cross(f);
            assert!(
                f.dot(u).abs() < EPS && f.dot(r).abs() < 1e-3 && r.dot(u).abs() < 1e-3,
                "{yaw} {pitch} {up:?}"
            );
        }
    }
}

#[test]
fn basis_vectors_are_unit_length_everywhere() {
    for (yaw, pitch) in [(0.0, 0.0), (1.0, 0.4), (-2.5, -0.9), (3.0, 1.2)] {
        for up in [WorldRealm::Overworld.up(), WorldRealm::RedBlack.up()] {
            assert!((look_direction(yaw, pitch, up).length() - 1.0).abs() < EPS);
            assert!((heading_about(yaw, up).length() - 1.0).abs() < EPS);
        }
    }
    assert!((REFERENCE_HEADING.length() - 1.0).abs() < EPS);
    assert!(near(heading(0.0), REFERENCE_HEADING));
    assert!(near(
        heading_about(0.0, WorldRealm::RedBlack.up()),
        REFERENCE_HEADING
    ));
}

#[test]
fn handedness_is_the_same_in_both_realms() {
    // Growing yaw turns toward the frame's right in both realms, and the
    // frame's right is `forward x up` in both.
    for up in [WorldRealm::Overworld.up(), WorldRealm::RedBlack.up()] {
        let f0 = heading_about(0.0, up);
        let right0 = f0.cross(up);
        let turned = heading_about(0.3, up);
        assert!(
            turned.dot(right0) > 0.0,
            "yaw turns the wrong way for up {up:?}"
        );
    }
    // Concretely: +X in the Overworld, -X in Red-Black.
    assert!(heading_about(0.3, WorldRealm::Overworld.up()).x > 0.0);
    assert!(heading_about(0.3, WorldRealm::RedBlack.up()).x < 0.0);
}

#[test]
fn a_moves_screen_left_in_the_overworld() {
    let state = world_free_fly_camera();
    let (_, right, _) = screen_basis(&state);
    assert!(move_direction(&state, strafe(-1.0)).dot(right) < -0.999);
}

#[test]
fn d_moves_screen_right_in_the_overworld() {
    let state = world_free_fly_camera();
    let (_, right, _) = screen_basis(&state);
    assert!(move_direction(&state, strafe(1.0)).dot(right) > 0.999);
}

#[test]
fn a_moves_screen_left_in_red_black() {
    for state in [
        red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
        red_black(v(5.0, -25.0, 5.0), v(20.0, -24.0, 9.0)),
    ] {
        let (_, right, _) = screen_basis(&state);
        assert!(
            move_direction(&state, strafe(-1.0)).dot(right) < -0.999,
            "{:?}",
            state.position
        );
    }
}

#[test]
fn d_moves_screen_right_in_red_black() {
    for state in [
        red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
        red_black(v(5.0, -25.0, 5.0), v(20.0, -24.0, 9.0)),
    ] {
        let (_, right, _) = screen_basis(&state);
        assert!(
            move_direction(&state, strafe(1.0)).dot(right) > 0.999,
            "{:?}",
            state.position
        );
    }
}

#[test]
fn the_basis_stays_valid_and_consistent_at_mid_roll() {
    let mut state = WorldFreeFlyCameraState::looking_at(v(16.5, -7.0, 15.2), v(16.5, -7.0, 10.0));
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    let forward_before = state.forward();
    for step in 0..8 {
        state.advance_transition(0.1);
        if state.transition().is_none() {
            break;
        }
        let (f, r, u) = screen_basis(&state);
        assert_orthonormal(f, r, u, &format!("mid-roll step {step}"));
        // The roll turns right and up about forward; forward is untouched.
        assert!(near(f, forward_before));
        assert!(near(r, state.right()) && near(u, state.up()));
        // A/D still follow the rolled screen right.
        assert!(move_direction(&state, strafe(1.0)).dot(r) > 0.999);
        assert!(move_direction(&state, strafe(-1.0)).dot(r) < -0.999);
    }
}

#[test]
fn the_reverse_roll_returns_to_the_overworld_frame() {
    let scene = WorldScene::new();
    let mut detector =
        PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
    let mut state = WorldFreeFlyCameraState::looking_at(v(16.5, -7.5, 18.0), v(16.5, -7.5, 14.5));
    let forward_start = state.forward();
    cross(
        &mut state,
        &mut detector,
        PortalCrossingEvent::OverworldToRedBlack,
    );
    assert_eq!(state.realm, WorldRealm::RedBlack);
    assert!(
        near(state.forward(), forward_start),
        "forward changed: {:?}",
        state.forward()
    );
    let (f, r, u) = screen_basis(&state);
    assert_orthonormal(f, r, u, "after forward crossing");
    assert!(u.y < 0.0);
    // Turn around and come back.
    turn_around(&mut state);
    let _ = detector; // re-armed far from the membrane during the turn
    let mut detector =
        PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
    cross(
        &mut state,
        &mut detector,
        PortalCrossingEvent::RedBlackToOverworld,
    );
    assert_eq!(state.realm, WorldRealm::Overworld);
    let (f, r, u) = screen_basis(&state);
    assert_orthonormal(f, r, u, "after reverse crossing");
    assert!(u.y > 0.0);
    assert!(move_direction(&state, strafe(1.0)).dot(r) > 0.999);
}

#[test]
fn repeated_crossings_never_drift_the_frame() {
    let scene = WorldScene::new();
    let volume = PortalVolume::from_anchor(&scene.rhombus().portal);
    let mut state = WorldFreeFlyCameraState::looking_at(v(16.5, -7.5, 18.0), v(16.5, -7.5, 14.5));
    let home_forward = state.forward();
    let (_, home_right, home_up) = screen_basis(&state);
    for round in 0..6 {
        let mut detector = PortalCrossingDetector::new(volume);
        cross(
            &mut state,
            &mut detector,
            PortalCrossingEvent::OverworldToRedBlack,
        );
        // Turn around in Red-Black.
        turn_around(&mut state);
        let mut detector = PortalCrossingDetector::new(volume);
        cross(
            &mut state,
            &mut detector,
            PortalCrossingEvent::RedBlackToOverworld,
        );
        // Turn around again: back to the original heading.
        turn_around(&mut state);
        assert_eq!(state.realm, WorldRealm::Overworld);
        let (f, r, u) = screen_basis(&state);
        assert!(
            near(f, home_forward) && near(r, home_right) && near(u, home_up),
            "drift after round {round}: {f:?} {r:?} {u:?}"
        );
        assert!(move_direction(&state, strafe(1.0)).dot(r) > 0.999);
    }
}

#[test]
fn mouse_yaw_turns_toward_screen_right_in_both_realms() {
    for mut state in [
        world_free_fly_camera(),
        red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
    ] {
        let (f0, right, _) = screen_basis(&state);
        state.look(40.0, 0.0);
        let f1 = state.forward();
        assert!(
            f1.dot(right) > 0.0,
            "mouse right turned left for realm {:?}",
            state.realm
        );
        assert!(f1.dot(f0) > 0.99);
        state.look(-80.0, 0.0);
        assert!(state.forward().dot(right) < 0.0);
        // Vertical look follows the local up.
        let mut up_state = state;
        up_state.look(0.0, -40.0);
        assert!(up_state.forward().dot(state.local_up) > f0.dot(state.local_up));
    }
}

#[test]
fn reset_restores_the_exact_home_frame() {
    let scene = WorldScene::new();
    let mut detector =
        PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
    let home = world_free_fly_camera();
    let mut state = home;
    let pose = WorldFreeFlyCameraState::looking_at(v(16.5, -7.5, 18.0), v(16.5, -7.5, 14.5));
    state.position = pose.position;
    state.yaw = pose.yaw;
    state.pitch = pose.pitch;
    cross(
        &mut state,
        &mut detector,
        PortalCrossingEvent::OverworldToRedBlack,
    );
    state.look(123.0, -45.0);
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert_eq!(
        (state.position, state.yaw, state.pitch),
        (home.position, home.yaw, home.pitch)
    );
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, WorldRealm::Overworld.up());
    let (f, r, u) = screen_basis(&state);
    let (hf, hr, hu) = screen_basis(&home);
    assert!(near(f, hf) && near(r, hr) && near(u, hu));
}
