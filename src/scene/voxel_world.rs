// Scene-stage container: lands ahead of the DDA/raycast work that will query
// it cell by cell.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;

/// Sparse voxel storage: only occupied cells exist, keyed by their integer
/// cell coordinate (which may be negative or arbitrarily far apart). There
/// is no dense grid and no global bounds; the extent of any diorama comes
/// from how a scene is built, not from this container.
#[derive(Debug, Default)]
pub struct VoxelWorld {
    cells: HashMap<IVec3, BlockInstance>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    /// Number of occupied cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The block stored at `position`, or `None` for an empty cell.
    pub fn get(&self, position: IVec3) -> Option<&BlockInstance> {
        self.cells.get(&position)
    }

    pub fn contains(&self, position: IVec3) -> bool {
        self.cells.contains_key(&position)
    }

    /// Stores `block` at `position`. An empty cell becomes occupied (`len`
    /// grows by one); an occupied cell is replaced (`len` is unchanged) and
    /// the previous instance is returned.
    pub fn insert(&mut self, position: IVec3, block: BlockInstance) -> Option<BlockInstance> {
        self.cells.insert(position, block)
    }

    /// Empties the cell at `position`, returning the removed instance if
    /// the cell was occupied.
    pub fn remove(&mut self, position: IVec3) -> Option<BlockInstance> {
        self.cells.remove(&position)
    }
}
