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
use crate::scene::fortress::{
    TOWER_BAND_COURSE, TOWER_CREST_COURSES, dark_masonry, family_brick, is_dark_structure,
    is_family_brick,
};
use crate::scene::orientation::Orientation;
use crate::scene::overworld::ColumnRect;
use crate::scene::overworld::Tree;
use crate::scene::overworld_blocks::{
    crimson_diamond_material_id, crying_obsidian_crimson_material_id,
    crying_obsidian_orange_material_id, orange_spade_material_id,
};
use crate::scene::overworld_blocks::{
    mycelium_material_id, polished_blackstone_bricks_material_id, smooth_basalt_material_id,
};
use crate::scene::red_black_maze::Family;
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
/// beside the forecourt (off every sight line from the approach to the
/// gate) and two on the lobe's east edge, beside the fortress rather than
/// in front of it, so no view of the four towers is screened by foliage.
pub fn tree_sites(fortress: &RedBlackFortressLayout) -> Vec<(i32, i32)> {
    let f = fortress.footprint;
    vec![
        (f.min_x - 4, f.min_z + 5),
        (f.max_x + 1, f.min_z + 2),
        (f.max_x + 1, f.max_z - 1),
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
    // Inside the world box as it stands: a tree never grows the bounds.
    let bounds = world.bounds()?;
    let inside = |c: IVec3| bounds.contains_cell(c);
    let trunk: Vec<IVec3> = (1..=height).map(|d| IVec3::new(x, base.y - d, z)).collect();
    if trunk.iter().any(|c| {
        world.contains(*c)
            || clearance.cells.contains(c)
            || near_focal(*c)
            || in_walk_column(*c)
            || !inside(*c)
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
            || !inside(c)
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

// ---------------------------------------------------------------------
// Fourth tower (C217)
// ---------------------------------------------------------------------

/// The fourth tower, as built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourthTower {
    /// The corner the Gate 17 layout left without a tower.
    pub bounds: ColumnRect,
    /// Dark shell cells placed (keep and curtain cells are reused as seam).
    pub shell: Vec<IVec3>,
    /// Existing fortress cells the tower shares on its outline (the keep's
    /// south wall, the curtain corner), left as they were.
    pub seam: Vec<IVec3>,
    /// The two-colour band ring (Crimson and Orange alternating).
    pub band: Vec<IVec3>,
    /// Heraldic motifs on the two outward faces, with their type.
    pub crests: Vec<(IVec3, BlockType)>,
    /// Crying-obsidian finials on the corners.
    pub finials: Vec<IVec3>,
    /// The corinto cap closing the core at the tower top.
    pub cap: Vec<IVec3>,
    /// The entrance from the courtyard, left open.
    pub entrance: Vec<IVec3>,
    /// The open core column.
    pub core: (i32, i32),
    /// Gate 17 accent cells the tower replaced (the corner crystal).
    pub replaced: Vec<IVec3>,
}

impl Default for FourthTower {
    /// No tower: an empty outline (every corner already had one).
    fn default() -> Self {
        Self {
            bounds: ColumnRect::new(0, 0, -1, -1),
            shell: Vec::new(),
            seam: Vec::new(),
            band: Vec::new(),
            crests: Vec::new(),
            finials: Vec::new(),
            cap: Vec::new(),
            entrance: Vec::new(),
            core: (0, 0),
            replaced: Vec::new(),
        }
    }
}

impl FourthTower {
    /// Cells new to the world (the replaced Gate 17 crystal cells already
    /// counted as fortress cells are left out).
    pub fn added_cells(&self) -> Vec<IVec3> {
        self.placed_cells()
            .into_iter()
            .filter(|c| !self.replaced.contains(c))
            .collect()
    }

    /// Every cell this stage wrote.
    pub fn placed_cells(&self) -> Vec<IVec3> {
        self.shell
            .iter()
            .chain(self.band.iter())
            .chain(self.crests.iter().map(|(c, _)| c))
            .chain(self.finials.iter())
            .chain(self.cap.iter())
            .copied()
            .collect()
    }

    /// Every cell of the tower's outline above the ground, placed or shared.
    pub fn cells(&self) -> Vec<IVec3> {
        self.placed_cells()
            .into_iter()
            .chain(self.seam.iter().copied())
            .collect()
    }

    pub fn is_shell(&self, x: i32, z: i32) -> bool {
        let r = self.bounds;
        r.contains(x, z) && (x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z)
    }
}

/// The footprint corner that carries no Gate 17 tower.
pub fn missing_tower_corner(fortress: &RedBlackFortressLayout) -> Option<ColumnRect> {
    let f = fortress.footprint;
    let side = fortress.crimson_tower.bounds.width();
    let t = side - 1;
    [
        ColumnRect::new(f.min_x, f.min_z, f.min_x + t, f.min_z + t),
        ColumnRect::new(f.max_x - t, f.min_z, f.max_x, f.min_z + t),
        ColumnRect::new(f.min_x, f.max_z - t, f.min_x + t, f.max_z),
        ColumnRect::new(f.max_x - t, f.max_z - t, f.max_x, f.max_z),
    ]
    .into_iter()
    .find(|r| fortress.towers().iter().all(|tw| tw.bounds != *r))
}

/// Completes the fortress's fourth corner with a dark mixed tower.
///
/// The tower stands on the corner the Gate 17 layout left free, with the
/// same section, courses and Down orientation as the three family towers.
/// Its outline reuses the existing keep and curtain cells it meets as seam
/// (only the Gate 17 corner crystal gives way), adds dark masonry
/// elsewhere, a band alternating Crimson and Orange bricks (the two
/// families with the fewest coloured bricks), a CrimsonDiamond and an
/// OrangeSpade on its outward faces, alternating crying-obsidian finials
/// and a corinto cap over its open core. It opens onto the courtyard
/// through a two-cell entrance on its west face. No new colour family.
pub fn build_fourth_tower(
    seed: u32,
    fortress: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> FourthTower {
    let Some(bounds) = missing_tower_corner(fortress) else {
        return FourthTower::default();
    };
    let lv = fortress.levels;
    let f = fortress.footprint;
    let core = (
        (bounds.min_x + bounds.max_x) / 2,
        (bounds.min_z + bounds.max_z) / 2,
    );
    let mut built = FourthTower {
        bounds,
        core,
        ..Default::default()
    };
    // Entrance: the west face, middle row, from the courtyard.
    let entrance_x = bounds.min_x;
    let entrance_z = core.1;
    built.entrance = vec![
        IVec3::new(entrance_x, lv.base_y, entrance_z),
        IVec3::new(entrance_x, lv.base_y - 1, entrance_z),
    ];
    let face_x = if bounds.max_x == f.max_x {
        bounds.max_x
    } else {
        bounds.min_x
    };
    let face_z = if bounds.max_z == f.max_z {
        bounds.max_z
    } else {
        bounds.min_z
    };
    let crests = [
        (
            IVec3::new(core.0, lv.ground_y - TOWER_CREST_COURSES[0], face_z),
            (BlockType::CrimsonDiamond, crimson_diamond_material_id()),
        ),
        (
            IVec3::new(face_x, lv.ground_y - TOWER_CREST_COURSES[1], core.1),
            (BlockType::OrangeSpade, orange_spade_material_id()),
        ),
    ];
    let accent = |b: &BlockInstance| {
        matches!(
            b.block_type(),
            BlockType::BuddingAmethyst | BlockType::AmethystCluster
        )
    };
    for z in bounds.min_z..=bounds.max_z {
        for x in bounds.min_x..=bounds.max_x {
            let shell = built.is_shell(x, z);
            if !shell {
                for y in (lv.tower_top_y + 1)..=lv.base_y {
                    world.remove(IVec3::new(x, y, z));
                }
                continue;
            }
            for y in (lv.tower_top_y..=lv.base_y).rev() {
                let cell = IVec3::new(x, y, z);
                if built.entrance.contains(&cell) {
                    world.remove(cell);
                    continue;
                }
                match world.get(cell) {
                    Some(b) if accent(b) => {
                        built.replaced.push(cell);
                    }
                    Some(_) => {
                        built.seam.push(cell);
                        continue;
                    }
                    None => {}
                }
                if let Some((_, (bt, material))) = crests.iter().find(|(c, _)| *c == cell) {
                    world.insert(cell, down(*bt, *material));
                    built.crests.push((cell, *bt));
                } else if y == lv.ground_y - TOWER_BAND_COURSE {
                    let family = if (x + z) % 2 == 0 {
                        Family::Crimson
                    } else {
                        Family::Orange
                    };
                    world.insert(cell, family_brick(family));
                    built.band.push(cell);
                } else {
                    world.insert(cell, dark_masonry(seed, cell));
                    built.shell.push(cell);
                }
            }
            let corner = (x == bounds.min_x || x == bounds.max_x)
                && (z == bounds.min_z || z == bounds.max_z);
            let finial = IVec3::new(x, lv.max_y, z);
            if corner && !world.contains(finial) {
                let (bt, material) = if (x + z) % 2 == 0 {
                    (
                        BlockType::CryingObsidianCrimson,
                        crying_obsidian_crimson_material_id(),
                    )
                } else {
                    (
                        BlockType::CryingObsidianOrange,
                        crying_obsidian_orange_material_id(),
                    )
                };
                world.insert(finial, down(bt, material));
                built.finials.push(finial);
            }
        }
    }
    let cap = IVec3::new(core.0, lv.tower_top_y, core.1);
    if !world.contains(cap) {
        world.insert(cap, planks());
        built.cap.push(cap);
    }
    built
}

// ---------------------------------------------------------------------
// North shoulder of the lower lobe (C218)
// ---------------------------------------------------------------------

const SHOULDER_RELIEF_SALT: u32 = 0x175E_0003;
const SHOULDER_EDGE_SALT: u32 = 0x175E_0004;
const SHOULDER_GROUND_SALT: u32 = 0x175E_0005;

/// The landscape added around the fortress, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LandscapeExpansion {
    /// New columns `(x, z)`.
    pub columns: Vec<(i32, i32)>,
    /// Their ground cells (the lowest cell; its -Y face is the floor).
    pub ground: Vec<IVec3>,
    /// Every cell added (ground and body).
    pub cells: Vec<IVec3>,
}

/// Grows a narrow, irregular shoulder along the lower lobe's north ring:
/// one full row (`z = min_z - 1` of the lobe) from just past the Gate 14
/// landing to the fortress's last column, and a ragged second row further
/// out. Each new column hangs beside its ring neighbour, three cells thick
/// on the inner row and two on the outer, with a hashed one-course relief;
/// its ground is basalt, mycelium or blackstone and its body dark masonry,
/// all `Down`. It gives the corinto path a walkable north shoulder and the
/// grove behind the fortress room, stays attached to the one lower mass,
/// and never reaches past the world box.
pub fn expand_red_black_landscape(
    seed: u32,
    expansion: &WorldExpansionLayout,
    fortress: &RedBlackFortressLayout,
    protected: &HashSet<IVec3>,
    floor_y: i32,
    world: &mut VoxelWorld,
) -> LandscapeExpansion {
    let mut built = LandscapeExpansion::default();
    let lobe = expansion.red_black_extension;
    let inner_z = lobe.min_z - 1;
    let outer_z = lobe.min_z - 2;
    let x_from = expansion.red_black_path_anchor.x + 2;
    let x_to = fortress.footprint.max_x - 1;
    let ground_block = |cell: IVec3| {
        let roll = expansion_hash(seed, cell.x, cell.z, SHOULDER_GROUND_SALT);
        if roll < 0.45 {
            down(BlockType::SmoothBasalt, smooth_basalt_material_id())
        } else if roll < 0.75 {
            down(BlockType::Mycelium, mycelium_material_id())
        } else {
            down(
                BlockType::PolishedBlackstoneBricks,
                polished_blackstone_bricks_material_id(),
            )
        }
    };
    let grow = |world: &mut VoxelWorld,
                built: &mut LandscapeExpansion,
                x: i32,
                z: i32,
                ground_y: i32,
                thickness: i32| {
        let cells: Vec<IVec3> = (0..thickness)
            .map(|d| IVec3::new(x, ground_y + d, z))
            .collect();
        if ground_y < floor_y
            || cells
                .iter()
                .any(|c| world.contains(*c) || protected.contains(c))
        {
            return false;
        }
        for (i, c) in cells.iter().enumerate() {
            let block = if i == 0 {
                ground_block(*c)
            } else {
                crate::scene::fortress::dark_masonry(seed, *c)
            };
            world.insert(*c, block);
            built.cells.push(*c);
        }
        built.ground.push(cells[0]);
        built.columns.push((x, z));
        true
    };
    for x in x_from..=x_to {
        // The ring neighbour's ground sets the level (trees ignored).
        let Some(ring) = (floor_y..=0)
            .map(|y| IVec3::new(x, y, lobe.min_z))
            .find(|c| {
                world.get(*c).is_some_and(|b| {
                    !matches!(
                        b.block_type(),
                        BlockType::RedBlackLog | BlockType::RedBlackLeaves
                    )
                })
            })
        else {
            continue;
        };
        let relief = (expansion_hash(seed, x, inner_z, SHOULDER_RELIEF_SALT) < 0.3) as i32;
        let inner_y = ring.y + relief;
        if !grow(world, &mut built, x, inner_z, inner_y, 3) {
            continue;
        }
        let ragged = expansion_hash(seed, x, outer_z, SHOULDER_EDGE_SALT) < 0.65;
        if ragged && x > x_from + 1 && x < x_to - 1 {
            let outer_y =
                inner_y + (expansion_hash(seed, x, outer_z, SHOULDER_RELIEF_SALT) < 0.5) as i32;
            grow(world, &mut built, x, outer_z, outer_y, 2);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Approach composition (C219)
// ---------------------------------------------------------------------

/// Fence posts per fragment along the path's north edge, and the gap after.
pub const EDGE_FRAGMENT: usize = 2;
pub const EDGE_GAP: usize = 2;

/// The composed approach, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApproachComposition {
    /// Shoulder ground cells beside the path re-laid in blackstone.
    pub edge: Vec<IVec3>,
    /// `RedBlackFence` fragments standing on that edge.
    pub fences: Vec<IVec3>,
    /// Trunk sites of the shoulder trees (planted into the grove).
    pub tree_sites: Vec<(i32, i32)>,
}

/// The shoulder's tree sites: on its ragged outer row, one past the
/// landing's side of the path and one at the path's turn, both north of
/// every sight line to the gate.
pub fn shoulder_tree_sites(expansion: &WorldExpansionLayout) -> Vec<(i32, i32)> {
    let outer_z = expansion.red_black_extension.min_z - 2;
    let turn_x = expansion.red_black_fortress_pad.min_x - 1;
    vec![(turn_x - 3, outer_z), (turn_x, outer_z)]
}

/// Composes the fortress approach: the shoulder's inner ground cells beside
/// the path become a blackstone edge with short `RedBlackFence` fragments
/// (posts in pairs with gaps, only within one course of the path), and two
/// crimson trees grow on the shoulder's outer row at the path's sides. The
/// path, its head room, the gate's sight lines and every family scene stay
/// clear; the fortress silhouette is left open.
pub fn compose_fortress_approach(
    seed: u32,
    expansion: &WorldExpansionLayout,
    approach: &RedBlackApproach,
    landscape: &LandscapeExpansion,
    clearance: &TreeClearance,
    grove: &mut RedBlackGrove,
    world: &mut VoxelWorld,
) -> ApproachComposition {
    let mut built = ApproachComposition::default();
    let inner_z = expansion.red_black_extension.min_z - 1;
    let mut edge: Vec<(IVec3, IVec3)> = landscape
        .ground
        .iter()
        .filter(|g| g.z == inner_z)
        .filter_map(|g| {
            approach
                .main
                .iter()
                .find(|p| p.x == g.x && p.z == g.z + 1)
                .map(|p| (*g, *p))
        })
        .collect();
    edge.sort_by_key(|(g, _)| g.x);
    for (i, (g, path)) in edge.iter().enumerate() {
        if world
            .get(*g)
            .is_some_and(|b| b.block_type() != BlockType::PolishedBlackstoneBricks)
        {
            world.insert(
                *g,
                down(
                    BlockType::PolishedBlackstoneBricks,
                    polished_blackstone_bricks_material_id(),
                ),
            );
        }
        built.edge.push(*g);
        let in_fragment = i % (EDGE_FRAGMENT + EDGE_GAP) < EDGE_FRAGMENT;
        let post = IVec3::new(g.x, g.y - 1, g.z);
        if in_fragment
            && (path.y - g.y).abs() <= 1
            && !world.contains(post)
            && !clearance.cells.contains(&post)
        {
            world.insert(post, fence());
            built.fences.push(post);
        }
    }
    for (x, z) in shoulder_tree_sites(expansion) {
        if grove.trees.len() >= RED_BLACK_TREES_MAX {
            break;
        }
        let spaced = grove
            .trees
            .iter()
            .all(|t| (t.base.x - x).abs() >= TREE_SPACING || (t.base.z - z).abs() >= TREE_SPACING);
        if !spaced {
            continue;
        }
        if let Some(tree) = grow_red_black_tree(seed, x, z, clearance, world) {
            grove.trees.push(tree);
            built.tree_sites.push((x, z));
        }
    }
    built
}

// ---------------------------------------------------------------------
// Identity balance (C220)
// ---------------------------------------------------------------------

/// The final balance pass, as applied (every cell is a re-laid one).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdentityBalance {
    /// Dark top-course cells of the curtain between towers, now a corinto
    /// hoarding.
    pub hoardings: Vec<IVec3>,
    /// Courtyard ground linking the walkway to the tower alleys.
    pub walkways: Vec<IVec3>,
    /// The keep's upper-floor beams, now corinto logs.
    pub beams: Vec<IVec3>,
}

impl IdentityBalance {
    pub fn cells(&self) -> Vec<IVec3> {
        self.hoardings
            .iter()
            .chain(self.walkways.iter())
            .chain(self.beams.iter())
            .copied()
            .collect()
    }
}

/// Rebalances the fortress so dark stone stays the structure while corinto
/// timber clearly carries the routes and the woodwork: the curtain's dark
/// top-course cells between the towers become a timber hoarding (coloured
/// bands stay), the courtyard cells linking the walkway to both tower
/// alleys become planks (the organic patches stay), and the keep's
/// upper-floor beams become `RedBlackLog`. Only existing dark cells are
/// re-laid; nothing is added, no suit block, brick, crystal or light moves.
pub fn balance_red_black_identity(
    fortress: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> IdentityBalance {
    let mut built = IdentityBalance::default();
    let lv = fortress.levels;
    let f = fortress.footprint;
    let dark = |world: &VoxelWorld, c: IVec3| {
        world
            .get(c)
            .is_some_and(|b| is_dark_structure(b.block_type()))
    };
    for z in [f.min_z, f.max_z] {
        for x in f.min_x..=f.max_x {
            if fortress.tower_at(x, z).is_some()
                || fortress.central_keep.contains(x, z)
                || (x >= f.max_x - 2 && (z == f.max_z || z == f.min_z))
            {
                continue;
            }
            let cell = IVec3::new(x, lv.wall_top_y, z);
            if dark(world, cell) {
                world.insert(cell, planks());
                built.hoardings.push(cell);
            }
        }
    }
    let walk_x = fortress.central_keep.min_x - 1;
    for z in [f.min_z + 1, f.min_z + 2, f.max_z - 2, f.max_z - 1] {
        if !fortress.courtyard.contains(&(walk_x, z)) {
            continue;
        }
        let cell = IVec3::new(walk_x, lv.ground_y, z);
        if dark(world, cell) {
            world.insert(cell, planks());
            built.walkways.push(cell);
        }
    }
    let h = fortress.hall;
    for x in h.min_x..=h.max_x {
        for z in [h.min_z, h.max_z] {
            let cell = IVec3::new(x, lv.upper_floor_y, z);
            if dark(world, cell) {
                world.insert(
                    cell,
                    down(BlockType::RedBlackLog, red_black_log_material_id()),
                );
                built.beams.push(cell);
            }
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
    pub fourth_tower: FourthTower,
    pub landscape: LandscapeExpansion,
    pub composition: ApproachComposition,
    pub balance: IdentityBalance,
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
            .chain(self.fourth_tower.added_cells())
            .chain(self.landscape.cells.iter().copied())
            .chain(self.composition.fences.iter().copied())
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
    let clearance = TreeClearance {
        cells: protected,
        walk_columns,
        focal_columns,
        floor_y,
    };
    // Architecture first (timber, path, gatehouse, fourth tower, landscape),
    // then the vegetation that has to keep clear of all of it.
    let timber = weave_fortress_timber(fortress, world);
    let path = lay_corinto_path(
        approach,
        fortress,
        &clearance,
        &RedBlackGrove::default(),
        world,
    );
    let gatehouse = build_red_black_gatehouse(fortress, world);
    let fourth_tower = build_fourth_tower(seed, fortress, world);
    let landscape =
        expand_red_black_landscape(seed, expansion, fortress, protected, floor_y, world);
    // The fourth tower's crests join the protected cells: no leaf may touch
    // a symbol.
    let mut grown_protection = protected.clone();
    for (c, _) in &fourth_tower.crests {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            grown_protection.insert(IVec3::new(c.x + dx, c.y + dy, c.z + dz));
        }
    }
    let clearance = TreeClearance {
        cells: &grown_protection,
        walk_columns,
        focal_columns,
        floor_y,
    };
    let mut grove = plant_red_black_grove(seed, fortress, &clearance, world);
    let composition = compose_fortress_approach(
        seed, expansion, approach, &landscape, &clearance, &mut grove, world,
    );
    let balance = balance_red_black_identity(fortress, world);
    RedBlackIdentity {
        grove,
        timber,
        path,
        gatehouse,
        fourth_tower,
        landscape,
        composition,
        balance,
    }
}
