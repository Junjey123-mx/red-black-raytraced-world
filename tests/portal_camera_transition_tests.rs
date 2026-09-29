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
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
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

use camera::portal_crossing::PortalCrossingEvent;
use camera::world_free_fly::{
    FreeFlyInput, PORTAL_TRANSITION_DURATION, PortalTransitionState, WorldFreeFlyCameraState,
    WorldRealm, apply_free_fly_input, rotate_around, smoothstep,
};
use core::math::Vec3;
use scene::world::world_free_fly_camera;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

fn unit(v: Vec3) -> bool {
    finite(v) && (v.length() - 1.0).abs() < 1e-4
}

fn orthonormal(state: &WorldFreeFlyCameraState) {
    let (f, r, u) = (state.forward(), state.right(), state.up());
    assert!(unit(f) && unit(r) && unit(u), "{f:?} {r:?} {u:?}");
    assert!(f.dot(r).abs() < 1e-4 && f.dot(u).abs() < 1e-4 && r.dot(u).abs() < 1e-4);
    assert!((f.cross(u) - r).length() < 1e-4);
}

/// A level camera in the Overworld, looking north.
fn level() -> WorldFreeFlyCameraState {
    let mut state = world_free_fly_camera();
    state.pitch = 0.0;
    state.yaw = 0.0;
    state
}

#[test]
fn a_crossing_event_starts_the_transition_once() {
    let mut state = level();
    assert!(state.transition().is_none());
    assert!(state.begin_transition(PortalCrossingEvent::OverworldToRedBlack));
    let t = state.transition().unwrap();
    assert_eq!(t.from_realm, WorldRealm::Overworld);
    assert_eq!(t.to_realm, WorldRealm::RedBlack);
    assert_eq!(t.elapsed, 0.0);
    assert_eq!(t.duration, PORTAL_TRANSITION_DURATION);
    assert!((0.65..=1.0).contains(&PORTAL_TRANSITION_DURATION));
    // A second crossing while rolling is ignored, as is an event for the
    // wrong realm.
    assert!(!state.begin_transition(PortalCrossingEvent::OverworldToRedBlack));
    assert!(!state.begin_transition(PortalCrossingEvent::RedBlackToOverworld));
    assert_eq!(state.realm, WorldRealm::Overworld);
}

#[test]
fn the_roll_goes_from_plus_y_through_ninety_degrees_to_minus_y() {
    let mut state = level();
    let up0 = state.up();
    assert!((up0 - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-4);
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    // Progress 0: still +Y.
    assert!((state.up() - up0).length() < 1e-4);
    // Progress 0.5 (smoothstep(0.5) = 0.5): rotated about 90 degrees.
    state.advance_transition(PORTAL_TRANSITION_DURATION * 0.5);
    let t = state.transition().unwrap();
    assert!((t.progress() - 0.5).abs() < 1e-4 && (t.eased() - 0.5).abs() < 1e-4);
    assert!(state.up().dot(up0).abs() < 1e-3, "{:?}", state.up());
    assert!(unit(state.up()));
    orthonormal(&state);
    // Progress 1: -Y, realm switched, transition gone.
    let forward_before = state.forward();
    state.advance_transition(PORTAL_TRANSITION_DURATION * 0.5 + 1e-3);
    assert!(state.transition().is_none());
    assert_eq!(state.realm, WorldRealm::RedBlack);
    assert_eq!(state.local_up, Vec3::new(0.0, -1.0, 0.0));
    assert!(
        (state.up() - Vec3::new(0.0, -1.0, 0.0)).length() < 1e-3,
        "{:?}",
        state.up()
    );
    assert!(
        (state.forward() - forward_before).length() < 1e-4,
        "forward drifted"
    );
    orthonormal(&state);
}

#[test]
fn the_reverse_transition_returns_to_plus_y() {
    let mut state = level();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(PORTAL_TRANSITION_DURATION + 0.01);
    assert_eq!(state.realm, WorldRealm::RedBlack);
    let forward = state.forward();
    assert!(state.begin_transition(PortalCrossingEvent::RedBlackToOverworld));
    let mut steps = 0;
    while state.transition().is_some() {
        state.advance_transition(1.0 / 60.0);
        orthonormal(&state);
        steps += 1;
        assert!(steps < 200);
    }
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert!((state.up() - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-3);
    assert!((state.forward() - forward).length() < 1e-4);
    // Roughly 0.8 s of 60 Hz frames.
    assert!((45..=52).contains(&steps), "{steps} frames");
}

#[test]
fn the_basis_stays_orthonormal_and_finite_through_the_roll_with_pitch() {
    let mut state = world_free_fly_camera();
    state.pitch = 0.6;
    state.yaw = 2.1;
    let forward0 = state.forward();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    let mut previous_up = state.up();
    while state.transition().is_some() {
        state.advance_transition(0.02);
        orthonormal(&state);
        // Smooth: no large jump between frames.
        assert!(
            (state.up() - previous_up).length() < 0.25,
            "snap in the roll"
        );
        previous_up = state.up();
        assert!((state.forward() - forward0).length() < 1e-4);
    }
    assert_eq!(state.realm, WorldRealm::RedBlack);
    assert!((state.pitch + 0.6).abs() < 1e-5);
    assert!((state.forward() - forward0).length() < 1e-4);
    assert!(
        (state.up()
            + forward0
                .cross(forward0.cross(Vec3::new(0.0, 1.0, 0.0)).normalize())
                .normalize())
        .length()
            < 1e-3
            || state.up().y < 0.0
    );
    assert!(state.up().y < 0.0);
}

#[test]
fn movement_during_the_roll_uses_the_interpolated_frame() {
    let mut state = level();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(PORTAL_TRANSITION_DURATION * 0.5);
    let up_mid = state.up();
    let right_mid = state.right();
    let before = state.position;
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            move_up: 1.0,
            delta_time: 0.1,
            ..Default::default()
        },
    );
    let delta = state.position - before;
    assert!(
        (delta.normalize() - up_mid).length() < 1e-3,
        "Space follows the rolled up"
    );
    let before = state.position;
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            move_right: 1.0,
            delta_time: 0.1,
            ..Default::default()
        },
    );
    assert!(((state.position - before).normalize() - right_mid).length() < 1e-3);
    // W keeps working and follows forward.
    let before = state.position;
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            move_forward: 1.0,
            delta_time: 0.1,
            ..Default::default()
        },
    );
    assert!(((state.position - before).normalize() - state.forward()).length() < 1e-3);
    assert!(state.transition().is_some());
}

#[test]
fn reset_during_a_transition_cancels_it_and_restores_the_overworld() {
    let mut state = world_free_fly_camera();
    let home = state.home();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(0.3);
    state.position = Vec3::new(3.0, -9.0, 3.0);
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert!(state.transition().is_none());
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(state.position, home.position);
    assert_eq!((state.yaw, state.pitch), (home.yaw, home.pitch));
    assert!(!state.advance_transition(0.1));
}

#[test]
fn easing_and_rotation_helpers_are_the_projects_own() {
    assert_eq!(smoothstep(0.0), 0.0);
    assert_eq!(smoothstep(1.0), 1.0);
    assert_eq!(smoothstep(0.5), 0.5);
    assert!(smoothstep(0.1) < 0.1 && smoothstep(0.9) > 0.9);
    let axis = Vec3::new(0.0, 0.0, -1.0);
    let v = Vec3::new(0.0, 1.0, 0.0);
    assert!((rotate_around(v, axis, std::f32::consts::PI) + v).length() < 1e-5);
    let quarter = rotate_around(v, axis, std::f32::consts::FRAC_PI_2);
    assert!(quarter.dot(v).abs() < 1e-5 && unit(quarter));
    let mut t = PortalTransitionState::start(PortalCrossingEvent::RedBlackToOverworld);
    t.advance(f32::NAN);
    t.advance(-1.0);
    assert_eq!(t.elapsed, 0.0);
    t.advance(100.0);
    assert!(t.is_complete());
    assert_eq!(t.eased(), 1.0);
}
