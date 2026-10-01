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
use crate::scene::expansion::RedBlackApproach;
use crate::scene::expansion::{WorldExpansionLayout, expansion_hash, lower_ground_cell};
use crate::scene::fortress::RedBlackFortressLayout;
use crate::scene::fortress::{is_dark_structure, is_family_brick};
use crate::scene::orientation::Orientation;
use crate::scene::overworld::Tree;
use crate::scene::overworld_blocks::polished_blackstone_bricks_material_id;
use crate::scene::red_black_timber::{
    red_black_fence_material_id, red_black_leaves_material_id, red_black_log_material_id,
    red_black_wood_door_bottom_material_id, red_black_wood_door_top_material_id,
    red_black_wood_planks_material_id,
};
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

// ---------------------------------------------------------------------
// Corinto timber inside the fortress (C214)
// ---------------------------------------------------------------------

/// The fortress's timber layer, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FortressTimber {
    /// Dark cells turned into `RedBlackWoodPlanks`: hall floor, upper
    /// floor between the beams, keep roof, gate vault, courtyard walkway.
    pub planks: Vec<IVec3>,
    /// Small timber balconies added on the keep's alley faces.
    pub balconies: Vec<IVec3>,
    /// `RedBlackFence` rails: balcony edges and the keep's stair well.
    pub rails: Vec<IVec3>,
}

impl FortressTimber {
    /// Cells added (the planks replace existing cells).
    pub fn added_cells(&self) -> Vec<IVec3> {
        self.balconies
            .iter()
            .chain(self.rails.iter())
            .copied()
            .collect()
    }

    pub fn cells(&self) -> Vec<IVec3> {
        self.planks
            .iter()
            .chain(self.balconies.iter())
            .chain(self.rails.iter())
            .copied()
            .collect()
    }
}

fn planks() -> BlockInstance {
    down(
        BlockType::RedBlackWoodPlanks,
        red_black_wood_planks_material_id(),
    )
}

fn fence() -> BlockInstance {
    down(BlockType::RedBlackFence, red_black_fence_material_id())
}

/// Lays the fortress's timber layer. Only dark structural cells of floors,
/// roof, vault and the courtyard walkway are re-laid in corinto planks;
/// the curtain, the towers, the keep's walls and pillars, the coloured
/// bricks and every suit block keep their stone. Two short balconies (one
/// per alley face of the keep, one course above head room) and rails along
/// their edges and around the keep's stair well are the only added cells.
pub fn weave_fortress_timber(
    fortress: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> FortressTimber {
    let mut built = FortressTimber::default();
    let lv = fortress.levels;
    let h = fortress.hall;
    let k = fortress.central_keep;
    let relay = |world: &mut VoxelWorld, cell: IVec3, built: &mut FortressTimber| {
        if world
            .get(cell)
            .is_some_and(|b| is_dark_structure(b.block_type()))
        {
            world.insert(cell, planks());
            built.planks.push(cell);
        }
    };
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            // Hall floor and keep roof.
            relay(world, IVec3::new(x, lv.ground_y, z), &mut built);
            relay(world, IVec3::new(x, lv.keep_roof_y, z), &mut built);
            // Upper floor between the two basalt beam rows.
            if z != h.min_z && z != h.max_z {
                relay(world, IVec3::new(x, lv.upper_floor_y, z), &mut built);
            }
        }
    }
    // Gate vault: the timber ceiling of the entry corridor.
    let g = fortress.gatehouse;
    let vault_y = lv.wall_top_y - 1;
    for z in g.min_z..=g.max_z {
        for x in g.min_x..=g.max_x {
            relay(world, IVec3::new(x, vault_y, z), &mut built);
        }
    }
    // Courtyard walkway: the route's courtyard cells between the threshold
    // and the keep door (the blackstone threshold stays).
    let walk_x = k.min_x - 1;
    for z in fortress.gate.base.z..=fortress.keep_door.base.z {
        relay(world, IVec3::new(walk_x, lv.ground_y, z), &mut built);
    }
    // Balconies on the keep's north and south faces, over the alleys, with
    // a rail on their outer edge.
    let balcony_y = lv.wall_top_y;
    for (z, rail_z) in [(k.min_z - 1, k.min_z - 1), (k.max_z + 1, k.max_z + 1)] {
        for x in [h.min_x, h.min_x + 1] {
            let cell = IVec3::new(x, balcony_y, z);
            if world.contains(cell) || !fortress.courtyard.contains(&(x, z)) {
                continue;
            }
            world.insert(cell, planks());
            built.balconies.push(cell);
            let rail = IVec3::new(x, balcony_y - 1, rail_z);
            if !world.contains(rail) {
                world.insert(rail, fence());
                built.rails.push(rail);
            }
        }
    }
    // Stair-well rail on the upper floor, on the well's west side.
    for (x, z) in &fortress.stair_well {
        let cell = IVec3::new(x - 1, lv.upper_floor_y - 1, *z);
        let floor = IVec3::new(cell.x, lv.upper_floor_y, cell.z);
        if h.contains(cell.x, cell.z) && world.contains(floor) && !world.contains(cell) {
            world.insert(cell, fence());
            built.rails.push(cell);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Corinto path to the fortress (C215)
// ---------------------------------------------------------------------

/// Every how many border cells a fence post stands.
pub const PATH_FENCE_PERIOD: usize = 2;

/// The corinto path, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CorintoPath {
    /// The walking surface, in walking order (Gate 15 approach cells now
    /// laid in `RedBlackWoodPlanks`).
    pub planks: Vec<IVec3>,
    /// The two coloured-brick accents flanking the forecourt, kept.
    pub accents: Vec<IVec3>,
    /// Ground cells beside the path re-laid in polished blackstone.
    pub borders: Vec<IVec3>,
    /// `RedBlackFence` posts standing on selected border cells.
    pub fences: Vec<IVec3>,
}

impl CorintoPath {
    pub fn added_cells(&self) -> Vec<IVec3> {
        self.fences.clone()
    }

    pub fn columns(&self) -> Vec<(i32, i32)> {
        self.planks
            .iter()
            .chain(self.accents.iter())
            .map(|c| (c.x, c.z))
            .collect()
    }
}

/// Lays the corinto path: every Gate 15 approach cell (the coloured
/// accents at the forecourt excepted) becomes `RedBlackWoodPlanks`; each
/// ground cell beside it that belongs to no path, route, focal scene, tree
/// or fortress becomes a polished-blackstone border; and every
/// `PATH_FENCE_PERIOD`-th border cell, within one course of the path, carries a
/// `RedBlackFence` post on its ground (functionally above it). The path
/// cells, their head room and every route stay exactly where they were.
pub fn lay_corinto_path(
    approach: &RedBlackApproach,
    fortress: &RedBlackFortressLayout,
    clearance: &TreeClearance,
    grove: &RedBlackGrove,
    world: &mut VoxelWorld,
) -> CorintoPath {
    let mut built = CorintoPath::default();
    for cell in &approach.main {
        let accent = world
            .get(*cell)
            .is_some_and(|b| is_family_brick(b.block_type()));
        if accent {
            built.accents.push(*cell);
        } else {
            world.insert(*cell, planks());
            built.planks.push(*cell);
        }
    }
    let path: HashSet<(i32, i32)> = built.columns().into_iter().collect();
    let trees: HashSet<(i32, i32)> = grove.cells().iter().map(|c| (c.x, c.z)).collect();
    let pad = fortress.footprint;
    let mut seen = HashSet::new();
    for cell in approach.main.iter() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (x, z) = (cell.x + dx, cell.z + dz);
            if path.contains(&(x, z))
                || pad.contains(x, z)
                || clearance.walk_columns.contains(&(x, z))
                || trees.contains(&(x, z))
                || !seen.insert((x, z))
            {
                continue;
            }
            let Some(ground) = lower_ground_cell(world, x, z, clearance.floor_y - 4, 0) else {
                continue;
            };
            let near_focal = clearance.focal_columns.iter().any(|(fx, fz)| {
                (fx - x).abs() < TREE_FOCAL_CLEARANCE && (fz - z).abs() < TREE_FOCAL_CLEARANCE
            });
            // Only plain terrain is re-laid: scenery (amethyst, outcrops),
            // focal scenes and protected cells keep their blocks.
            let plain = world.get(ground).is_some_and(|b| {
                matches!(
                    b.block_type(),
                    BlockType::SmoothBasalt
                        | BlockType::Deepslate
                        | BlockType::Mycelium
                        | BlockType::PolishedBlackstoneBricks
                )
            });
            if clearance.cells.contains(&ground) || near_focal || !plain {
                continue;
            }
            world.insert(
                ground,
                down(
                    BlockType::PolishedBlackstoneBricks,
                    polished_blackstone_bricks_material_id(),
                ),
            );
            built.borders.push(ground);
        }
    }
    for (i, border) in built.borders.iter().enumerate() {
        if i % PATH_FENCE_PERIOD != 0 {
            continue;
        }
        // Within one course of the path cell it edges.
        let level = approach.main.iter().any(|p| {
            (p.x - border.x).abs() + (p.z - border.z).abs() == 1 && (p.y - border.y).abs() <= 1
        });
        let post = IVec3::new(border.x, border.y - 1, border.z);
        if level && !world.contains(post) && !clearance.cells.contains(&post) {
            world.insert(post, fence());
            built.fences.push(post);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Gatehouse and Red-Black door (C216)
// ---------------------------------------------------------------------

/// The gatehouse, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackGatehouse {
    /// The door cells (`RedBlackWoodDoor`), the ground-side half first in
    /// each column.
    pub door: Vec<IVec3>,
    /// The entry corridor's floor, re-laid in corinto planks.
    pub corridor_floor: Vec<IVec3>,
    /// The timber gallery crowning the gate block above its vault.
    pub hood: Vec<IVec3>,
    /// Fence rails on the gallery's outer edge.
    pub rails: Vec<IVec3>,
}

impl RedBlackGatehouse {
    pub fn added_cells(&self) -> Vec<IVec3> {
        self.door
            .iter()
            .chain(self.hood.iter())
            .chain(self.rails.iter())
            .copied()
            .collect()
    }
}

/// Completes the fortress entrance as a readable gatehouse.
///
/// The gate opening receives a `RedBlackWoodDoor`, two columns wide and
/// two cells tall: the door's lower half sits next to the ground (the
/// realm's floor at `-Y`'s far side) and its upper half one course further
/// toward `-Y`, both facing West toward the path. A door panel is a
/// vertical slab, so it keeps the horizontal `West` facing every door in
/// the world uses; turning it `Down` would lay the panel across the
/// corridor. The corridor floor becomes corinto planks so the path runs
/// through the gate; a planks hood projects one column out over the
/// entrance, one course past the path's head room, with fence finials on
/// its ends. The arch, its coloured jambs and lintel and its crying
/// obsidian stay as Gate 17 built them; nothing here is a portal.
pub fn build_red_black_gatehouse(
    fortress: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> RedBlackGatehouse {
    let mut built = RedBlackGatehouse::default();
    let gate = fortress.gate;
    let lv = fortress.levels;
    for dz in 0..gate.width {
        for (dy, material) in [
            (0, red_black_wood_door_bottom_material_id()),
            (1, red_black_wood_door_top_material_id()),
        ] {
            let cell = IVec3::new(gate.base.x, gate.base.y - dy, gate.base.z + dz);
            world.insert(
                cell,
                BlockInstance::new(BlockType::RedBlackWoodDoor, material, gate.facing),
            );
            built.door.push(cell);
        }
    }
    for (x, z) in fortress.gate_corridor() {
        let cell = IVec3::new(x, lv.ground_y, z);
        if world
            .get(cell)
            .is_some_and(|b| is_dark_structure(b.block_type()))
        {
            world.insert(cell, planks());
            built.corridor_floor.push(cell);
        }
    }
    // A timber gallery crowns the gate block one course past its vault, so
    // the gatehouse rises above the curtain as its own corinto story; fence
    // rails line its outer (west) edge. Nothing projects over the path.
    let g = fortress.gatehouse;
    let gallery_y = lv.wall_top_y - 2;
    for z in g.min_z..=g.max_z {
        for x in g.min_x..=g.max_x {
            let cell = IVec3::new(x, gallery_y, z);
            let under = IVec3::new(x, gallery_y + 1, z);
            if !world.contains(cell) && world.contains(under) {
                world.insert(cell, planks());
                built.hood.push(cell);
            }
        }
    }
    for z in (g.min_z + 1)..g.max_z {
        let cell = IVec3::new(g.min_x, gallery_y - 1, z);
        if built.hood.contains(&IVec3::new(g.min_x, gallery_y, z)) && !world.contains(cell) {
            world.insert(cell, fence());
            built.rails.push(cell);
        }
    }
    built
}

/// Everything the Gate 17.5 stages built, in pipeline order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackIdentity {
    pub grove: RedBlackGrove,
    pub timber: FortressTimber,
    pub path: CorintoPath,
    pub gatehouse: RedBlackGatehouse,
}

impl RedBlackIdentity {
    /// Cells the identity stages added to the world (replacements of
    /// existing cells are not counted), each once.
    pub fn added_cells(&self) -> Vec<IVec3> {
        let mut seen = HashSet::new();
        self.grove
            .cells()
            .into_iter()
            .chain(self.timber.added_cells())
            .chain(self.path.added_cells())
            .chain(self.gatehouse.added_cells())
            .filter(|c| seen.insert(*c))
            .collect()
    }
}

/// Builds the Gate 17.5 identity stages on the finished Gate 17 world.
pub fn build_red_black_identity(
    seed: u32,
    expansion: &WorldExpansionLayout,
    approach: &RedBlackApproach,
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
    let timber = weave_fortress_timber(fortress, world);
    let path = lay_corinto_path(approach, fortress, &clearance, &grove, world);
    let gatehouse = build_red_black_gatehouse(fortress, world);
    RedBlackIdentity {
        grove,
        timber,
        path,
        gatehouse,
    }
}
