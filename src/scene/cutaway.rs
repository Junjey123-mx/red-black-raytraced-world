// The physical cutaway of the diorama: the south-east quadrant of the
// diamond simply does not exist (above and below the shelf), so the
// geological layers, the descent and the portal are exposed to the camera
// like a sectioned scale model. No material is made transparent and no
// face is culled: removed cells are removed.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::rhombus::{CutawayRegion, RhombusConfig};
use crate::scene::terrain::TerrainConfig;
use crate::scene::voxel_world::VoxelWorld;

/// What the cutaway took out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cutaway {
    pub region: CutawayRegion,
    pub shelf_y: i32,
    /// Every cell removed, in scan order.
    pub removed: Vec<IVec3>,
}

impl Cutaway {
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }
}

/// Removes every existing cell of the cutaway region (`RhombusConfig::is_cut`)
/// from `world`. The shelf row is kept: it is the cut's floor and the
/// portal's sill.
pub fn carve_cutaway(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    world: &mut VoxelWorld,
) -> Cutaway {
    let mut cutaway = Cutaway {
        region: rhombus.cutaway,
        shelf_y: rhombus.shelf_y,
        removed: Vec::new(),
    };
    let top = terrain.max_surface_height() + 16;
    for y in (rhombus.lower_tip_y..=top).rev() {
        for z in rhombus.cutaway.min_z..terrain.depth + 4 {
            for x in rhombus.cutaway.min_x..terrain.width + 4 {
                if !rhombus.is_cut(x, y, z) {
                    continue;
                }
                let cell = IVec3::new(x, y, z);
                if world.remove(cell).is_some() {
                    cutaway.removed.push(cell);
                }
            }
        }
    }
    cutaway
}
