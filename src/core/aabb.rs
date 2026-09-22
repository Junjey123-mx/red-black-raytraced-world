// Foundation-stage primitive: lands ahead of the ray-intersection and cube
// work that will consume it.
#![allow(dead_code)]

use crate::core::math::Vec3;

/// Axis-aligned bounding box. The constructor normalizes its two input
/// corners component-wise so `min <= max` always holds, regardless of the
/// order the caller passes them in.
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(a: Vec3, b: Vec3) -> Self {
        Self {
            min: Vec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Vec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extent(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
}
