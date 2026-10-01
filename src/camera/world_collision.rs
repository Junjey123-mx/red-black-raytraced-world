//! Local voxel collision for the World free-fly camera.
//!
//! The camera is a small sphere. Every movement is resolved against only
//! the cells that sphere can overlap (at most a 2x2x2 block of cells for a
//! radius under half a cell), never against the whole `VoxelWorld`. A move
//! that would push the sphere into solid block geometry is resolved per
//! axis, so the free components survive and the camera slides along walls
//! instead of stopping dead. Long moves are cut into substeps no longer
//! than the radius, so a slow frame cannot tunnel through a wall.
//!
//! Solidity is decided per block by `is_camera_solid`; the exact block
//! geometry (full cubes, stair halves, thin doors, the portal membrane) is
//! what the sphere is tested against, not the whole cell.

// `overlapped_cells` and the config constructors are the test-facing API.
#![allow(dead_code)]

use crate::core::math::{IVec3, Vec3};
use crate::core::prism::Prism;
use crate::scene::block::BlockInstance;
use crate::scene::block_geometry::BlockGeometry;
use crate::scene::block_shape_factory::block_geometry;
use crate::scene::block_type::BlockType;
use crate::scene::voxel_world::VoxelWorld;

/// Radius of the camera's collision sphere, in blocks: small enough for
/// one-cell corridors, stair pits and the two-cell doorway, large enough
/// that the eye never touches a face.
pub const CAMERA_COLLISION_RADIUS: f32 = 0.25;

/// Longest movement resolved in one piece. Below the radius, so a wall one
/// cell thick can never be jumped in a single step.
pub const COLLISION_SUBSTEP: f32 = 0.2;

/// Tiny gap kept between the sphere and any surface it is pushed against,
/// so a resolved position is never exactly touching.
pub const COLLISION_SKIN: f32 = 1e-4;

/// The camera passability contract: every placed block stops the camera
/// except the ones it must be able to go through. The portal membrane
/// (`PortalCoreDarkCrimson`) stays visible, raytraced, emissive and the
/// crossing trigger, but the camera passes it; both door variants (the
/// Overworld `WoodDoor` and its Red-Black twin `RedBlackWoodDoor`) stay
/// visible and closed, but the camera enters through them. The portal
/// frame, walls, glass, fences, terrain and every other block remain solid.
pub fn is_camera_passable(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::PortalCoreDarkCrimson | BlockType::WoodDoor | BlockType::RedBlackWoodDoor
    )
}

/// Whether the camera may not pass through a placed block (see
/// `is_camera_passable`).
pub fn is_camera_solid(block: &BlockInstance) -> bool {
    !is_camera_passable(block.block_type())
}

/// What a noclip toggle did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoclipToggle {
    /// Collision is now bypassed.
    Enabled,
    /// Collision is back on.
    Disabled,
    /// Collision could not be re-enabled: the camera is embedded in a
    /// solid block and must first fly out to a clear position.
    RefusedInsideSolid,
}

/// The camera's collision switch. Collision is on by default; `N` toggles
/// noclip, which bypasses collision only (portal crossing, realm change,
/// roll and sky transition keep working). Noclip cannot be switched off
/// while the camera sits inside a solid block, so it is never trapped and
/// never teleported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollisionState {
    noclip: bool,
}

impl CollisionState {
    /// Collision on, noclip off.
    pub fn new() -> Self {
        Self { noclip: false }
    }

    pub fn is_noclip(&self) -> bool {
        self.noclip
    }

    pub fn collision_enabled(&self) -> bool {
        !self.noclip
    }

    /// Toggles noclip. Turning it off is refused while `position` is not
    /// clear of solid blocks.
    pub fn toggle_noclip(
        &mut self,
        world: &VoxelWorld,
        position: Vec3,
        config: &CameraCollisionConfig,
    ) -> NoclipToggle {
        if !self.noclip {
            self.noclip = true;
            NoclipToggle::Enabled
        } else if is_position_clear(world, position, config, &is_camera_solid) {
            self.noclip = false;
            NoclipToggle::Disabled
        } else {
            NoclipToggle::RefusedInsideSolid
        }
    }

    /// `R`: collision on, noclip off (the reset pose is always clear).
    pub fn reset(&mut self) {
        self.noclip = false;
    }

    /// The HUD suffix while noclip is active.
    pub fn hud_suffix(&self) -> &'static str {
        if self.noclip { " | Noclip: ON" } else { "" }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraCollisionConfig {
    pub radius: f32,
    pub substep: f32,
}

impl Default for CameraCollisionConfig {
    fn default() -> Self {
        Self {
            radius: CAMERA_COLLISION_RADIUS,
            substep: COLLISION_SUBSTEP,
        }
    }
}

impl CameraCollisionConfig {
    pub fn with_radius(radius: f32) -> Self {
        Self {
            radius: radius.max(0.0),
            substep: COLLISION_SUBSTEP.min(radius.max(1e-3)),
        }
    }
}

/// The sphere around `position`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraCollisionVolume {
    pub center: Vec3,
    pub radius: f32,
}

impl CameraCollisionVolume {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// The cells this sphere can overlap: the integer range covered by
    /// its bounding box (8 cells for a radius below 0.5).
    pub fn overlapped_cells(&self) -> impl Iterator<Item = IVec3> {
        let lo = self.center - Vec3::new(self.radius, self.radius, self.radius);
        let hi = self.center + Vec3::new(self.radius, self.radius, self.radius);
        let (x0, y0, z0) = (
            lo.x.floor() as i32,
            lo.y.floor() as i32,
            lo.z.floor() as i32,
        );
        let (x1, y1, z1) = (
            hi.x.floor() as i32,
            hi.y.floor() as i32,
            hi.z.floor() as i32,
        );
        (x0..=x1).flat_map(move |x| {
            (y0..=y1).flat_map(move |y| (z0..=z1).map(move |z| IVec3::new(x, y, z)))
        })
    }

    /// Squared distance from the center to the nearest point of the box.
    fn distance_squared_to_box(&self, min: Vec3, max: Vec3) -> f32 {
        let c = self.center;
        let d = Vec3::new(
            (min.x - c.x).max(0.0).max(c.x - max.x),
            (min.y - c.y).max(0.0).max(c.y - max.y),
            (min.z - c.z).max(0.0).max(c.z - max.z),
        );
        d.length_squared()
    }

    /// Whether the sphere penetrates `prism` (touching is not penetrating).
    pub fn overlaps_prism(&self, prism: &Prism) -> bool {
        self.distance_squared_to_box(prism.min(), prism.max()) < self.radius * self.radius
    }

    /// Whether the sphere penetrates the geometry of `block` placed at `cell`.
    pub fn overlaps_block(&self, cell: IVec3, block: &BlockInstance) -> bool {
        let origin = Vec3::new(cell.x as f32, cell.y as f32, cell.z as f32);
        match block_geometry(block.block_type(), block.orientation()) {
            BlockGeometry::FullCube => {
                self.distance_squared_to_box(origin, origin + Vec3::new(1.0, 1.0, 1.0))
                    < self.radius * self.radius
            }
            geometry => geometry
                .parts()
                .iter()
                .any(|part| self.overlaps_prism(&part.translated(origin))),
        }
    }
}

/// Whether a camera at `position` is free of every solid block, checking
/// only the cells its sphere can overlap.
pub fn is_position_clear(
    world: &VoxelWorld,
    position: Vec3,
    config: &CameraCollisionConfig,
    solid: &dyn Fn(&BlockInstance) -> bool,
) -> bool {
    let volume = CameraCollisionVolume::new(position, config.radius);
    volume.overlapped_cells().all(|cell| match world.get(cell) {
        Some(block) if solid(block) => !volume.overlaps_block(cell, block),
        _ => true,
    })
}

/// Moves the camera from `from` by `delta` as far as the solid blocks allow.
/// The move is cut into substeps of at most `config.substep`; each substep
/// is tried whole, then one axis at a time (largest component first), so
/// the components that are not blocked are kept and the camera slides.
/// A camera that already starts inside a solid is left free to move (it
/// can always leave), since blocking it there would trap it.
pub fn resolve_camera_motion(
    world: &VoxelWorld,
    from: Vec3,
    delta: Vec3,
    config: &CameraCollisionConfig,
    solid: &dyn Fn(&BlockInstance) -> bool,
) -> Vec3 {
    let length = delta.length();
    if !length.is_finite() || length <= 0.0 {
        return from;
    }
    if !is_position_clear(world, from, config, solid) {
        return from + delta;
    }
    let steps = (length / config.substep.max(1e-4)).ceil().max(1.0) as usize;
    let step = delta * (1.0 / steps as f32);
    let mut position = from;
    for _ in 0..steps {
        position = resolve_step(world, position, step, config, solid);
    }
    position
}

fn resolve_step(
    world: &VoxelWorld,
    from: Vec3,
    step: Vec3,
    config: &CameraCollisionConfig,
    solid: &dyn Fn(&BlockInstance) -> bool,
) -> Vec3 {
    let whole = from + step;
    if is_position_clear(world, whole, config, solid) {
        return whole;
    }
    // Axis by axis, largest component first, keeping what fits.
    let mut axes = [
        (step.x.abs(), Vec3::new(step.x, 0.0, 0.0)),
        (step.y.abs(), Vec3::new(0.0, step.y, 0.0)),
        (step.z.abs(), Vec3::new(0.0, 0.0, step.z)),
    ];
    axes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut position = from;
    for (magnitude, component) in axes {
        if magnitude <= 0.0 {
            continue;
        }
        let candidate = position + component;
        if is_position_clear(world, candidate, config, solid) {
            position = candidate;
        } else {
            position = slide_to_contact(world, position, component, config, solid);
        }
    }
    position
}

/// Advances along `component` as far as it stays clear (bisection), so a
/// blocked axis still closes the gap up to the surface minus the skin.
fn slide_to_contact(
    world: &VoxelWorld,
    from: Vec3,
    component: Vec3,
    config: &CameraCollisionConfig,
    solid: &dyn Fn(&BlockInstance) -> bool,
) -> Vec3 {
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..8 {
        let mid = 0.5 * (lo + hi);
        if is_position_clear(world, from + component * mid, config, solid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let travel = (lo * component.length() - COLLISION_SKIN).max(0.0);
    if travel <= 0.0 {
        from
    } else {
        from + component.normalize() * travel
    }
}
