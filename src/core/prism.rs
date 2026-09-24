// Geometry-stage primitive: lands ahead of the BlockGeometry work that will
// compose several prisms into one partial voxel shape.
#![allow(dead_code)]

use crate::core::aabb::Aabb;
use crate::core::cube::Cube;
use crate::core::hit::HitRecord;
use crate::core::math::Vec3;
use crate::core::ray::Ray;

/// Axis-aligned box defined by two corners. Unlike `Cube` it is meant to be
/// *any* orthogonal box, typically living inside the local `[0, 1]^3` space
/// of a voxel cell (a half-height slab, a thin post, a narrow door leaf).
/// Intersection reuses `Cube::intersect`, so distance, point, normal, face
/// and UV follow exactly the same contract as a full cube; UVs are
/// normalized against this prism's own extents.
///
/// A prism needs a positive extent on every axis to be hittable: a
/// zero-thickness box is a valid value but never produces a hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prism {
    min: Vec3,
    max: Vec3,
}

impl Prism {
    /// The two corners are normalized component-wise so `min <= max`.
    pub fn new(a: Vec3, b: Vec3) -> Self {
        let aabb = Aabb::new(a, b);
        Self {
            min: aabb.min,
            max: aabb.max,
        }
    }

    /// The full unit cell `[0, 1]^3`.
    pub fn unit() -> Self {
        Self::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0))
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.min, self.max)
    }

    /// The same prism shifted by `offset`.
    pub fn translated(&self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// `true` when the whole prism lies inside the unit cell `[0, 1]^3`
    /// (within a small tolerance for accumulated float error).
    pub fn is_within_unit_cell(&self) -> bool {
        const TOLERANCE: f32 = 1e-5;
        let lo = -TOLERANCE;
        let hi = 1.0 + TOLERANCE;
        [
            self.min.x, self.min.y, self.min.z, self.max.x, self.max.y, self.max.z,
        ]
        .iter()
        .all(|&value| value >= lo && value <= hi)
    }

    /// Nearest valid positive hit within `[t_min, t_max]`, delegating to
    /// `Cube::intersect` (slab test, exit distance when the ray starts
    /// inside).
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        Cube::new(self.min, self.max).intersect(ray, t_min, t_max)
    }
}
