// Scene-stage factory: resolves the local shape of a block from its type
// and orientation. Block types without a partial shape are full cubes.
#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::core::prism::Prism;
use crate::scene::block_geometry::BlockGeometry;
use crate::scene::block_type::BlockType;
use crate::scene::geometry_orientation::orient_geometry;
use crate::scene::orientation::Orientation;

fn prism(min: [f32; 3], max: [f32; 3]) -> Prism {
    Prism::new(
        Vec3::new(min[0], min[1], min[2]),
        Vec3::new(max[0], max[1], max[2]),
    )
}

fn composite(parts: Vec<Prism>) -> BlockGeometry {
    BlockGeometry::composite(parts).expect("canonical shapes always have at least one part")
}

/// Stairs facing South (+Z), upright: a full-footprint lower half plus an
/// upper half over the back (north) half of the cell.
fn wood_stairs() -> BlockGeometry {
    composite(vec![
        prism([0.0, 0.0, 0.0], [1.0, 0.5, 1.0]),
        prism([0.0, 0.5, 0.0], [1.0, 1.0, 0.5]),
    ])
}

/// The shape of `block_type` in its canonical pose (facing South, upright).
pub fn canonical_geometry(block_type: BlockType) -> BlockGeometry {
    match block_type {
        BlockType::WoodStairs => wood_stairs(),
        _ => BlockGeometry::FullCube,
    }
}

/// The local shape of a block of `block_type` placed with `orientation`,
/// in unit-cell space `[0, 1]^3`.
pub fn block_geometry(block_type: BlockType, orientation: Orientation) -> BlockGeometry {
    orient_geometry(&canonical_geometry(block_type), orientation)
}
