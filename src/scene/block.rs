// Scene-stage record: lands ahead of the VoxelWorld work that will store one
// instance per occupied cell.
#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;

/// The lightweight unit stored per voxel cell: which block it is, which
/// material it references, and how it is oriented. Position is deliberately
/// absent (it is the `IVec3` key of the future `VoxelWorld`), and so is any
/// `Material`, `Cube`, `Aabb`, texture, light, or traversal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockInstance {
    block_type: BlockType,
    material_id: MaterialId,
    orientation: Orientation,
}

impl BlockInstance {
    pub fn new(block_type: BlockType, material_id: MaterialId, orientation: Orientation) -> Self {
        Self {
            block_type,
            material_id,
            orientation,
        }
    }

    pub fn block_type(&self) -> BlockType {
        self.block_type
    }

    pub fn material_id(&self) -> MaterialId {
        self.material_id
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}
