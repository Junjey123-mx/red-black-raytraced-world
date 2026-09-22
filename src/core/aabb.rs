// Foundation-stage primitive: lands ahead of the ray-intersection and cube
// work that will consume it.
#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::core::ray::Ray;

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

    /// Slab-method ray/AABB intersection. Returns the clipped `(t_enter,
    /// t_exit)` interval within the caller-provided `[t_min, t_max]` bound,
    /// or `None` on a miss. Axis-parallel directions are handled explicitly
    /// (rather than relying on IEEE division by zero) so the result never
    /// contains `NaN`.
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(f32, f32)> {
        let (t_min, t_max) = slab(
            t_min,
            t_max,
            ray.origin.x,
            ray.direction.x,
            self.min.x,
            self.max.x,
        )?;
        let (t_min, t_max) = slab(
            t_min,
            t_max,
            ray.origin.y,
            ray.direction.y,
            self.min.y,
            self.max.y,
        )?;
        let (t_min, t_max) = slab(
            t_min,
            t_max,
            ray.origin.z,
            ray.direction.z,
            self.min.z,
            self.max.z,
        )?;
        Some((t_min, t_max))
    }
}

/// Clips `[t_min, t_max]` against a single axis slab `[min, max]`.
fn slab(
    t_min: f32,
    t_max: f32,
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
) -> Option<(f32, f32)> {
    if direction.abs() < f32::EPSILON {
        if origin < min || origin > max {
            return None;
        }
        return Some((t_min, t_max));
    }

    let inv_direction = 1.0 / direction;
    let mut t0 = (min - origin) * inv_direction;
    let mut t1 = (max - origin) * inv_direction;
    if inv_direction < 0.0 {
        std::mem::swap(&mut t0, &mut t1);
    }

    let new_min = t_min.max(t0);
    let new_max = t_max.min(t1);

    if new_max <= new_min {
        None
    } else {
        Some((new_min, new_max))
    }
}
