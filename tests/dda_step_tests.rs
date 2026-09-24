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
use renderer::voxel_traversal::{DdaState, DdaStep};
use std::collections::HashSet;

const EPS: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn make_ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn cells_after(origin: (f32, f32, f32), direction: (f32, f32, f32), steps: usize) -> Vec<IVec3> {
    let mut state = DdaState::from_ray(&make_ray(origin, direction));
    (0..steps)
        .map(|_| state.advance().expect("active ray must keep stepping").cell)
        .collect()
}

fn c(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

// ---------------------------------------------------------------------
// Simple axes
// ---------------------------------------------------------------------

#[test]
fn plus_x_walks_cell_by_cell() {
    let mut state = DdaState::from_ray(&make_ray((0.5, 0.5, 0.5), (1.0, 0.0, 0.0)));
    assert_eq!(state.cell, c(0, 0, 0));

    let first = state.advance().unwrap();
    assert_eq!(first.previous_cell, c(0, 0, 0));
    assert_eq!(first.cell, c(1, 0, 0));
    assert!(approx(first.t_enter, 0.5));

    let second = state.advance().unwrap();
    assert_eq!(second.previous_cell, c(1, 0, 0));
    assert_eq!(second.cell, c(2, 0, 0));
    assert!(approx(second.t_enter, 1.5));
}

#[test]
fn minus_x_walks_cell_by_cell() {
    let cells = cells_after((0.5, 0.5, 0.5), (-1.0, 0.0, 0.0), 2);
    assert_eq!(cells, vec![c(-1, 0, 0), c(-2, 0, 0)]);
}

#[test]
fn plus_and_minus_y_walk_cell_by_cell() {
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (0.0, 1.0, 0.0), 2),
        vec![c(0, 1, 0), c(0, 2, 0)]
    );
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (0.0, -1.0, 0.0), 2),
        vec![c(0, -1, 0), c(0, -2, 0)]
    );
}

#[test]
fn plus_and_minus_z_walk_cell_by_cell() {
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (0.0, 0.0, 1.0), 2),
        vec![c(0, 0, 1), c(0, 0, 2)]
    );
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (0.0, 0.0, -1.0), 2),
        vec![c(0, 0, -1), c(0, 0, -2)]
    );
}

#[test]
fn origin_on_a_boundary_moving_negative_crosses_at_zero_then_progresses() {
    let mut state = DdaState::from_ray(&make_ray((1.0, 0.5, 0.5), (-1.0, 0.0, 0.0)));

    let first = state.advance().unwrap();
    assert_eq!(first.previous_cell, c(1, 0, 0));
    assert_eq!(first.cell, c(0, 0, 0));
    assert_eq!(first.t_enter, 0.0);

    let second = state.advance().unwrap();
    assert_eq!(second.cell, c(-1, 0, 0));
    assert!(approx(second.t_enter, 1.0));
}

// ---------------------------------------------------------------------
// Non-symmetric diagonal (no accidental ties): compare against an
// independent analytic oracle that evaluates every boundary crossing
// directly instead of accumulating `t_delta`.
// ---------------------------------------------------------------------

fn oracle_cells(origin: (f32, f32, f32), direction: (f32, f32, f32), count: usize) -> Vec<IVec3> {
    let ray = make_ray(origin, direction);
    let o = [ray.origin.x, ray.origin.y, ray.origin.z];
    let d = [ray.direction.x, ray.direction.y, ray.direction.z];

    let mut events: Vec<(f32, usize, i32)> = Vec::new();
    for axis in 0..3 {
        if d[axis] == 0.0 {
            continue;
        }
        let sign = if d[axis] > 0.0 { 1 } else { -1 };
        for n in 1..=(count as i32 + 2) {
            let boundary = if sign > 0 {
                o[axis].floor() + n as f32
            } else {
                o[axis].floor() - (n as f32 - 1.0)
            };
            let t = (boundary - o[axis]) / d[axis];
            if t > 0.0 {
                events.push((t, axis, sign));
            }
        }
    }
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut cell = [
        o[0].floor() as i32,
        o[1].floor() as i32,
        o[2].floor() as i32,
    ];
    let mut out = Vec::new();
    for (_, axis, sign) in events.into_iter().take(count) {
        cell[axis] += sign;
        out.push(IVec3::new(cell[0], cell[1], cell[2]));
    }
    out
}

#[test]
fn asymmetric_diagonal_matches_the_analytic_oracle() {
    let origin = (0.3, 0.5, 0.7);
    let direction = (1.0, 0.4, 0.2);
    assert_eq!(
        cells_after(origin, direction, 40),
        oracle_cells(origin, direction, 40)
    );
}

#[test]
fn asymmetric_mixed_sign_direction_matches_the_analytic_oracle() {
    let origin = (0.35, 0.65, 0.15);
    let direction = (0.9, -0.55, 0.3);
    assert_eq!(
        cells_after(origin, direction, 40),
        oracle_cells(origin, direction, 40)
    );
}

#[test]
fn negative_origin_diagonal_matches_the_analytic_oracle() {
    let origin = (-2.3, -0.4, -5.1);
    let direction = (0.6, 0.45, -0.8);
    assert_eq!(
        cells_after(origin, direction, 40),
        oracle_cells(origin, direction, 40)
    );
}

// ---------------------------------------------------------------------
// Tie policy: simultaneous boundaries advance together.
// ---------------------------------------------------------------------

#[test]
fn double_tie_xy_moves_diagonally_without_a_phantom_side_cell() {
    let mut state = DdaState::from_ray(&make_ray((0.5, 0.5, 0.5), (1.0, 1.0, 0.0)));

    let step = state.advance().unwrap();
    assert_eq!(step.previous_cell, c(0, 0, 0));
    assert_eq!(step.cell, c(1, 1, 0));
    assert_ne!(step.cell, c(1, 0, 0));
    assert_ne!(step.cell, c(0, 1, 0));

    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (1.0, 1.0, 0.0), 4),
        vec![c(1, 1, 0), c(2, 2, 0), c(3, 3, 0), c(4, 4, 0)]
    );
}

#[test]
fn double_tie_in_every_axis_pair_is_symmetric() {
    // The result must not depend on which axes are tied or on axis order.
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (1.0, 0.0, 1.0), 2),
        vec![c(1, 0, 1), c(2, 0, 2)]
    );
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (0.0, 1.0, 1.0), 2),
        vec![c(0, 1, 1), c(0, 2, 2)]
    );
}

#[test]
fn triple_tie_moves_along_the_space_diagonal() {
    let mut state = DdaState::from_ray(&make_ray((0.5, 0.5, 0.5), (1.0, 1.0, 1.0)));
    let step = state.advance().unwrap();

    assert_eq!(step.previous_cell, c(0, 0, 0));
    assert_eq!(step.cell, c(1, 1, 1));

    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (1.0, 1.0, 1.0), 3),
        vec![c(1, 1, 1), c(2, 2, 2), c(3, 3, 3)]
    );
}

#[test]
fn mixed_sign_triple_tie() {
    assert_eq!(
        cells_after((0.5, 0.5, 0.5), (1.0, -1.0, 1.0), 3),
        vec![c(1, -1, 1), c(2, -2, 2), c(3, -3, 3)]
    );
}

#[test]
fn a_tie_between_two_axes_does_not_drag_a_slower_third_axis() {
    // x and y tie every step; z (direction 0.25) crosses much later.
    let cells = cells_after((0.5, 0.5, 0.5), (1.0, 1.0, 0.25), 6);
    assert_eq!(cells[0], c(1, 1, 0));
    assert_eq!(cells[1], c(2, 2, 0));
    // z eventually advances alone, as its own crossing.
    assert!(cells.iter().any(|cell| cell.z == 1));
}

#[test]
fn accumulated_float_error_does_not_split_an_exact_tie() {
    // Direction (1, 2, 0) from the origin: x boundaries at s = 1, 2, 3, ...
    // and y boundaries at s = 0.5, 1, 1.5, ...; every second y crossing
    // coincides exactly with an x crossing, yet the two t_max values are
    // built from different sequences of f32 additions.
    let cells = cells_after((0.0, 0.0, 0.5), (1.0, 2.0, 0.0), 100);
    for (i, cell) in cells.iter().enumerate() {
        let n = i as i32 + 1;
        // Odd steps are y-only crossings; even steps are the x/y ties.
        assert_eq!(*cell, c(n / 2, n, 0), "step {n}");
    }
}

// ---------------------------------------------------------------------
// Parallel axes and degenerate rays
// ---------------------------------------------------------------------

#[test]
fn a_parallel_axis_never_changes() {
    let mut state = DdaState::from_ray(&make_ray((0.2, 3.5, -4.5), (1.0, 0.0, 0.0)));
    for _ in 0..300 {
        let step = state.advance().unwrap();
        assert_eq!(step.cell.y, 3);
        assert_eq!(step.cell.z, -5);
    }
}

#[test]
fn two_parallel_axes_never_change() {
    let mut state = DdaState::from_ray(&make_ray((0.5, 0.5, 0.5), (0.0, -1.0, 0.0)));
    for _ in 0..200 {
        let step = state.advance().unwrap();
        assert_eq!(step.cell.x, 0);
        assert_eq!(step.cell.z, 0);
    }
}

#[test]
fn a_ray_with_no_active_axis_cannot_advance_and_is_left_untouched() {
    let mut state = DdaState::from_ray(&make_ray((0.5, 0.5, 0.5), (0.0, 0.0, 0.0)));
    let before = state;

    assert_eq!(state.advance(), None);
    assert_eq!(state, before);
}

// ---------------------------------------------------------------------
// Stability over hundreds of steps
// ---------------------------------------------------------------------

#[test]
fn long_walks_stay_finite_monotonic_and_never_repeat_a_cell() {
    let cases = [
        ((0.3, 0.5, 0.7), (1.0, 0.4, 0.2)),
        ((-3.2, 1.1, 0.4), (-0.7, 0.2, 0.9)),
        ((0.5, 0.5, 0.5), (1.0, 1.0, 1.0)),
        ((0.5, 0.5, 0.5), (1.0, -1.0, 0.0)),
        ((1.0, 2.0, 3.0), (-0.3, -0.6, -0.1)),
        ((0.25, 0.75, 0.5), (0.0, 1.0, 0.05)),
    ];

    for (origin, direction) in cases {
        let mut state = DdaState::from_ray(&make_ray(origin, direction));
        let mut seen: HashSet<(i32, i32, i32)> = HashSet::new();
        seen.insert((state.cell.x, state.cell.y, state.cell.z));
        let mut last_t = 0.0_f32;

        for _ in 0..500 {
            let DdaStep {
                previous_cell,
                cell,
                t_enter,
            } = state.advance().unwrap();

            assert_ne!(previous_cell, cell);
            assert!(t_enter.is_finite());
            assert!(t_enter >= last_t, "t went backwards: {t_enter} < {last_t}");
            assert!(
                seen.insert((cell.x, cell.y, cell.z)),
                "cell {cell:?} repeated for {origin:?} -> {direction:?}"
            );
            for value in [state.t_max.x, state.t_max.y, state.t_max.z] {
                assert!(!value.is_nan());
            }
            last_t = t_enter;
        }
    }
}

#[test]
fn every_step_changes_each_moved_axis_by_exactly_one_cell() {
    let mut state = DdaState::from_ray(&make_ray((0.3, 0.5, 0.7), (1.0, 0.4, 0.2)));
    for _ in 0..200 {
        let step = state.advance().unwrap();
        let dx = (step.cell.x - step.previous_cell.x).abs();
        let dy = (step.cell.y - step.previous_cell.y).abs();
        let dz = (step.cell.z - step.previous_cell.z).abs();
        assert!(dx <= 1 && dy <= 1 && dz <= 1);
        assert!(dx + dy + dz >= 1);
    }
}
