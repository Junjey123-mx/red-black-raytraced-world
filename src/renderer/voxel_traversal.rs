// Renderer-stage math: lands ahead of the Ray -> DDA initialization and
// stepping work that will drive it.
#![allow(dead_code)]

use crate::core::math::{IVec3, Vec3};

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
}
