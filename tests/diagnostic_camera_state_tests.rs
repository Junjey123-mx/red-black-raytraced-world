#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
}

use camera::diagnostic::{
    DIAGNOSTIC_FOV_DEGREES, DiagnosticCameraState, MAX_DISTANCE, MAX_PITCH, MIN_DISTANCE, MIN_PITCH,
};
use camera::projection::primary_ray;
use core::math::Vec3;

const EPS: f32 = 1e-4;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

fn state(target: Vec3, yaw: f32, pitch: f32, distance: f32) -> DiagnosticCameraState {
    DiagnosticCameraState::new(target, yaw, pitch, distance)
}

#[test]
fn the_initial_state_is_valid_and_looks_at_its_target() {
    let s = DiagnosticCameraState::default();

    assert!(finite(s.target) && finite(s.position()));
    assert!((MIN_PITCH..=MAX_PITCH).contains(&s.pitch));
    assert!((MIN_DISTANCE..=MAX_DISTANCE).contains(&s.distance));
    assert_eq!(s.target, Vec3::zero());
}

#[test]
fn the_position_is_finite_for_a_range_of_states() {
    for yaw in [-7.0, -3.0, 0.0, 1.0, 3.2, 100.0] {
        for pitch in [-2.0, -0.5, 0.0, 0.5, 2.0] {
            for distance in [0.0, 1.0, 10.0, 1000.0] {
                let s = state(Vec3::new(1.0, 2.0, 3.0), yaw, pitch, distance);
                assert!(finite(s.position()), "{yaw} {pitch} {distance}");
            }
        }
    }
}

#[test]
fn the_eye_is_exactly_distance_away_from_the_target() {
    for (yaw, pitch, distance) in [(0.0, 0.3, 5.0), (1.2, -0.7, 9.5), (-2.5, 1.0, 2.0)] {
        let target = Vec3::new(4.0, -1.0, 7.0);
        let s = state(target, yaw, pitch, distance);

        assert!(((s.position() - target).length() - s.distance).abs() < EPS);
    }
}

#[test]
fn yaw_swings_the_eye_around_the_vertical_axis() {
    let mut s = state(Vec3::zero(), 0.0, 0.0, 10.0);
    let front = s.position();
    assert!((front.z - 10.0).abs() < EPS && front.x.abs() < EPS);

    s.set_angles(std::f32::consts::FRAC_PI_2, 0.0);
    let side = s.position();
    assert!((side.x - 10.0).abs() < EPS && side.z.abs() < EPS);
    assert!(side.y.abs() < EPS, "yaw never changes the height");
}

#[test]
fn pitch_raises_and_lowers_the_eye() {
    let mut s = state(Vec3::zero(), 0.0, 0.0, 10.0);
    let level = s.position().y;

    s.set_angles(0.0, 0.5);
    assert!(s.position().y > level + 1.0);
    s.set_angles(0.0, -0.5);
    assert!(s.position().y < level - 1.0);
}

#[test]
fn pitch_is_limited_away_from_the_poles() {
    let mut s = DiagnosticCameraState::default();
    s.set_angles(0.0, 10.0);
    assert!((s.pitch - MAX_PITCH).abs() < EPS);
    s.set_angles(0.0, -10.0);
    assert!((s.pitch - MIN_PITCH).abs() < EPS);
    assert!(MAX_PITCH < std::f32::consts::FRAC_PI_2);

    // Even at the limit the camera basis stays valid.
    let camera = s.build_camera(4.0 / 3.0);
    let ray = primary_ray(&camera, 10, 10, 20, 20);
    assert!(finite(ray.direction) && ray.direction.length() > 0.99);
}

#[test]
fn distance_is_limited_and_never_zero() {
    let mut s = DiagnosticCameraState::default();
    s.set_distance(0.0);
    assert!(s.distance >= MIN_DISTANCE && s.distance > 0.0);
    s.set_distance(-5.0);
    assert!((s.distance - MIN_DISTANCE).abs() < EPS);
    s.set_distance(1.0e9);
    assert!((s.distance - MAX_DISTANCE).abs() < EPS);
}

#[test]
fn reset_restores_the_initial_pose_exactly() {
    let mut s = state(Vec3::new(3.0, 1.0, -2.0), 0.7, 0.4, 9.0);
    let initial = s;

    s.set_target(Vec3::new(50.0, 50.0, 50.0));
    s.set_angles(2.0, -1.0);
    s.set_distance(30.0);
    assert_ne!(s, initial);

    s.reset();
    assert_eq!(s, initial);
    assert_eq!(s.position(), initial.position());
}

#[test]
fn an_arbitrary_target_shifts_the_whole_orbit() {
    let a = state(Vec3::zero(), 0.6, 0.3, 8.0);
    let target = Vec3::new(10.0, -4.0, 25.0);
    let b = state(target, 0.6, 0.3, 8.0);

    let shift = b.position() - a.position();
    assert!((shift - target).length() < EPS);
    assert_eq!(b.target, target);
}

#[test]
fn non_finite_input_never_produces_nan() {
    let mut s = DiagnosticCameraState::default();
    let before = s;

    s.set_angles(f32::NAN, f32::INFINITY);
    s.set_distance(f32::NAN);
    s.set_target(Vec3::new(f32::NAN, 0.0, 0.0));
    assert_eq!(s, before, "invalid values are ignored");

    let bad = DiagnosticCameraState::new(
        Vec3::new(f32::INFINITY, 0.0, 0.0),
        f32::NAN,
        f32::NAN,
        f32::NEG_INFINITY,
    );
    assert!(finite(bad.target) && finite(bad.position()));
    assert!(bad.yaw.is_finite() && bad.pitch.is_finite() && bad.distance.is_finite());
}

#[test]
fn the_built_camera_matches_the_state_and_shoots_toward_the_target() {
    let s = state(Vec3::new(2.0, 1.0, -3.0), 0.9, 0.35, 11.0);
    let camera = s.build_camera(4.0 / 3.0);

    assert_eq!(camera.position, s.position());
    assert_eq!(camera.target, s.target);
    assert_eq!(camera.up, Vec3::new(0.0, 1.0, 0.0));
    assert!((camera.fov - DIAGNOSTIC_FOV_DEGREES).abs() < EPS);

    // The center primary ray aims at the target.
    let ray = primary_ray(&camera, 50, 50, 100, 100);
    let to_target = (s.target - s.position()).normalize();
    assert!(ray.direction.dot(to_target) > 0.9999);
}

#[test]
fn a_full_yaw_turn_keeps_the_orbit_distance() {
    let target = Vec3::new(1.0, 2.0, 3.0);
    let mut s = state(target, 0.0, 0.4, 7.0);
    let start = s.position();

    for step in 1..=72 {
        s.set_angles(step as f32 * TAU_STEP, s.pitch);
        assert!(
            ((s.position() - target).length() - 7.0).abs() < 1e-3,
            "step {step}"
        );
        assert!((-std::f32::consts::PI..=std::f32::consts::PI).contains(&s.yaw));
    }
    // 72 steps of 5 degrees is a full turn: back where it started.
    assert!((s.position() - start).length() < 1e-2);
}

const TAU_STEP: f32 = 5.0 * std::f32::consts::PI / 180.0;

#[test]
fn from_pose_is_the_inverse_of_position() {
    let position = Vec3::new(6.0, 5.6, 14.0);
    let target = Vec3::new(6.0, 0.6, 1.8);
    let s = DiagnosticCameraState::from_pose(position, target);

    assert!((s.position() - position).length() < 1e-3);
    assert_eq!(s.target, target);

    // Degenerate pose (eye on the target) falls back to the default view.
    let d = DiagnosticCameraState::from_pose(target, target);
    assert!(d.distance >= MIN_DISTANCE && finite(d.position()));
}

#[test]
fn clamp_reapplies_every_invariant() {
    let mut s = DiagnosticCameraState::default();
    s.yaw = 50.0;
    s.pitch = 9.0;
    s.distance = 0.0;
    s.clamp();

    assert!(s.yaw.abs() <= std::f32::consts::PI + EPS);
    assert!((MIN_PITCH..=MAX_PITCH).contains(&s.pitch));
    assert!(s.distance >= MIN_DISTANCE);
}
