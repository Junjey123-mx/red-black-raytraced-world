// Scene-stage container: lands ahead of the DDA/raycast work that will query
// it cell by cell.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::core::math::{IVec3, Vec3};
use crate::scene::block::BlockInstance;

/// The axis-aligned box that contains every occupied cell of a
/// `VoxelWorld`: `min` is the lowest occupied cell and `max_exclusive` is one
/// past the highest, so the box spans `[min, max_exclusive)` in world units
/// (a cell `c` occupies `[c, c + 1)`). Maintained by the world itself, so a
/// ray can be clipped against it without visiting a single cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelBounds {
    pub min: IVec3,
    pub max_exclusive: IVec3,
}

/// Slack added past the box exit so a ray that grazes a corner still visits
/// the last cell it touches inside the box (cells beyond the box are empty,
/// so the slack can never add a hit).
pub const BOUNDS_EXIT_SLACK: f32 = 1e-3;

impl VoxelBounds {
    /// The box of a single cell.
    pub fn of_cell(cell: IVec3) -> Self {
        Self {
            min: cell,
            max_exclusive: IVec3::new(cell.x + 1, cell.y + 1, cell.z + 1),
        }
    }

    /// The smallest box containing both `self` and `cell`.
    pub fn including(self, cell: IVec3) -> Self {
        Self {
            min: IVec3::new(
                self.min.x.min(cell.x),
                self.min.y.min(cell.y),
                self.min.z.min(cell.z),
            ),
            max_exclusive: IVec3::new(
                self.max_exclusive.x.max(cell.x + 1),
                self.max_exclusive.y.max(cell.y + 1),
                self.max_exclusive.z.max(cell.z + 1),
            ),
        }
    }

    /// Whether `cell` lies on one of the six faces of the box (removing it
    /// may shrink the box).
    pub fn touches_face(&self, cell: IVec3) -> bool {
        cell.x == self.min.x
            || cell.y == self.min.y
            || cell.z == self.min.z
            || cell.x + 1 == self.max_exclusive.x
            || cell.y + 1 == self.max_exclusive.y
            || cell.z + 1 == self.max_exclusive.z
    }

    pub fn contains_cell(&self, cell: IVec3) -> bool {
        cell.x >= self.min.x
            && cell.y >= self.min.y
            && cell.z >= self.min.z
            && cell.x < self.max_exclusive.x
            && cell.y < self.max_exclusive.y
            && cell.z < self.max_exclusive.z
    }

    pub fn min_corner(&self) -> Vec3 {
        Vec3::new(self.min.x as f32, self.min.y as f32, self.min.z as f32)
    }

    pub fn max_corner(&self) -> Vec3 {
        Vec3::new(
            self.max_exclusive.x as f32,
            self.max_exclusive.y as f32,
            self.max_exclusive.z as f32,
        )
    }

    /// Size in cells along each axis.
    pub fn extent(&self) -> IVec3 {
        IVec3::new(
            self.max_exclusive.x - self.min.x,
            self.max_exclusive.y - self.min.y,
            self.max_exclusive.z - self.min.z,
        )
    }

    /// Slab test of the ray `origin + direction * t` (`t >= 0`) against the
    /// box: the ray parameter at which it leaves the box, or `None` when the
    /// ray never enters it. The entry parameter is `0` for an origin inside
    /// the box. A direction component of exactly `0` keeps the ray in its
    /// slab only if the origin already is; otherwise the ray misses.
    pub fn exit_distance(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let lo = self.min_corner();
        let hi = self.max_corner();
        let (mut t_enter, mut t_exit) = (0.0_f32, f32::INFINITY);
        for (o, d, lo, hi) in [
            (origin.x, direction.x, lo.x, hi.x),
            (origin.y, direction.y, lo.y, hi.y),
            (origin.z, direction.z, lo.z, hi.z),
        ] {
            if d == 0.0 {
                if o < lo || o > hi {
                    return None;
                }
                continue;
            }
            let inv = 1.0 / d;
            let (near, far) = ((lo - o) * inv, (hi - o) * inv);
            let (near, far) = if near <= far {
                (near, far)
            } else {
                (far, near)
            };
            t_enter = t_enter.max(near);
            t_exit = t_exit.min(far);
            if t_enter > t_exit {
                return None;
            }
        }
        if !t_exit.is_finite() || t_exit < 0.0 {
            return None;
        }
        Some(t_exit)
    }

    /// The traversal limit for a ray that may reach at most `max_distance`:
    /// `None` when the ray misses the box entirely, otherwise the smaller of
    /// `max_distance` and the box exit (plus `BOUNDS_EXIT_SLACK`).
    pub fn clip_distance(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<f32> {
        let exit = self.exit_distance(origin, direction)?;
        Some(max_distance.min(exit + BOUNDS_EXIT_SLACK))
    }
}

/// Sparse voxel storage: only occupied cells exist, keyed by their integer
/// cell coordinate (which may be negative or arbitrarily far apart). There
/// is no dense grid and no global bounds; the extent of any diorama comes
/// from how a scene is built, not from this container.
#[derive(Debug, Default)]
pub struct VoxelWorld {
    cells: HashMap<IVec3, BlockInstance>,
    /// The box of every occupied cell, kept exact on every insert and
    /// remove (`None` for an empty world); never recomputed per query.
    bounds: Option<VoxelBounds>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            bounds: None,
        }
    }

    /// The exact box of the occupied cells, or `None` when empty.
    pub fn bounds(&self) -> Option<VoxelBounds> {
        self.bounds
    }

    /// Every occupied cell with its block, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (&IVec3, &BlockInstance)> {
        self.cells.iter()
    }

    fn recompute_bounds(&mut self) {
        self.bounds = self
            .cells
            .keys()
            .fold(None, |bounds: Option<VoxelBounds>, cell| {
                Some(match bounds {
                    Some(b) => b.including(*cell),
                    None => VoxelBounds::of_cell(*cell),
                })
            });
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
        let previous = self.cells.insert(position, block);
        self.bounds = Some(match self.bounds {
            Some(bounds) => bounds.including(position),
            None => VoxelBounds::of_cell(position),
        });
        previous
    }

    /// Empties the cell at `position`, returning the removed instance if
    /// the cell was occupied.
    pub fn remove(&mut self, position: IVec3) -> Option<BlockInstance> {
        let removed = self.cells.remove(&position);
        if removed.is_some()
            && let Some(bounds) = self.bounds
            && bounds.touches_face(position)
        {
            // Only a cell on a face of the box can shrink it.
            self.recompute_bounds();
        }
        removed
    }
}
