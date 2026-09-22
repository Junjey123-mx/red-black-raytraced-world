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
        let uv = self.uv_for_face(point, face);
        Some(HitRecord::new(t, point, face, uv))
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

    /// Normalizes a single world-space coordinate against `[min, max]` into
    /// `[0, 1]`, absorbing small floating-point overshoot at the boundary.
    /// A degenerate (near-zero) extent falls back to the midpoint rather
    /// than dividing by (near-)zero.
    fn local_coordinate(value: f32, min: f32, max: f32) -> f32 {
        let extent = max - min;
        if extent.abs() <= f32::EPSILON {
            0.5
        } else {
            ((value - min) / extent).clamp(0.0, 1.0)
        }
    }

    /// Maps a boundary `point` on the given `face` to the canonical
    /// normalized UV coordinate frozen for this project: `u = 0` is the left
    /// edge of the texture, `v = 0` is the top edge, matching a row-major
    /// image stored top-to-bottom. Each face is parameterized independently
    /// from the cube's local `(lx, ly, lz)` coordinates so a translated cube
    /// produces identical UVs to an equivalent cube at the origin.
    fn uv_for_face(&self, point: Vec3, face: Face) -> Vec2 {
        let lx = Self::local_coordinate(point.x, self.aabb.min.x, self.aabb.max.x);
        let ly = Self::local_coordinate(point.y, self.aabb.min.y, self.aabb.max.y);
        let lz = Self::local_coordinate(point.z, self.aabb.min.z, self.aabb.max.z);

        match face {
            Face::PositiveZ => Vec2::new(lx, 1.0 - ly),
            Face::NegativeZ => Vec2::new(1.0 - lx, 1.0 - ly),
            Face::PositiveX => Vec2::new(1.0 - lz, 1.0 - ly),
            Face::NegativeX => Vec2::new(lz, 1.0 - ly),
            Face::PositiveY => Vec2::new(lx, lz),
            Face::NegativeY => Vec2::new(lx, 1.0 - lz),
        }
    }
}
