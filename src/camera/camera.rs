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
}
