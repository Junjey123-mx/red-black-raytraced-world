// The portal at the waist: the red-obsidian frame set into the cutaway's
// north wall and, once activated, the dark-crimson membrane in its opening.
// Both reuse the certified catalog blocks and material identities; nothing
// here defines a material of its own.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::portal_frame_material_id;
use crate::scene::rhombus::RhombusConfig;
use crate::scene::voxel_world::VoxelWorld;

/// The portal as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PortalBuild {
    /// Red-obsidian frame cells.
    pub frame: Vec<IVec3>,
    /// The cells of the opening (left empty by the frame stage).
    pub opening: Vec<IVec3>,
}

/// Sets the frame into the waist wall: every anchor cell that is not part
/// of the opening becomes `PortalFrameRedObsidian` (replacing the deepslate
/// of the wall), and the opening is cleared so the frame really is a frame.
pub fn build_portal_frame(rhombus: &RhombusConfig, world: &mut VoxelWorld) -> PortalBuild {
    let p = rhombus.portal;
    let mut build = PortalBuild::default();
    for y in p.min_y..=p.max_y {
        for x in p.min_x..=p.max_x {
            let cell = IVec3::new(x, y, p.wall_z);
            if p.is_frame(x, y) {
                world.insert(
                    cell,
                    BlockInstance::new(
                        BlockType::PortalFrameRedObsidian,
                        portal_frame_material_id(),
                        Orientation::Up,
                    ),
                );
                build.frame.push(cell);
            } else {
                world.remove(cell);
                build.opening.push(cell);
            }
        }
    }
    build
}
