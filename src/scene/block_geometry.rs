// Scene-stage model: lands ahead of the local-intersection work that will
// let a cell's shape decide whether a ray really hits it.
#![allow(dead_code)]

use crate::core::hit::{Face, HitRecord};
use crate::core::prism::Prism;
use crate::core::ray::Ray;

/// Two hits closer than this are treated as the same distance, so the
/// winner is chosen by a fixed face order instead of by the order of the
/// parts (which must never change what is visible).
const TIE_DISTANCE_EPSILON: f32 = 1e-6;

fn face_rank(face: Face) -> u8 {
    match face {
        Face::NegativeX => 0,
        Face::PositiveX => 1,
        Face::NegativeY => 2,
        Face::PositiveY => 3,
        Face::NegativeZ => 4,
        Face::PositiveZ => 5,
    }
}

/// `true` when `candidate` should replace `best`: strictly nearer, or a
/// distance tie resolved by the fixed face order.
fn is_better_hit(candidate: &HitRecord, best: &HitRecord) -> bool {
    let difference = candidate.distance - best.distance;
    if difference.abs() <= TIE_DISTANCE_EPSILON {
        face_rank(candidate.face) < face_rank(best.face)
    } else {
        difference < 0.0
    }
}

/// Which family a `BlockGeometry` belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryKind {
    FullCube,
    Prism,
    Composite,
}

/// The local *shape* of a block inside its unit cell `[0, 1]^3`. It carries
/// no material, texture, block type, or world position: those belong to the
/// `BlockInstance` and the `VoxelWorld` key.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockGeometry {
    /// The whole unit cell.
    FullCube,
    /// A single box.
    Prism(Prism),
    /// Several boxes acting as one shape. Never empty; build it through
    /// `BlockGeometry::composite`.
    Composite(Vec<Prism>),
}

impl BlockGeometry {
    pub fn prism(prism: Prism) -> Self {
        BlockGeometry::Prism(prism)
    }

    /// Builds a composite shape, or `None` when `parts` is empty (a shape
    /// with no volume is not a valid geometry).
    pub fn composite(parts: Vec<Prism>) -> Option<Self> {
        if parts.is_empty() {
            None
        } else {
            Some(BlockGeometry::Composite(parts))
        }
    }

    pub fn kind(&self) -> GeometryKind {
        match self {
            BlockGeometry::FullCube => GeometryKind::FullCube,
            BlockGeometry::Prism(_) => GeometryKind::Prism,
            BlockGeometry::Composite(_) => GeometryKind::Composite,
        }
    }

    /// Number of prismatic parts (`FullCube` counts as one).
    pub fn part_count(&self) -> usize {
        match self {
            BlockGeometry::FullCube | BlockGeometry::Prism(_) => 1,
            BlockGeometry::Composite(parts) => parts.len(),
        }
    }

    /// The prismatic parts of this shape in local cell space; a `FullCube`
    /// yields the unit prism.
    pub fn parts(&self) -> Vec<Prism> {
        match self {
            BlockGeometry::FullCube => vec![Prism::unit()],
            BlockGeometry::Prism(prism) => vec![*prism],
            BlockGeometry::Composite(parts) => parts.clone(),
        }
    }

    pub fn is_full_cube(&self) -> bool {
        matches!(self, BlockGeometry::FullCube)
    }

    /// Nearest valid hit of `ray` against this shape, with the ray given in
    /// the same space as the geometry (local cell space for a freshly built
    /// shape). `FullCube` and `Prism` go through `Prism::intersect`
    /// (`Cube::intersect` underneath); a `Composite` intersects every part
    /// and keeps the hit with the smallest distance, so the order of the
    /// parts never changes the result.
    pub fn intersect_local(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            BlockGeometry::FullCube => Prism::unit().intersect(ray, t_min, t_max),
            BlockGeometry::Prism(prism) => prism.intersect(ray, t_min, t_max),
            BlockGeometry::Composite(parts) => {
                let mut best: Option<HitRecord> = None;
                for part in parts {
                    if let Some(hit) = part.intersect(ray, t_min, t_max)
                        && best.as_ref().is_none_or(|b| is_better_hit(&hit, b))
                    {
                        best = Some(hit);
                    }
                }
                best
            }
        }
    }
}
