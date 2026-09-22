// Foundation-stage primitive: lands ahead of the Cube intersection and
// raytracer work that will produce and consume hit records.
#![allow(dead_code)]

use crate::core::math::{Vec2, Vec3};

/// One of the six axis-aligned cube faces, each with a canonical outward
/// normal. Deciding which face applies at an edge or corner (where more
/// than one slab boundary is reached simultaneously) is the responsibility
/// of the intersection routine that constructs a `HitRecord`, not of this
/// type; `HitRecord` only stores whichever face that routine picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl Face {
    pub fn normal(self) -> Vec3 {
        match self {
            Face::PositiveX => Vec3::new(1.0, 0.0, 0.0),
            Face::NegativeX => Vec3::new(-1.0, 0.0, 0.0),
            Face::PositiveY => Vec3::new(0.0, 1.0, 0.0),
            Face::NegativeY => Vec3::new(0.0, -1.0, 0.0),
            Face::PositiveZ => Vec3::new(0.0, 0.0, 1.0),
            Face::NegativeZ => Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

/// Geometric result of a ray intersection: distance along the ray, world
/// point, outward unit normal, the face it belongs to, and the texture-space
/// UV coordinate at that point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitRecord {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub face: Face,
    pub uv: Vec2,
}

impl HitRecord {
    /// The normal is derived from `face` so it is always unitary and
    /// consistent with the reported face.
    pub fn new(distance: f32, point: Vec3, face: Face, uv: Vec2) -> Self {
        Self {
            distance,
            point,
            normal: face.normal(),
            face,
            uv,
        }
    }
}
