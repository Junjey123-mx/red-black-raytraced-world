// Scene-stage model: lands ahead of the local-intersection work that will
// let a cell's shape decide whether a ray really hits it.
#![allow(dead_code)]

use crate::core::prism::Prism;

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
}
