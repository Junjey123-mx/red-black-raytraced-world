// Foundation-stage primitive: lands ahead of the projection/render work
// that will construct and consume the camera.
#![allow(dead_code)]

use crate::core::math::Vec3;

const MIN_FOV_DEGREES: f32 = 1.0;
const MAX_FOV_DEGREES: f32 = 179.0;
const MIN_ASPECT_RATIO: f32 = 1e-4;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
}

/// Orthonormal viewing basis derived from a `Camera`: `forward` points from
/// the eye to the target, `right` and `up` complete a right-handed frame.
pub struct CameraBasis {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    /// `fov` (vertical, degrees) and `aspect_ratio` are clamped to a safe
    /// finite range so degenerate input cannot propagate NaN/Infinity.
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, aspect_ratio: f32) -> Self {
        Self {
            position,
            target,
            up,
            fov: fov.clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES),
            aspect_ratio: aspect_ratio.max(MIN_ASPECT_RATIO),
        }
    }

    /// Derives `forward = normalize(target - position)`,
    /// `right = normalize(forward x up)`, `true_up = normalize(right x forward)`.
    /// Falls back to an alternate reference up when `up` is (near-)parallel
    /// to `forward`, so the basis never collapses to NaN/zero vectors.
    pub fn basis(&self) -> CameraBasis {
        let forward = (self.target - self.position).normalize();

        let mut right = forward.cross(self.up).normalize();
        if right.length_squared() <= f32::EPSILON {
            let fallback_up = if forward.x.abs() < 0.99 {
                Vec3::new(1.0, 0.0, 0.0)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            right = forward.cross(fallback_up).normalize();
        }

        let true_up = right.cross(forward).normalize();

        CameraBasis {
            forward,
            right,
            up: true_up,
        }
    }
}
