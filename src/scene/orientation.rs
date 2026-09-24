// Scene-stage identifier: lands ahead of the BlockInstance/VoxelWorld work
// that will store one orientation per voxel.
#![allow(dead_code)]

use crate::core::math::Vec3;

/// Explicit block orientation. The single axis convention for the whole
/// project (matching `Face`'s +/-X, +/-Y, +/-Z normals) is:
///
/// ```text
/// Up    -> +Y      East  -> +X
/// Down  -> -Y      West  -> -X
/// North -> -Z      South -> +Z
/// ```
///
/// `Down` is what lets the inverted Red-Black half of the diorama be
/// expressed later without a second convention. This type only names a
/// direction: it performs no geometry rotation, UV transform, or matrix
/// work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl Orientation {
    /// Unit vector of the world axis this orientation points along.
    pub fn axis_direction(self) -> Vec3 {
        match self {
            Orientation::Up => Vec3::new(0.0, 1.0, 0.0),
            Orientation::Down => Vec3::new(0.0, -1.0, 0.0),
            Orientation::North => Vec3::new(0.0, 0.0, -1.0),
            Orientation::South => Vec3::new(0.0, 0.0, 1.0),
            Orientation::East => Vec3::new(1.0, 0.0, 0.0),
            Orientation::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }

    /// `true` for `Up` and `Down`, `false` for the four cardinal directions.
    pub fn is_vertical(self) -> bool {
        matches!(self, Orientation::Up | Orientation::Down)
    }
}
