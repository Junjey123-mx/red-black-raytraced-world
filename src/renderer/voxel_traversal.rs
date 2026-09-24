// Renderer-stage math: lands ahead of the Ray -> DDA initialization and
// stepping work that will drive it.
#![allow(dead_code)]

use crate::core::math::{IVec3, Vec3};
use crate::core::ray::Ray;

/// Ray-direction components whose magnitude is at or below this value are
/// treated as parallel to that axis's voxel planes (the axis is inactive).
pub const AXIS_EPSILON: f32 = 1e-6;

/// Per-axis advance direction: `+1` for a clearly positive component, `-1`
/// for a clearly negative one, and `0` when the component is within
/// `AXIS_EPSILON` of zero (or not a number), so an inactive axis can never
/// be chosen to advance.
pub fn axis_step(direction_component: f32) -> i32 {
    if direction_component > AXIS_EPSILON {
        1
    } else if direction_component < -AXIS_EPSILON {
        -1
    } else {
        0
    }
}

/// `|1 / direction_component|` for an active axis: the parametric distance
/// between two successive voxel boundaries along that axis. An inactive
/// axis returns `+INFINITY` on purpose (never `NaN`, never a division by
/// zero), meaning "this axis never reaches another boundary".
pub fn safe_reciprocal_magnitude(direction_component: f32) -> f32 {
    if axis_step(direction_component) == 0 {
        f32::INFINITY
    } else {
        (1.0 / direction_component).abs()
    }
}

/// The voxel cell containing `point`, using `floor` semantics so negative
/// coordinates land in the correct cell (`-0.1 -> -1`, `-1.0 -> -1`,
/// `-1.2 -> -2`) instead of truncating toward zero.
pub fn cell_from_point(point: Vec3) -> IVec3 {
    IVec3::new(
        point.x.floor() as i32,
        point.y.floor() as i32,
        point.z.floor() as i32,
    )
}

/// Ray parameter at which the ray first reaches a voxel boundary on one
/// axis, or `+INFINITY` for an inactive axis.
///
/// For `step > 0` the boundary is `cell + 1`; for `step < 0` it is `cell`
/// itself (the low edge). When the origin already lies exactly on the
/// boundary being approached (only possible for `step < 0`, e.g. origin
/// `x = 1.0` moving toward `-X` from cell `1`), the distance is exactly
/// `0.0`: the first step crosses into the neighboring cell at `t = 0`, and
/// every later crossing is spaced by `t_delta`, so the walk always makes
/// forward progress and cannot loop at zero distance. The result is never
/// negative and never `NaN`.
fn first_boundary_distance(origin: f32, direction: f32, cell: i32, step: i32) -> f32 {
    if step == 0 {
        return f32::INFINITY;
    }

    let boundary = (if step > 0 { cell + 1 } else { cell }) as f32;
    let distance = (boundary - origin) / direction;
    if distance > 0.0 { distance } else { 0.0 }
}

/// Relative tolerance used to decide that two axes reach their next voxel
/// boundary "at the same time". Boundary distances are accumulated with
/// repeated `f32` additions, so a mathematically exact tie (for example a
/// ray along `(1, 2, 0)` crossing `x` and `y` boundaries together) can
/// differ by a few ulps; without a tolerance it would be split into two
/// steps and invent a cell the ray only touches along an edge or corner.
/// The tolerance is `TIE_EPSILON * max(t, 1.0)`.
pub const TIE_EPSILON: f32 = 1e-6;

/// One DDA advance: the cell the walk left, the cell it entered, and the
/// ray parameter at which the new cell is entered. `t_enter` never
/// decreases from one advance to the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DdaStep {
    pub previous_cell: IVec3,
    pub cell: IVec3,
    pub t_enter: f32,
}

/// Mathematical state of a 3D DDA voxel traversal. It only describes where
/// the walk currently is and how it will advance; it knows nothing about a
/// world, blocks, materials, or textures.
///
/// - `cell`: the cell the ray is currently inside.
/// - `step`: per-axis advance direction (`+1`, `-1`, or `0` for an
///   inactive axis).
/// - `t_max`: ray parameter at which the ray reaches the next boundary on
///   each axis (`+INFINITY` for an inactive axis).
/// - `t_delta`: ray-parameter distance between successive boundaries on
///   each axis (`+INFINITY` for an inactive axis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DdaState {
    pub cell: IVec3,
    pub step: IVec3,
    pub t_max: Vec3,
    pub t_delta: Vec3,
}

impl DdaState {
    pub fn new(cell: IVec3, step: IVec3, t_max: Vec3, t_delta: Vec3) -> Self {
        Self {
            cell,
            step,
            t_max,
            t_delta,
        }
    }

    /// Initializes traversal from a (normalized) `Ray`: the starting cell
    /// is the `floor` cell of the origin, `step` follows the sign of each
    /// direction component, `t_max` is the distance to the first boundary
    /// on each axis, and `t_delta` is the boundary-to-boundary spacing.
    /// Axes parallel to the voxel planes are inactive (`step = 0`,
    /// `t_max = t_delta = +INFINITY`).
    pub fn from_ray(ray: &Ray) -> Self {
        let origin = ray.origin;
        let direction = ray.direction;

        let cell = cell_from_point(origin);
        let step = IVec3::new(
            axis_step(direction.x),
            axis_step(direction.y),
            axis_step(direction.z),
        );

        let t_max = Vec3::new(
            first_boundary_distance(origin.x, direction.x, cell.x, step.x),
            first_boundary_distance(origin.y, direction.y, cell.y, step.y),
            first_boundary_distance(origin.z, direction.z, cell.z, step.z),
        );
        let t_delta = Vec3::new(
            safe_reciprocal_magnitude(direction.x),
            safe_reciprocal_magnitude(direction.y),
            safe_reciprocal_magnitude(direction.z),
        );

        Self::new(cell, step, t_max, t_delta)
    }

    /// Advances to the next voxel cell in increasing ray distance.
    ///
    /// **Tie policy:** every active axis whose `t_max` equals the minimum
    /// (within `TIE_EPSILON`) is advanced in the *same* step. A ray that
    /// reaches an x boundary and a y boundary together therefore moves
    /// diagonally to the cell it actually enters, instead of first visiting
    /// a side cell that it only touches at an edge (or corner, for a triple
    /// tie). The outcome never depends on the order of the axes.
    ///
    /// An axis with `step == 0` is never advanced. Returns `None` (and
    /// leaves the state untouched) when no axis is active, i.e. the ray has
    /// a zero direction and can never leave its cell.
    pub fn advance(&mut self) -> Option<DdaStep> {
        let steps = [self.step.x, self.step.y, self.step.z];
        let t_max = [self.t_max.x, self.t_max.y, self.t_max.z];

        let t_enter = (0..3)
            .filter(|&axis| steps[axis] != 0)
            .map(|axis| t_max[axis])
            .fold(f32::INFINITY, f32::min);
        if !t_enter.is_finite() {
            return None;
        }

        let threshold = t_enter + TIE_EPSILON * t_enter.max(1.0);
        let previous_cell = self.cell;
        let mut cell = [self.cell.x, self.cell.y, self.cell.z];
        let mut next_t_max = t_max;
        let t_delta = [self.t_delta.x, self.t_delta.y, self.t_delta.z];

        for axis in 0..3 {
            if steps[axis] != 0 && t_max[axis] <= threshold {
                cell[axis] += steps[axis];
                next_t_max[axis] += t_delta[axis];
            }
        }

        self.cell = IVec3::new(cell[0], cell[1], cell[2]);
        self.t_max = Vec3::new(next_t_max[0], next_t_max[1], next_t_max[2]);

        Some(DdaStep {
            previous_cell,
            cell: self.cell,
            t_enter,
        })
    }
}
