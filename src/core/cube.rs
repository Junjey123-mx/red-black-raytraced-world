// Foundation-stage primitive: lands ahead of the scene/raytracer work that
// will place and cast rays against cubes.
#![allow(dead_code)]

use crate::core::aabb::Aabb;
use crate::core::hit::{Face, HitRecord};
use crate::core::math::{Vec2, Vec3};
use crate::core::ray::Ray;

/// Axis-aligned Cube, geometrically an `Aabb` with cube-specific hit
/// resolution (nearest face selection) layered on top. Intersection reuses
/// the tested slab algorithm instead of duplicating it.
pub struct Cube {
    aabb: Aabb,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            aabb: Aabb::new(min, max),
        }
    }

    fn contains(&self, point: Vec3) -> bool {
        point.x >= self.aabb.min.x
            && point.x <= self.aabb.max.x
            && point.y >= self.aabb.min.y
            && point.y <= self.aabb.max.y
            && point.z >= self.aabb.min.z
            && point.z <= self.aabb.max.z
    }

    /// Returns the nearest valid positive hit. When the ray originates
    /// inside the cube, the entry distance produced by the slab test is not
    /// a real surface (it is clipped to `t_min`), so the exit distance is
    /// reported instead.
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let (t_enter, t_exit) = self.aabb.intersect(ray, t_min, t_max)?;

        let t = if self.contains(ray.origin) {
            t_exit
        } else {
            t_enter
        };

        if t <= t_min {
            return None;
        }

        let point = ray.at(t);
        let face = self.face_for_point(point);
        // Bridging value: real per-face UV mapping lands in the next commit.
        Some(HitRecord::new(t, point, face, Vec2::zero()))
    }

    /// Classifies which face `point` (assumed to lie on the cube boundary)
    /// belongs to by nearest distance to each of the six bounding planes.
    /// Ties (edges and corners, where multiple planes are equidistant) are
    /// broken by a fixed priority: -X, +X, -Y, +Y, -Z, +Z, so the result is
    /// deterministic for every call with the same point.
    fn face_for_point(&self, point: Vec3) -> Face {
        let candidates = [
            ((point.x - self.aabb.min.x).abs(), Face::NegativeX),
            ((point.x - self.aabb.max.x).abs(), Face::PositiveX),
            ((point.y - self.aabb.min.y).abs(), Face::NegativeY),
            ((point.y - self.aabb.max.y).abs(), Face::PositiveY),
            ((point.z - self.aabb.min.z).abs(), Face::NegativeZ),
            ((point.z - self.aabb.max.z).abs(), Face::PositiveZ),
        ];

        let mut best = candidates[0];
        for candidate in &candidates[1..] {
            if candidate.0 < best.0 {
                best = *candidate;
            }
        }
        best.1
    }
}
