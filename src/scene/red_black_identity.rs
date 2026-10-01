//! Gate 17.5: the Red-Black world identity.
//!
//! The lower realm gains a material language of its own on top of the Gate
//! 17 fortress: crimson-canopy trees, corinto timber inside the fortress,
//! a timber path to a readable gatehouse with a Red-Black door, a fourth
//! tower and a little more landscape. Every stage works in the inverted
//! frame (functional up is `-Y`): ground cells are the lowest cell of their
//! column, things "grow" toward more negative `y`, and every block is
//! `Orientation::Down` unless its geometry demands a horizontal facing.

#![allow(dead_code)]

use std::collections::HashSet;

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::expansion::{WorldExpansionLayout, expansion_hash, lower_ground_cell};
use crate::scene::fortress::RedBlackFortressLayout;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::Tree;
use crate::scene::red_black_timber::{red_black_leaves_material_id, red_black_log_material_id};
use crate::scene::voxel_world::VoxelWorld;

const TREE_TRUNK_SALT: u32 = 0x175E_0001;
const TREE_CANOPY_SALT: u32 = 0x175E_0002;

/// Shortest trunk of a Red-Black tree (a little taller than the Overworld's).
pub const RED_BLACK_TRUNK_MIN: i32 = 5;
/// Tallest trunk.
pub const RED_BLACK_TRUNK_MAX: i32 = 6;
/// Fewest and most trees the realm carries.
pub const RED_BLACK_TREES_MIN: usize = 4;
pub const RED_BLACK_TREES_MAX: usize = 7;
/// Columns kept between any tree cell and a family focal cell.
pub const TREE_FOCAL_CLEARANCE: i32 = 2;
/// Columns between two trunks (Chebyshev distance).
pub const TREE_SPACING: i32 = 3;

fn down(block_type: BlockType, material: crate::core::material::MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Down)
}

/// The trunk sites of the first grove, derived from the fortress pad: one
/// beside the forecourt and one south of the gate (both off every sight
/// line from the approach to the gate) and two on the lobe's north ring
/// behind the fortress, clear of the towers' crests.
pub fn tree_sites(fortress: &RedBlackFortressLayout) -> Vec<(i32, i32)> {
    let f = fortress.footprint;
    vec![
        (f.min_x - 4, f.min_z + 5),
        (f.min_x - 1, f.max_z),
        (f.min_x + 3, f.min_z - 1),
        (f.min_x + 7, f.min_z - 1),
    ]
}

/// What a tree must never touch: cells and columns other stages own.
pub struct TreeClearance<'a> {
    /// Cells that must stay as they are (routes, focal scenes, fortress).
    pub cells: &'a HashSet<IVec3>,
    /// Columns people walk under (paths, courtyard, routes): no tree cell.
    pub walk_columns: &'a HashSet<(i32, i32)>,
    /// Family focal columns.
    pub focal_columns: &'a [(i32, i32)],
    /// The deepest `y` any cell may reach (the lower tip).
    pub floor_y: i32,
}

/// The crown of an inverted tree whose trunk ends at `end`: a ring around
/// the trunk's last two cells, a few hashed arms one further out, a plus
/// beyond the end and a hashed tip, all growing toward `-Y`.
fn inverted_canopy(seed: u32, base: IVec3, end: i32) -> Vec<IVec3> {
    let keep = |salt: i32, dx: i32, dz: i32, p: f32| {
        expansion_hash(
            seed,
            base.x * 7 + dx * 3 + salt,
            base.z * 11 + dz * 5,
            TREE_CANOPY_SALT,
        ) < p
    };
    let mut cells = Vec::new();
    for (dy, salt) in [(1, 0), (0, 1)] {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let corner = dx != 0 && dz != 0;
                if corner && dy == 1 && !keep(salt, dx, dz, 0.5) {
                    continue;
                }
                cells.push(IVec3::new(base.x + dx, end + dy, base.z + dz));
            }
        }
    }
    for (dx, dz) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
        if keep(2, dx, dz, 0.4) {
            cells.push(IVec3::new(base.x + dx, end, base.z + dz));
        }
    }
    cells.push(IVec3::new(base.x, end - 1, base.z));
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if keep(3, dx, dz, 0.7) {
            cells.push(IVec3::new(base.x + dx, end - 1, base.z + dz));
        }
    }
    if keep(4, 0, 0, 0.6) {
        cells.push(IVec3::new(base.x, end - 2, base.z));
    }
    cells
}

/// Grows one Red-Black tree hanging from the ground cell of `(x, z)`:
/// a `RedBlackLog` trunk toward `-Y`, then the inverted crown of
/// `RedBlackLeaves`. Returns `None` (placing nothing) when the column has
/// no ground, the trunk would cross an occupied or protected cell, or the
/// tree would reach past the lower tip. Leaves skip occupied, protected
/// and walk-column head-room cells, and keep clear of family focal columns.
pub fn grow_red_black_tree(
    seed: u32,
    x: i32,
    z: i32,
    clearance: &TreeClearance,
    world: &mut VoxelWorld,
) -> Option<Tree> {
    let base = lower_ground_cell(world, x, z, clearance.floor_y - 4, 0)?;
    let span = RED_BLACK_TRUNK_MAX - RED_BLACK_TRUNK_MIN + 1;
    let pick = (expansion_hash(seed, x, z, TREE_TRUNK_SALT) * span as f32) as i32;
    let mut height = RED_BLACK_TRUNK_MIN + pick.min(span - 1);
    // Fit inside the world: the crown reaches two cells past the trunk end.
    while height > 3 && base.y - height - 2 < clearance.floor_y {
        height -= 1;
    }
    if base.y - height - 2 < clearance.floor_y {
        return None;
    }
    let near_focal = |c: IVec3| {
        clearance.focal_columns.iter().any(|(fx, fz)| {
            (fx - c.x).abs() < TREE_FOCAL_CLEARANCE && (fz - c.z).abs() < TREE_FOCAL_CLEARANCE
        })
    };
    // Walk columns stay free from the ground down: no leaf hangs under a
    // path, so its paving remains the lowest cell of the column.
    let in_walk_column = |c: IVec3| clearance.walk_columns.contains(&(c.x, c.z));
    let trunk: Vec<IVec3> = (1..=height).map(|d| IVec3::new(x, base.y - d, z)).collect();
    if trunk.iter().any(|c| {
        world.contains(*c) || clearance.cells.contains(c) || near_focal(*c) || in_walk_column(*c)
    }) {
        return None;
    }
    for c in &trunk {
        world.insert(
            *c,
            down(BlockType::RedBlackLog, red_black_log_material_id()),
        );
    }
    let end = base.y - height;
    let mut leaves = Vec::new();
    for c in inverted_canopy(seed, base, end) {
        if world.contains(c)
            || clearance.cells.contains(&c)
            || near_focal(c)
            || in_walk_column(c)
            || c.y < clearance.floor_y
        {
            continue;
        }
        world.insert(
            c,
            down(BlockType::RedBlackLeaves, red_black_leaves_material_id()),
        );
        leaves.push(c);
    }
    Some(Tree {
        base,
        trunk_height: height,
        trunk,
        leaves,
    })
}

/// The Red-Black grove, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackGrove {
    pub trees: Vec<Tree>,
}

impl RedBlackGrove {
    pub fn cells(&self) -> Vec<IVec3> {
        self.trees
            .iter()
            .flat_map(|t| t.trunk.iter().chain(t.leaves.iter()).copied())
            .collect()
    }
}

/// Plants the first Red-Black grove at `tree_sites`, keeping the trunk
/// spacing and every clearance.
pub fn plant_red_black_grove(
    seed: u32,
    fortress: &RedBlackFortressLayout,
    clearance: &TreeClearance,
    world: &mut VoxelWorld,
) -> RedBlackGrove {
    let mut grove = RedBlackGrove::default();
    for (x, z) in tree_sites(fortress) {
        if grove
            .trees
            .iter()
            .any(|t| (t.base.x - x).abs() < TREE_SPACING && (t.base.z - z).abs() < TREE_SPACING)
        {
            continue;
        }
        if let Some(tree) = grow_red_black_tree(seed, x, z, clearance, world) {
            grove.trees.push(tree);
        }
    }
    grove
}

/// Everything the Gate 17.5 stages built, in pipeline order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackIdentity {
    pub grove: RedBlackGrove,
}

impl RedBlackIdentity {
    /// Cells the identity stages added to the world (replacements of
    /// existing cells are not counted), each once.
    pub fn added_cells(&self) -> Vec<IVec3> {
        let mut seen = HashSet::new();
        self.grove
            .cells()
            .into_iter()
            .filter(|c| seen.insert(*c))
            .collect()
    }
}

/// Builds the Gate 17.5 identity stages on the finished Gate 17 world.
pub fn build_red_black_identity(
    seed: u32,
    expansion: &WorldExpansionLayout,
    fortress: &RedBlackFortressLayout,
    protected: &HashSet<IVec3>,
    walk_columns: &HashSet<(i32, i32)>,
    focal_columns: &[(i32, i32)],
    floor_y: i32,
    world: &mut VoxelWorld,
) -> RedBlackIdentity {
    let _ = expansion;
    let clearance = TreeClearance {
        cells: protected,
        walk_columns,
        focal_columns,
        floor_y,
    };
    let grove = plant_red_black_grove(seed, fortress, &clearance, world);
    RedBlackIdentity { grove }
}
