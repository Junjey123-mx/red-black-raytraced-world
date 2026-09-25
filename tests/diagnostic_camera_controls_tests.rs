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
    #[path = "../src/camera/controls.rs"]
    pub mod controls;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
}

use camera::controls::{
    MAX_DRAG_PIXELS_PER_FRAME, ORBIT_SENSITIVITY, OrbitInput, ZOOM_SENSITIVITY, apply_orbit_input,
};
use camera::diagnostic::{DiagnosticCameraState, MAX_DISTANCE, MAX_PITCH, MIN_DISTANCE, MIN_PITCH};
use core::math::Vec3;

const EPS: f32 = 1e-5;

fn start() -> DiagnosticCameraState {
    DiagnosticCameraState::new(Vec3::new(2.0, 1.0, 3.0), 0.4, 0.3, 10.0)
}

fn drag(dx: f32, dy: f32) -> OrbitInput {
    OrbitInput {
        drag_dx: dx,
        drag_dy: dy,
        ..OrbitInput::default()
    }
}

fn wheel(notches: f32) -> OrbitInput {
    OrbitInput {
        wheel: notches,
        ..OrbitInput::default()
    }
}

#[test]
fn a_horizontal_drag_changes_only_the_yaw() {
    let mut s = start();
    let before = s;
    apply_orbit_input(&mut s, &drag(50.0, 0.0));

    assert!((s.yaw - (before.yaw - 50.0 * ORBIT_SENSITIVITY)).abs() < EPS);
    assert_eq!(s.pitch, before.pitch);
    assert_eq!(s.distance, before.distance);

    apply_orbit_input(&mut s, &drag(-100.0, 0.0));
    assert!(s.yaw > before.yaw, "dragging left turns the other way");
}

#[test]
fn a_vertical_drag_changes_only_the_pitch() {
    let mut s = start();
    let before = s;
    apply_orbit_input(&mut s, &drag(0.0, 40.0));

    assert!((s.pitch - (before.pitch + 40.0 * ORBIT_SENSITIVITY)).abs() < EPS);
    assert_eq!(s.yaw, before.yaw);
    assert!(
        s.position().y > before.position().y,
        "dragging down raises the eye"
    );
}

#[test]
fn pitch_stays_limited_however_far_you_drag() {
    let mut s = start();
    for _ in 0..100 {
        apply_orbit_input(&mut s, &drag(0.0, 200.0));
    }
    assert!((s.pitch - MAX_PITCH).abs() < EPS);
    for _ in 0..200 {
        apply_orbit_input(&mut s, &drag(0.0, -200.0));
    }
    assert!((s.pitch - MIN_PITCH).abs() < EPS);
}

#[test]
fn the_wheel_zooms_in_and_out_proportionately() {
    let mut s = start();
    let d0 = s.distance;

    apply_orbit_input(&mut s, &wheel(1.0));
    assert!((s.distance - d0 * (1.0 - ZOOM_SENSITIVITY)).abs() < 1e-4);
    assert!(s.distance < d0);

    apply_orbit_input(&mut s, &wheel(-1.0));
    apply_orbit_input(&mut s, &wheel(-1.0));
    assert!(s.distance > d0 * 0.99);
}

#[test]
fn distance_stays_limited_and_the_eye_never_reaches_the_target() {
    let mut s = start();
    for _ in 0..200 {
        apply_orbit_input(&mut s, &wheel(4.0));
    }
    assert!((s.distance - MIN_DISTANCE).abs() < EPS);
    assert!((s.position() - s.target).length() > 0.5);

    for _ in 0..200 {
        apply_orbit_input(&mut s, &wheel(-4.0));
    }
    assert!((s.distance - MAX_DISTANCE).abs() < EPS);
}

#[test]
fn reset_restores_yaw_pitch_distance_and_target() {
    let mut s = start();
    let initial = s;
    apply_orbit_input(&mut s, &drag(80.0, -30.0));
    apply_orbit_input(&mut s, &wheel(3.0));
    s.set_target(Vec3::new(9.0, 9.0, 9.0));
    assert_ne!(s, initial);

    apply_orbit_input(
        &mut s,
        &OrbitInput {
            reset: true,
            drag_dx: 500.0,
            wheel: 3.0,
            ..OrbitInput::default()
        },
    );
    assert_eq!(s, initial, "reset wins over the rest of the frame");
}

#[test]
fn no_input_changes_nothing() {
    let mut s = start();
    let before = s;
    let idle = OrbitInput::default();

    assert!(!idle.is_active());
    apply_orbit_input(&mut s, &idle);
    assert_eq!(s, before);

    // A key held with zero elapsed time is not movement either.
    let held_no_time = OrbitInput {
        key_yaw: 1.0,
        ..OrbitInput::default()
    };
    apply_orbit_input(&mut s, &held_no_time);
    assert_eq!(s, before);
    assert!(drag(1.0, 0.0).is_active() && wheel(1.0).is_active());
}

#[test]
fn a_long_input_sequence_keeps_every_value_finite_and_in_range() {
    let mut s = start();
    for i in 0..2000 {
        let t = i as f32;
        apply_orbit_input(
            &mut s,
            &OrbitInput {
                drag_dx: (t * 0.37).sin() * 300.0,
                drag_dy: (t * 0.11).cos() * 300.0,
                wheel: (t * 0.05).sin() * 5.0,
                key_yaw: (t * 0.2).sin(),
                key_pitch: (t * 0.3).cos(),
                delta_time: 0.016,
                reset: false,
            },
        );
        assert!(s.yaw.is_finite() && s.pitch.is_finite() && s.distance.is_finite());
        assert!((MIN_PITCH..=MAX_PITCH).contains(&s.pitch));
        assert!((MIN_DISTANCE..=MAX_DISTANCE).contains(&s.distance));
        let p = s.position();
        assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    }
}

#[test]
fn non_finite_and_spiking_input_is_harmless() {
    let mut s = start();
    let before = s;
    apply_orbit_input(
        &mut s,
        &OrbitInput {
            drag_dx: f32::NAN,
            drag_dy: f32::INFINITY,
            wheel: f32::NAN,
            key_yaw: f32::NAN,
            delta_time: f32::NAN,
            ..OrbitInput::default()
        },
    );
    assert_eq!(s, before);

    apply_orbit_input(&mut s, &drag(1.0e9, 0.0));
    let turned = (s.yaw - before.yaw).abs();
    assert!(turned <= MAX_DRAG_PIXELS_PER_FRAME * ORBIT_SENSITIVITY + EPS);
}

#[test]
fn orbiting_never_moves_the_target() {
    let mut s = start();
    let target = s.target;
    for i in 0..50 {
        apply_orbit_input(&mut s, &drag(i as f32, 30.0 - i as f32));
    }
    assert_eq!(s.target, target);
}

#[test]
fn zooming_never_changes_yaw_or_pitch() {
    let mut s = start();
    let (yaw, pitch) = (s.yaw, s.pitch);
    for n in [1.0, 2.0, -1.0, 3.0, -4.0] {
        apply_orbit_input(&mut s, &wheel(n));
    }
    assert_eq!((s.yaw, s.pitch), (yaw, pitch));
}

#[test]
fn the_arrow_key_fallback_orbits_at_a_frame_rate_independent_speed() {
    let step = |dt: f32| {
        let mut s = start();
        // One second of holding right, as 1 big step or many small ones.
        let frames = (1.0 / dt).round() as usize;
        for _ in 0..frames {
            apply_orbit_input(
                &mut s,
                &OrbitInput {
                    key_yaw: 1.0,
                    delta_time: dt,
                    ..OrbitInput::default()
                },
            );
        }
        s.yaw
    };

    assert!((step(0.5) - step(0.01)).abs() < 1e-3);
}
