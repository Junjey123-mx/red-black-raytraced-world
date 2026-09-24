#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::voxel_traversal::DdaState;

const EPS: f32 = 1e-5;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn state(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> DdaState {
    DdaState::from_ray(&ray(origin, direction))
}

fn assert_no_nan(state: &DdaState) {
    for value in [
        state.t_max.x,
        state.t_max.y,
        state.t_max.z,
        state.t_delta.x,
        state.t_delta.y,
        state.t_delta.z,
    ] {
        assert!(!value.is_nan(), "state contains NaN: {state:?}");
    }
}

// ---------------------------------------------------------------------
// Axis-aligned rays
// ---------------------------------------------------------------------

#[test]
fn ray_plus_x() {
    let s = state((0.2, 0.5, 0.5), (1.0, 0.0, 0.0));
    assert_eq!(s.cell, IVec3::new(0, 0, 0));
    assert_eq!(s.step, IVec3::new(1, 0, 0));
    assert!(approx(s.t_max.x, 0.8));
    assert!(s.t_max.y.is_infinite() && s.t_max.z.is_infinite());
    assert!(approx(s.t_delta.x, 1.0));
    assert!(s.t_delta.y.is_infinite() && s.t_delta.z.is_infinite());
}

#[test]
fn ray_minus_x() {
    let s = state((0.25, 0.5, 0.5), (-1.0, 0.0, 0.0));
    assert_eq!(s.cell, IVec3::new(0, 0, 0));
    assert_eq!(s.step, IVec3::new(-1, 0, 0));
    assert!(approx(s.t_max.x, 0.25));
    assert!(approx(s.t_delta.x, 1.0));
}

#[test]
fn ray_plus_y() {
    let s = state((0.5, 0.4, 0.5), (0.0, 1.0, 0.0));
    assert_eq!(s.step, IVec3::new(0, 1, 0));
    assert!(approx(s.t_max.y, 0.6));
    assert!(approx(s.t_delta.y, 1.0));
    assert!(s.t_max.x.is_infinite() && s.t_max.z.is_infinite());
}

#[test]
fn ray_minus_y() {
    let s = state((0.5, 0.4, 0.5), (0.0, -1.0, 0.0));
    assert_eq!(s.step, IVec3::new(0, -1, 0));
    assert!(approx(s.t_max.y, 0.4));
    assert!(approx(s.t_delta.y, 1.0));
}

#[test]
fn ray_plus_z() {
    let s = state((0.5, 0.5, 0.9), (0.0, 0.0, 1.0));
    assert_eq!(s.step, IVec3::new(0, 0, 1));
    assert!(approx(s.t_max.z, 0.1));
    assert!(approx(s.t_delta.z, 1.0));
    assert!(s.t_max.x.is_infinite() && s.t_max.y.is_infinite());
}

#[test]
fn ray_minus_z() {
    let s = state((0.5, 0.5, 0.9), (0.0, 0.0, -1.0));
    assert_eq!(s.step, IVec3::new(0, 0, -1));
    assert!(approx(s.t_max.z, 0.9));
    assert!(approx(s.t_delta.z, 1.0));
}

// ---------------------------------------------------------------------
// Diagonal, mixed signs, negative origins
// ---------------------------------------------------------------------

#[test]
fn positive_diagonal() {
    let d = 1.0 / 3.0_f32.sqrt();
    let s = state((0.5, 0.5, 0.5), (1.0, 1.0, 1.0));

    assert_eq!(s.cell, IVec3::new(0, 0, 0));
    assert_eq!(s.step, IVec3::new(1, 1, 1));
    assert!(approx(s.t_max.x, 0.5 / d));
    assert!(approx(s.t_max.y, 0.5 / d));
    assert!(approx(s.t_max.z, 0.5 / d));
    assert!(approx(s.t_delta.x, 1.0 / d));
}

#[test]
fn diagonal_with_mixed_signs() {
    let d = 1.0 / 3.0_f32.sqrt();
    let s = state((0.25, 0.75, 0.5), (1.0, -1.0, 1.0));

    assert_eq!(s.cell, IVec3::new(0, 0, 0));
    assert_eq!(s.step, IVec3::new(1, -1, 1));
    assert!(approx(s.t_max.x, 0.75 / d)); // to x = 1
    assert!(approx(s.t_max.y, 0.75 / d)); // to y = 0
    assert!(approx(s.t_max.z, 0.5 / d)); // to z = 1
}

#[test]
fn negative_origin_uses_floor_for_the_starting_cell() {
    let d = 1.0 / 3.0_f32.sqrt();
    let s = state((-0.2, 0.7, 0.1), (1.0, 1.0, 1.0));

    assert_eq!(s.cell, IVec3::new(-1, 0, 0));
    assert!(approx(s.t_max.x, 0.2 / d)); // to x = 0
    assert!(approx(s.t_max.y, 0.3 / d)); // to y = 1
    assert!(approx(s.t_max.z, 0.9 / d)); // to z = 1
}

#[test]
fn negative_integer_origin_starts_in_its_own_floor_cell() {
    let d = 1.0 / 3.0_f32.sqrt();
    let s = state((-1.0, -1.0, -1.0), (1.0, 1.0, 1.0));

    assert_eq!(s.cell, IVec3::new(-1, -1, -1));
    // Next boundary on every axis is 0, one full cell away.
    assert!(approx(s.t_max.x, 1.0 / d));
    assert!(approx(s.t_max.y, 1.0 / d));
    assert!(approx(s.t_max.z, 1.0 / d));
}

#[test]
fn negative_origin_moving_negative_targets_the_low_edge() {
    let s = state((-0.25, 0.5, 0.5), (-1.0, 0.0, 0.0));

    assert_eq!(s.cell, IVec3::new(-1, 0, 0));
    assert_eq!(s.step, IVec3::new(-1, 0, 0));
    // Cell -1 spans [-1, 0): the next boundary toward -X is x = -1.
    assert!(approx(s.t_max.x, 0.75));
}

// ---------------------------------------------------------------------
// Origin exactly on a boundary
// ---------------------------------------------------------------------

#[test]
fn origin_on_boundary_moving_positive_starts_in_that_cell_and_targets_the_next() {
    let s = state((1.0, 0.5, 0.5), (1.0, 0.0, 0.0));

    assert_eq!(s.cell.x, 1);
    assert_eq!(s.step.x, 1);
    // Next boundary is x = 2, one full cell away.
    assert!(approx(s.t_max.x, 1.0));
}

#[test]
fn origin_on_boundary_moving_negative_crosses_immediately_without_looping() {
    let s = state((1.0, 0.5, 0.5), (-1.0, 0.0, 0.0));

    assert_eq!(s.cell.x, 1);
    assert_eq!(s.step.x, -1);
    // Documented policy: the boundary being approached is the origin
    // itself, so the first crossing (into cell x = 0) happens at t = 0...
    assert_eq!(s.t_max.x, 0.0);
    // ...and every later crossing is one full cell further, so a stepping
    // loop always advances.
    assert!(approx(s.t_delta.x, 1.0));
    assert!(s.t_delta.x > 0.0);
    assert_no_nan(&s);
}

#[test]
fn integer_origin_on_all_axes_is_finite_and_non_negative() {
    let s = state((2.0, -3.0, 0.0), (-1.0, 1.0, -1.0));
    assert_eq!(s.cell, IVec3::new(2, -3, 0));
    assert_no_nan(&s);
    assert!(s.t_max.x >= 0.0 && s.t_max.y >= 0.0 && s.t_max.z >= 0.0);
}

// ---------------------------------------------------------------------
// Parallel (inactive) axes
// ---------------------------------------------------------------------

#[test]
fn two_zero_components_leave_two_inactive_axes() {
    let s = state((0.5, 0.5, 0.5), (0.0, 0.0, 1.0));

    assert_eq!(s.step, IVec3::new(0, 0, 1));
    assert!(s.t_max.x.is_infinite() && s.t_max.y.is_infinite());
    assert!(s.t_delta.x.is_infinite() && s.t_delta.y.is_infinite());
    assert!(s.t_max.z.is_finite());
    assert_no_nan(&s);
}

#[test]
fn one_zero_component_leaves_one_inactive_axis() {
    let s = state((0.5, 0.5, 0.5), (1.0, 0.0, 1.0));

    assert_eq!(s.step, IVec3::new(1, 0, 1));
    assert!(s.t_max.y.is_infinite() && s.t_delta.y.is_infinite());
    assert!(s.t_max.x.is_finite() && s.t_max.z.is_finite());
    assert_no_nan(&s);
}

// ---------------------------------------------------------------------
// Numeric contracts across many directions
// ---------------------------------------------------------------------

fn sample_directions() -> Vec<(f32, f32, f32)> {
    let mut out = Vec::new();
    for x in [-1.0, -0.5, 0.0, 0.3, 1.0] {
        for y in [-1.0, -0.2, 0.0, 0.7, 1.0] {
            for z in [-1.0, 0.0, 0.4, 1.0] {
                out.push((x, y, z));
            }
        }
    }
    out
}

fn sample_origins() -> Vec<(f32, f32, f32)> {
    vec![
        (0.0, 0.0, 0.0),
        (0.2, 0.7, 0.1),
        (-0.2, 0.7, 0.1),
        (-1.0, -1.0, -1.0),
        (1.0, 2.0, 3.0),
        (-4.5, 2.25, 7.75),
        (0.999, -0.001, 5.0),
    ]
}

#[test]
fn no_nan_for_any_sampled_ray() {
    for origin in sample_origins() {
        for direction in sample_directions() {
            if direction == (0.0, 0.0, 0.0) {
                continue;
            }
            assert_no_nan(&state(origin, direction));
        }
    }
}

#[test]
fn degenerate_zero_direction_ray_produces_no_nan() {
    let s = state((0.5, 0.5, 0.5), (0.0, 0.0, 0.0));
    assert_eq!(s.step, IVec3::new(0, 0, 0));
    assert!(s.t_max.x.is_infinite() && s.t_max.y.is_infinite() && s.t_max.z.is_infinite());
    assert_no_nan(&s);
}

#[test]
fn t_delta_is_positive_for_every_active_axis() {
    for origin in sample_origins() {
        for direction in sample_directions() {
            if direction == (0.0, 0.0, 0.0) {
                continue;
            }
            let s = state(origin, direction);
            for (step, delta) in [
                (s.step.x, s.t_delta.x),
                (s.step.y, s.t_delta.y),
                (s.step.z, s.t_delta.z),
            ] {
                if step != 0 {
                    assert!(delta > 0.0 && delta.is_finite(), "{s:?}");
                } else {
                    assert!(delta.is_infinite(), "{s:?}");
                }
            }
        }
    }
}

#[test]
fn t_max_is_non_negative_for_the_first_valid_crossing_on_every_axis() {
    for origin in sample_origins() {
        for direction in sample_directions() {
            if direction == (0.0, 0.0, 0.0) {
                continue;
            }
            let s = state(origin, direction);
            for (step, t_max) in [
                (s.step.x, s.t_max.x),
                (s.step.y, s.t_max.y),
                (s.step.z, s.t_max.z),
            ] {
                assert!(t_max >= 0.0, "{s:?}");
                if step == 0 {
                    assert!(t_max.is_infinite(), "{s:?}");
                } else {
                    assert!(t_max.is_finite(), "{s:?}");
                }
            }
        }
    }
}

#[test]
fn initialization_is_deterministic() {
    let a = state((-0.3, 1.7, 2.2), (0.4, -0.8, 0.1));
    let b = state((-0.3, 1.7, 2.2), (0.4, -0.8, 0.1));
    assert_eq!(a, b);
}
