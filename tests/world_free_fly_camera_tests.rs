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

use camera::diagnostic::DiagnosticCameraState;
use camera::world_free_fly::{
    FREE_FLY_MAX_PITCH, FreeFlyPose, WorldFreeFlyCameraState, WorldRealm, angles_for, heading,
    look_direction,
};
use core::math::Vec3;
use scene::world::{WorldScene, world_camera, world_free_fly_camera};

const ASPECT: f32 = 4.0 / 3.0;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

fn unit(v: Vec3) -> bool {
    finite(v) && (v.length() - 1.0).abs() < 1e-4
}

#[test]
fn the_initial_pose_is_finite_and_matches_the_official_framing() {
    let state = world_free_fly_camera();
    assert!(finite(state.position));
    assert!(state.yaw.is_finite() && state.pitch.is_finite());
    let official = world_camera(ASPECT);
    assert_eq!(state.position, official.position);
    // It looks where the official framing looks.
    let expected = (official.target - official.position).normalize();
    assert!((state.forward() - expected).length() < 1e-3);
    let camera = state.camera(ASPECT);
    assert_eq!(camera.position, official.position);
    assert!((camera.basis().forward - expected).length() < 1e-3);
    assert_eq!(camera.fov, 60.0);
}

#[test]
fn the_initial_realm_is_overworld_with_up_positive_y() {
    let state = world_free_fly_camera();
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(WorldRealm::Overworld.up(), Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(WorldRealm::RedBlack.up(), Vec3::new(0.0, -1.0, 0.0));
    assert!(state.up().y > 0.5);
}

#[test]
fn forward_right_and_up_form_an_orthonormal_right_handed_basis() {
    for (yaw, pitch) in [(0.0, 0.0), (1.2, 0.4), (-2.5, -0.9), (3.0, 1.5)] {
        for realm in [WorldRealm::Overworld, WorldRealm::RedBlack] {
            let mut state = WorldFreeFlyCameraState::from_pose(FreeFlyPose {
                position: Vec3::new(1.0, 2.0, 3.0),
                yaw,
                pitch,
            });
            state.realm = realm;
            state.local_up = realm.up();
            let (f, r, u) = (state.forward(), state.right(), state.up());
            assert!(unit(f) && unit(r) && unit(u), "{yaw} {pitch} {realm:?}");
            assert!(f.dot(r).abs() < 1e-4 && f.dot(u).abs() < 1e-4 && r.dot(u).abs() < 1e-4);
            // Right-handed: forward x up = right, and up points toward the
            // realm's vertical.
            assert!((f.cross(u) - r).length() < 1e-4);
            assert!(u.dot(realm.up()) > 0.0);
            // The camera built from the state uses that same basis.
            let basis = state.camera(ASPECT).basis();
            assert!((basis.forward - f).length() < 1e-4);
            assert!((basis.right - r).length() < 1e-4);
            assert!((basis.up - u).length() < 1e-4);
        }
    }
}

#[test]
fn yaw_and_pitch_conventions_round_trip() {
    assert!((heading(0.0) - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);
    assert!((heading(std::f32::consts::FRAC_PI_2) - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
    let up = Vec3::new(0.0, 1.0, 0.0);
    for (yaw, pitch) in [(0.3, 0.2), (-1.1, -0.7), (2.9, 1.2)] {
        let f = look_direction(yaw, pitch, up);
        let (y2, p2) = angles_for(f, up);
        assert!((look_direction(y2, p2, up) - f).length() < 1e-4);
        assert!((p2 - pitch).abs() < 1e-4);
    }
    assert!(FREE_FLY_MAX_PITCH < std::f32::consts::FRAC_PI_2);
    assert!(FREE_FLY_MAX_PITCH > 1.5);
}

#[test]
fn reset_restores_the_pose_and_the_overworld_frame() {
    let mut state = world_free_fly_camera();
    let home = state.home();
    state.position = Vec3::new(-40.0, -30.0, 9.0);
    state.yaw += 2.0;
    state.pitch = 0.9;
    state.realm = WorldRealm::RedBlack;
    state.local_up = WorldRealm::RedBlack.up();
    state.reset();
    assert_eq!(state.position, home.position);
    assert_eq!(state.yaw, home.yaw);
    assert_eq!(state.pitch, home.pitch);
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(state.home(), home);
}

#[test]
fn the_catalog_keeps_its_diagnostic_camera_and_the_world_builds_its_own() {
    // The diagnostic orbit camera type still exists and still starts from
    // the gallery framing.
    let gallery = scene::material_gallery::gallery_camera(ASPECT);
    let view = DiagnosticCameraState::from_pose(gallery.position, gallery.target);
    assert!((view.build_camera(ASPECT).position - gallery.position).length() < 1e-3);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog =
        &app[app.find("struct CatalogMode").unwrap()..app.find("struct WorldMode").unwrap()];
    assert!(!catalog.contains("WorldFreeFlyCameraState"));
    let world =
        &app[app.find("struct WorldMode").unwrap()..app.find("/// Casts one primary ray").unwrap()];
    assert!(world.contains("free_fly: WorldFreeFlyCameraState"));
    // Building the camera does not touch the world.
    let before = WorldScene::new();
    let _ = world_free_fly_camera();
    let after = WorldScene::new();
    assert_eq!(before.world().len(), after.world().len());
    assert_eq!(after.world().len(), 6784);
}

// ---------------------------------------------------------------------
// Navigation (C156)
// ---------------------------------------------------------------------

use camera::world_free_fly::{FREE_FLY_SPEED, FreeFlyInput, apply_free_fly_input};

fn fresh() -> WorldFreeFlyCameraState {
    world_free_fly_camera()
}

fn moved(input: FreeFlyInput) -> (WorldFreeFlyCameraState, Vec3) {
    let mut state = fresh();
    let before = state.position;
    apply_free_fly_input(&mut state, &input);
    (state, state.position - before)
}

#[test]
fn w_and_s_move_along_forward() {
    let base = fresh();
    let (_, w) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 0.2,
        ..Default::default()
    });
    let (_, s) = moved(FreeFlyInput {
        move_forward: -1.0,
        delta_time: 0.2,
        ..Default::default()
    });
    let expected = base.forward() * (FREE_FLY_SPEED * 0.2);
    assert!((w - expected).length() < 1e-4, "{w:?} vs {expected:?}");
    assert!((s + expected).length() < 1e-4);
}

#[test]
fn a_and_d_strafe_along_the_local_right() {
    let base = fresh();
    let (_, d) = moved(FreeFlyInput {
        move_right: 1.0,
        delta_time: 0.25,
        ..Default::default()
    });
    let (_, a) = moved(FreeFlyInput {
        move_right: -1.0,
        delta_time: 0.25,
        ..Default::default()
    });
    let expected = base.right() * (FREE_FLY_SPEED * 0.25);
    assert!((d - expected).length() < 1e-4);
    assert!((a + expected).length() < 1e-4);
    assert!(d.dot(base.forward()).abs() < 1e-3);
}

#[test]
fn space_and_shift_use_the_local_up_in_both_realms() {
    let (_, up) = moved(FreeFlyInput {
        move_up: 1.0,
        delta_time: 0.25,
        ..Default::default()
    });
    let (_, down) = moved(FreeFlyInput {
        move_up: -1.0,
        delta_time: 0.25,
        ..Default::default()
    });
    let base = fresh();
    assert!((up - base.up() * (FREE_FLY_SPEED * 0.25)).length() < 1e-4);
    assert!((down + base.up() * (FREE_FLY_SPEED * 0.25)).length() < 1e-4);
    assert!(up.y > 0.0);
    // In the Red-Black realm the local up is -Y: Space goes down in world Y.
    let mut inverted = fresh();
    inverted.pitch = 0.0;
    inverted.realm = WorldRealm::RedBlack;
    inverted.local_up = WorldRealm::RedBlack.up();
    let before = inverted.position;
    apply_free_fly_input(
        &mut inverted,
        &FreeFlyInput {
            move_up: 1.0,
            delta_time: 0.25,
            ..Default::default()
        },
    );
    let delta = inverted.position - before;
    assert!(delta.y < -0.9 * FREE_FLY_SPEED * 0.25, "{delta:?}");
}

#[test]
fn movement_scales_with_delta_time_and_never_teleports() {
    let (_, once) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 0.1,
        ..Default::default()
    });
    let (_, twice) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 0.2,
        ..Default::default()
    });
    assert!((twice - once * 2.0).length() < 1e-4);
    let (_, none) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 0.0,
        ..Default::default()
    });
    assert_eq!(none.length(), 0.0);
    let (_, stall) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 30.0,
        ..Default::default()
    });
    assert!(stall.length() <= FREE_FLY_SPEED * 0.25 + 1e-4);
    let (state, nan) = moved(FreeFlyInput {
        move_forward: 1.0,
        delta_time: f32::NAN,
        ..Default::default()
    });
    assert_eq!(nan.length(), 0.0);
    assert!(finite(state.position));
}

#[test]
fn mouse_yaw_turns_the_view_and_pitch_is_clamped() {
    let mut state = fresh();
    let before = state.forward();
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            look_dx: 120.0,
            ..Default::default()
        },
    );
    let after = state.forward();
    assert!((after - before).length() > 0.05);
    assert!(unit(after));
    // Turning right lowers the dot with the original right-hand side? No:
    // turning right moves forward toward the old right.
    assert!(after.dot(fresh().right()) > before.dot(fresh().right()));
    // Pitch clamps below the vertical in both directions.
    let mut state = fresh();
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            look_dy: -100_000.0,
            ..Default::default()
        },
    );
    for _ in 0..50 {
        apply_free_fly_input(
            &mut state,
            &FreeFlyInput {
                look_dy: -200.0,
                ..Default::default()
            },
        );
    }
    assert!(state.pitch <= FREE_FLY_MAX_PITCH && state.pitch > 1.5);
    assert!(state.forward().dot(state.local_up) < 0.9999);
    assert!(unit(state.right()) && unit(state.up()));
    for _ in 0..200 {
        apply_free_fly_input(
            &mut state,
            &FreeFlyInput {
                look_dy: 200.0,
                ..Default::default()
            },
        );
    }
    assert!(state.pitch >= -FREE_FLY_MAX_PITCH && state.pitch < -1.5);
    assert!(unit(state.right()) && unit(state.up()));
    // Non-finite mouse input is ignored.
    let yaw = state.yaw;
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            look_dx: f32::INFINITY,
            look_dy: f32::NAN,
            ..Default::default()
        },
    );
    assert_eq!(state.yaw, yaw);
}

#[test]
fn the_basis_stays_valid_along_a_long_flight() {
    let mut state = fresh();
    for i in 0..500 {
        let input = FreeFlyInput {
            move_forward: if i % 3 == 0 { 1.0 } else { -0.5 },
            move_right: if i % 5 == 0 { 1.0 } else { 0.0 },
            move_up: if i % 7 == 0 { -1.0 } else { 0.2 },
            look_dx: ((i as f32) * 0.37).sin() * 60.0,
            look_dy: ((i as f32) * 0.11).cos() * 40.0,
            delta_time: 1.0 / 60.0,
            reset: false,
        };
        apply_free_fly_input(&mut state, &input);
        assert!(finite(state.position));
        assert!(unit(state.forward()) && unit(state.right()) && unit(state.up()));
        assert!(state.yaw.abs() <= std::f32::consts::PI + 1e-4);
    }
}

#[test]
fn r_resets_the_flight_and_ignores_the_rest_of_the_frame() {
    let mut state = fresh();
    let home = state.home();
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            move_forward: 1.0,
            look_dx: 300.0,
            delta_time: 0.2,
            ..Default::default()
        },
    );
    assert_ne!(state.position, home.position);
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            reset: true,
            move_forward: 1.0,
            look_dx: 50.0,
            delta_time: 0.2,
            ..Default::default()
        },
    );
    assert_eq!(state.position, home.position);
    assert_eq!((state.yaw, state.pitch), (home.yaw, home.pitch));
    assert!(
        FreeFlyInput {
            reset: true,
            ..Default::default()
        }
        .is_active()
    );
    assert!(!FreeFlyInput::default().is_active());
    assert!(
        !FreeFlyInput {
            move_forward: 1.0,
            delta_time: 0.0,
            ..Default::default()
        }
        .is_active()
    );
}

#[test]
fn the_catalog_controls_are_unchanged_and_the_world_hud_is_free_fly() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog =
        &app[app.find("struct CatalogMode").unwrap()..app.find("struct WorldMode").unwrap()];
    for needed in [
        "poll_orbit_input",
        "apply_orbit_input",
        "select_previous",
        "select_next",
        "focus_camera",
        "KEY_LEFT_BRACKET",
        "KEY_F",
    ] {
        assert!(
            catalog.contains(needed),
            "{needed} missing from the catalog mode"
        );
    }
    assert!(
        catalog.contains("Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset")
    );
    assert!(!catalog.contains("free_fly"));
    let world =
        &app[app.find("struct WorldMode").unwrap()..app.find("/// Casts one primary ray").unwrap()];
    assert!(world.contains("poll_free_fly_input") && world.contains("apply_free_fly_input"));
    assert!(world.contains("WASD move | Mouse look | Space/Shift up/down | R reset"));
    assert!(!world.contains("apply_orbit_input"));
    for key in [
        "KEY_W",
        "KEY_S",
        "KEY_A",
        "KEY_D",
        "KEY_SPACE",
        "KEY_LEFT_SHIFT",
        "KEY_R",
        "get_mouse_delta",
        "get_frame_time",
    ] {
        assert!(app.contains(key), "{key}");
    }
}
