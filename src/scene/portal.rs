// The portal at the waist: the red-obsidian frame set into the cutaway's
// north wall and, once activated, the dark-crimson membrane in its opening.
// Both reuse the certified catalog blocks and material identities; nothing
// here defines a material of its own.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::math::{IVec3, Vec3};
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::light::{Light, PointLight};
use crate::scene::material_gallery::portal_core_material_id;
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
    /// Dark-crimson membrane cells filling the opening.
    pub core: Vec<IVec3>,
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

/// Fills the opening with the thin `PortalCoreDarkCrimson` membrane, facing
/// the cutaway like the frame, so adjacent cells tile into one continuous
/// energetic surface inside the frame.
pub fn build_portal_core(rhombus: &RhombusConfig, world: &mut VoxelWorld, build: &mut PortalBuild) {
    for cell in build.opening.clone() {
        world.insert(
            cell,
            BlockInstance::new(
                BlockType::PortalCoreDarkCrimson,
                portal_core_material_id(),
                rhombus.portal.facing,
            ),
        );
        build.core.push(cell);
    }
}

/// Color of the portal's light: cornelian crimson, the same tone the
/// catalog gives the core's representative light.
pub fn portal_light_color() -> Color {
    Color::new(0.75, 0.10, 0.30, 1.0)
}

/// Intensity of the portal's light: a third of the sun, so the membrane
/// tints the deepslate and stairs around the waist without competing with
/// daylight above.
pub const PORTAL_LIGHT_INTENSITY: f32 = 0.35;

/// World-space position of the portal's light: the middle of the opening,
/// one block in front of the membrane on the cutaway side.
pub fn portal_light_position(rhombus: &RhombusConfig) -> Vec3 {
    let p = rhombus.portal;
    Vec3::new(
        (p.min_x + p.max_x + 1) as f32 / 2.0,
        (p.min_y + p.max_y + 1) as f32 / 2.0,
        p.wall_z as f32 + 1.5,
    )
}

/// The portal's representative point light.
pub fn portal_light(rhombus: &RhombusConfig) -> Light {
    Light::Point(PointLight::new(
        portal_light_position(rhombus),
        portal_light_color(),
        PORTAL_LIGHT_INTENSITY,
    ))
}
