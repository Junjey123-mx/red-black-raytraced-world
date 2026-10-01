//! Gate 15: the structural expansion of both worlds.
//!
//! The diamond grows one lobe to the east, north of the portal wall: the
//! only sector free of the house, the pond, the trees, the cutaway and the
//! Red-Black family compositions. The lobe belongs to the same floating
//! mass: its top is Overworld geology (grass, dirt, stone, deepslate) and
//! its underside is Red-Black ground facing -Y, exactly like the rest of
//! the rhombus. On top it reserves the castle pad of Gate 16; underneath,
//! the fortress pad of Gate 17. Everything is derived from the world as
//! built: the current bounds, the portal anchor, the existing path and the
//! Gate 14 landing.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::descent::InvertedRouteLayout;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::column_top;
use crate::scene::overworld::{ColumnRect, HOUSE_FOOTPRINT, PathLayout, pond_basin_columns};
use crate::scene::overworld_blocks::{
    cobblestone_material_id, deepslate_material_id, fence_material_id, log_material_id,
};
use crate::scene::red_black_maze::{Family, lower_mass_block, surface_block};
use crate::scene::rhombus::RhombusConfig;
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::generator::stratum_material;
use crate::scene::voxel_world::VoxelWorld;

/// Columns the lobe adds east of the current bounds (the last one is the
/// ragged edge, present only where the outline hash says so).
pub const EXPANSION_WIDTH: i32 = 13;
/// Rows of the lobe (it ends at the portal wall row, never in the cutaway).
pub const EXPANSION_DEPTH: i32 = 12;
/// Side of the reserved architecture pads, in navigable columns.
pub const PAD_SIDE: i32 = 10;
/// Recommended total voxel count at the close of the gate.
pub const VOXEL_BUDGET_TARGET: usize = 8200;
/// Soft maximum: not to be exceeded without stopping and measuring.
pub const VOXEL_BUDGET_SOFT_MAX: usize = 8500;

/// Where both worlds grow and where the two castles will stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldExpansionLayout {
    /// Columns of the new upper terrain (the lobe's plan).
    pub overworld_extension: ColumnRect,
    /// Navigable columns reserved for the Gate 16 castle, on the lobe's top.
    pub overworld_castle_pad: ColumnRect,
    /// Grass cell level of the castle pad (its walkable face is one above).
    pub castle_pad_surface_y: i32,
    /// The existing path cell the new approach branches from.
    pub overworld_path_anchor: IVec3,
    /// Where the approach arrives: the pad's west edge, mid row.
    pub overworld_path_target: IVec3,
    /// Columns of the new lower mass (the same lobe, seen from below).
    pub red_black_extension: ColumnRect,
    /// Navigable columns reserved for the Gate 17 fortress, on the lobe's
    /// underside.
    pub red_black_fortress_pad: ColumnRect,
    /// Lowest cell level of the fortress pad (its -Y face is the ground).
    pub fortress_pad_bottom_y: i32,
    /// The Gate 14 landing cell the lower approach starts from.
    pub red_black_path_anchor: IVec3,
    /// Where the lower approach arrives: the pad's west edge, mid row.
    pub red_black_path_target: IVec3,
    /// Columns west of the fortress pad whose underside steps from the rim
    /// level down (toward +Y) to the pad level, one cell per column.
    pub fortress_terrace_min_x: i32,
    pub fortress_terrace_max_x: i32,
}

impl WorldExpansionLayout {
    pub fn derive(
        terrain: &TerrainConfig,
        rhombus: &RhombusConfig,
        route: &InvertedRouteLayout,
        path: &PathLayout,
        world: &VoxelWorld,
    ) -> Self {
        // The terrain footprint ends at `width`; the lobe starts right
        // after it (independent of anything built later, so the layout is
        // the same before and after the lobe exists).
        let east = terrain.width;
        let _ = world;
        let wall_z = rhombus.portal.wall_z;
        let overworld_extension = ColumnRect::new(
            east,
            wall_z - EXPANSION_DEPTH + 1,
            east + EXPANSION_WIDTH - 1,
            wall_z,
        );
        // The pads keep one column of terrain around them inside the lobe.
        let pad = ColumnRect::new(
            overworld_extension.max_x - PAD_SIDE,
            overworld_extension.min_z + 1,
            overworld_extension.max_x - 1,
            overworld_extension.min_z + PAD_SIDE,
        );
        let mid_z = (pad.min_z + pad.max_z) / 2;
        let castle_pad_surface_y = terrain.base_height;
        let fortress_pad_bottom_y = rhombus.shelf_y - 2;
        // The approach branches from the existing path cell closest to the
        // lobe (the one nearest the pit, at the cutaway's north-west edge).
        let overworld_path_anchor = path
            .cells
            .iter()
            .copied()
            .min_by_key(|c| (overworld_extension.min_x - c.x).abs() + (mid_z - c.z).abs())
            .unwrap_or(IVec3::new(terrain.width / 2, terrain.base_height, wall_z));
        let red_black_path_anchor =
            IVec3::new(route.exit_max_x, route.platform_y, route.platform_z_max);
        Self {
            overworld_extension,
            overworld_castle_pad: pad,
            castle_pad_surface_y,
            overworld_path_anchor,
            overworld_path_target: IVec3::new(pad.min_x, castle_pad_surface_y, mid_z),
            red_black_extension: overworld_extension,
            red_black_fortress_pad: pad,
            fortress_pad_bottom_y,
            red_black_path_anchor,
            red_black_path_target: IVec3::new(pad.min_x, fortress_pad_bottom_y, mid_z),
            fortress_terrace_min_x: overworld_extension.min_x - 1,
            fortress_terrace_max_x: pad.min_x - 1,
        }
    }

    /// Whether a column lies under/over either pad.
    pub fn is_pad_column(&self, x: i32, z: i32) -> bool {
        self.overworld_castle_pad.contains(x, z) || self.red_black_fortress_pad.contains(x, z)
    }

    /// Pad footprint in columns.
    pub fn castle_pad_area(&self) -> i32 {
        (self.overworld_castle_pad.max_x - self.overworld_castle_pad.min_x + 1)
            * (self.overworld_castle_pad.max_z - self.overworld_castle_pad.min_z + 1)
    }

    pub fn fortress_pad_area(&self) -> i32 {
        (self.red_black_fortress_pad.max_x - self.red_black_fortress_pad.min_x + 1)
            * (self.red_black_fortress_pad.max_z - self.red_black_fortress_pad.min_z + 1)
    }

    /// The invariants the layout must satisfy against the existing world:
    /// no overlap with the house, the pond, the portal wall's frame columns
    /// or the cutaway, and pads inside the lobe.
    pub fn validate(&self, rhombus: &RhombusConfig) -> Result<(), String> {
        let house = HOUSE_FOOTPRINT.grown(2);
        for (name, r) in [
            ("extension", self.overworld_extension),
            ("castle pad", self.overworld_castle_pad),
            ("fortress pad", self.red_black_fortress_pad),
        ] {
            for z in r.min_z..=r.max_z {
                for x in r.min_x..=r.max_x {
                    if house.contains(x, z) {
                        return Err(format!("{name} overlaps the house at ({x}, {z})"));
                    }
                    if pond_basin_columns().contains(&(x, z)) {
                        return Err(format!("{name} overlaps the pond at ({x}, {z})"));
                    }
                    if rhombus.cutaway.contains_column(x, z) {
                        return Err(format!("{name} overlaps the cutaway at ({x}, {z})"));
                    }
                    let p = rhombus.portal;
                    if z == p.wall_z && x >= p.min_x && x <= p.max_x {
                        return Err(format!("{name} overlaps the portal at ({x}, {z})"));
                    }
                }
            }
        }
        for (name, pad) in [
            ("castle pad", self.overworld_castle_pad),
            ("fortress pad", self.red_black_fortress_pad),
        ] {
            let inside = pad.min_x >= self.overworld_extension.min_x
                && pad.max_x <= self.overworld_extension.max_x
                && pad.min_z >= self.overworld_extension.min_z
                && pad.max_z <= self.overworld_extension.max_z;
            if !inside {
                return Err(format!("{name} leaves the extension"));
            }
        }
        if self.castle_pad_area() < PAD_SIDE * PAD_SIDE
            || self.fortress_pad_area() < PAD_SIDE * PAD_SIDE
        {
            return Err("a pad is smaller than the reserved footprint".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Overworld mass (C177)
// ---------------------------------------------------------------------

/// Deterministic hash in `[0, 1)` for a column (the expansion's own salt,
/// independent from the terrain's noise).
pub fn expansion_hash(seed: u32, x: i32, z: i32, salt: u32) -> f32 {
    let mut h =
        seed ^ salt ^ (x as u32).wrapping_mul(0x9E37_79B1) ^ (z as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}

const OUTLINE_SALT: u32 = 0x1500_0001;
const RELIEF_SALT: u32 = 0x1500_0002;
const CUT_SALT: u32 = 0x1500_0003;
const EXPOSURE_SALT: u32 = 0x1500_0004;
const RISE_SALT: u32 = 0x1500_0005;
const PAVING_SALT: u32 = 0x1500_0006;
const SURFACE_RELIEF_SALT: u32 = 0x1500_0007;
const FAMILY_SCATTER_SALT: u32 = 0x1500_0008;

/// Layers of the lobe's upper geology: the grass, the dirt below it and
/// the stone under that. Deepslate follows below on the visible ring.
pub const UPPER_STRATA_DEPTH: i32 = 4;

/// One column of the new upper terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpansionColumn {
    pub x: i32,
    pub z: i32,
    /// The grass cell.
    pub surface_y: i32,
    /// On the lobe's outline (its side faces are visible).
    pub ring: bool,
    /// Touches the existing island (the seam).
    pub seam: bool,
}

/// The upper mass as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverworldExpansion {
    pub columns: Vec<ExpansionColumn>,
    pub cells: Vec<IVec3>,
}

impl OverworldExpansion {
    pub fn column(&self, x: i32, z: i32) -> Option<&ExpansionColumn> {
        self.columns.iter().find(|c| c.x == x && c.z == z)
    }

    pub fn surface_cells(&self) -> Vec<IVec3> {
        self.columns
            .iter()
            .map(|c| IVec3::new(c.x, c.surface_y, c.z))
            .collect()
    }
}

/// The easternmost island column of row `z` at the top layers, so the lobe
/// starts one cell beyond it and touches it face to face.
pub fn island_edge_x(layout: &WorldExpansionLayout, world: &VoxelWorld, z: i32) -> Option<i32> {
    let s = layout.castle_pad_surface_y;
    (0..layout.overworld_extension.min_x)
        .rev()
        .find(|&x| ((s - 6)..=(s + 8)).any(|y| world.contains(IVec3::new(x, y, z))))
}

/// The lobe's plan for row `z`: `(first_x, last_x)`. The west end is the
/// seam with the island; the east end is ragged (the last column exists
/// only where the outline hash says so, and the corner rows are shorter).
pub fn lobe_row(
    terrain: &TerrainConfig,
    layout: &WorldExpansionLayout,
    world: &VoxelWorld,
    z: i32,
) -> Option<(i32, i32)> {
    let ext = layout.overworld_extension;
    if z < ext.min_z || z > ext.max_z {
        return None;
    }
    let first = island_edge_x(layout, world, z).map_or(ext.min_x, |x| x + 1);
    let corner = if z == ext.min_z || z == ext.max_z {
        2
    } else if z == ext.min_z + 1 || z == ext.max_z - 1 {
        1
    } else {
        0
    };
    let ragged = if expansion_hash(terrain.seed, ext.max_x, z, OUTLINE_SALT) < 0.5 {
        1
    } else {
        0
    };
    let last = (ext.max_x - corner - ragged)
        .max(layout.overworld_castle_pad.max_x.min(ext.max_x - corner));
    let last = if layout.overworld_castle_pad.min_z <= z && z <= layout.overworld_castle_pad.max_z {
        last.max(layout.overworld_castle_pad.max_x)
    } else {
        last
    };
    Some((first, last))
}

/// Grows the upper island toward the castle site: for every column of the
/// lobe, grass at the surface, dirt and stone below, and deepslate on the
/// ring columns whose flanks are visible. The pad columns are level at
/// `castle_pad_surface_y`; the rest of the lobe varies by one cell around
/// it, and the seam matches the island's own edge height where the island
/// is lower. The interior of the shell is left hollow: nothing there can
/// ever be seen.
pub fn build_overworld_expansion(
    terrain: &TerrainConfig,
    layout: &WorldExpansionLayout,
    world: &mut VoxelWorld,
) -> OverworldExpansion {
    let mut expansion = OverworldExpansion::default();
    let ext = layout.overworld_extension;
    let s = layout.castle_pad_surface_y;
    let mut rows: Vec<(i32, i32, i32)> = Vec::new();
    for z in ext.min_z..=ext.max_z {
        if let Some((first, last)) = lobe_row(terrain, layout, world, z) {
            rows.push((z, first, last));
        }
    }
    let row_of = |z: i32| rows.iter().find(|r| r.0 == z).map(|r| (r.1, r.2));
    let top = terrain.max_surface_height() + 16;
    let bottom = terrain.deepslate_level - 8;
    for &(z, first, last) in &rows {
        for x in first..=last {
            let pad = layout.overworld_castle_pad.contains(x, z);
            let seam = x == first;
            let ring = seam
                || x == last
                || row_of(z - 1).is_none_or(|(f, l)| x < f || x > l)
                || row_of(z + 1).is_none_or(|(f, l)| x < f || x > l);
            // Height: the pad is level; elsewhere one cell of variation,
            // and the seam follows a lower island neighbour.
            let mut surface_y = if pad {
                s
            } else if expansion_hash(terrain.seed, x, z, RELIEF_SALT) < 0.3 {
                s - 1
            } else {
                s
            };
            if seam && let Some(neighbour) = column_top(world, x - 1, z, top, bottom) {
                surface_y = surface_y.min(neighbour + 1).max(s - 1);
            }
            expansion.columns.push(ExpansionColumn {
                x,
                z,
                surface_y,
                ring,
                seam,
            });
            // Ring columns show their flank: grass, dirt, dirt, stone,
            // deepslate. Interior columns are never seen from the side, so
            // they carry only what their top exposes (grass over dirt).
            let depth = if ring {
                UPPER_STRATA_DEPTH + 1
            } else {
                UPPER_STRATA_DEPTH - 1
            };
            for d in 0..depth {
                let y = surface_y - d;
                let cell = IVec3::new(x, y, z);
                if world.contains(cell) {
                    continue;
                }
                let block_type = if d == 0 {
                    BlockType::Grass
                } else if d <= 2 {
                    BlockType::Dirt
                } else if d == 3 {
                    BlockType::Stone
                } else {
                    BlockType::Deepslate
                };
                let material = if block_type == BlockType::Deepslate {
                    deepslate_material_id()
                } else {
                    stratum_material(block_type)
                };
                world.insert(
                    cell,
                    BlockInstance::new(block_type, material, Orientation::Up),
                );
                expansion.cells.push(cell);
            }
        }
    }
    expansion
}

// ---------------------------------------------------------------------
// Relief and edges (C178)
// ---------------------------------------------------------------------

/// The columns of the approach corridor from the existing path to the
/// castle pad: two rows east along the cutaway's north edge, then two
/// columns north along the pad's west side, ending at the pad's west edge
/// mid row. Kept free of relief so it stays a navigable route (C179 paves
/// it).
pub fn approach_columns(layout: &WorldExpansionLayout) -> Vec<(i32, i32)> {
    let mut cols = Vec::new();
    let anchor = layout.overworld_path_anchor;
    let pad = layout.overworld_castle_pad;
    let target = layout.overworld_path_target;
    // Segment A: east along rows `anchor.z - 1 ..= anchor.z`.
    for x in (anchor.x + 1)..pad.min_x {
        for z in (anchor.z - 1)..=anchor.z {
            cols.push((x, z));
        }
    }
    // Segment B: north along the two columns west of the pad, from the
    // corridor rows up to the target row (plus one for the forecourt).
    for x in (pad.min_x - 2)..pad.min_x {
        for z in (target.z - 1)..(anchor.z - 1) {
            if !cols.contains(&(x, z)) {
                cols.push((x, z));
            }
        }
    }
    cols
}

/// What the relief pass did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverworldRelief {
    /// Ring columns lowered by one (stepped edge).
    pub terraces: Vec<IVec3>,
    /// Columns whose grass was removed to expose the dirt (the new top).
    pub dirt_cuts: Vec<IVec3>,
    /// Dirt cells of visible flanks replaced by stone.
    pub stone_exposures: Vec<IVec3>,
    /// Columns raised by one grass cell (a small rise).
    pub rises: Vec<IVec3>,
}

fn is_corridor(layout: &WorldExpansionLayout, x: i32, z: i32) -> bool {
    approach_columns(layout).contains(&(x, z))
}

/// Sculpts the new upper terrain: the outer ring steps down one cell
/// (terraces along the edges), a few columns lose their grass to a dirt
/// cut, the visible flanks show stone where the dirt was, and a handful of
/// interior columns rise by one. The castle pad and the approach corridor
/// are never touched, no column ends more than one cell above all of its
/// neighbours, and every top keeps open air above it.
pub fn shape_overworld_relief(
    terrain: &TerrainConfig,
    layout: &WorldExpansionLayout,
    expansion: &mut OverworldExpansion,
    world: &mut VoxelWorld,
) -> OverworldRelief {
    let mut relief = OverworldRelief::default();
    let pad = layout.overworld_castle_pad;
    let s = layout.castle_pad_surface_y;
    let n = expansion.columns.len();
    for i in 0..n {
        let c = expansion.columns[i];
        if pad.contains(c.x, c.z) || is_corridor(layout, c.x, c.z) {
            continue;
        }
        let top = IVec3::new(c.x, c.surface_y, c.z);
        // 1. Terraces: the ring (not the seam) steps down to `s - 1`.
        if c.ring && !c.seam && c.surface_y > s - 1 {
            world.remove(top);
            // The cell below becomes the new grass top.
            let below = IVec3::new(c.x, c.surface_y - 1, c.z);
            world.insert(
                below,
                BlockInstance::new(
                    BlockType::Grass,
                    stratum_material(BlockType::Grass),
                    Orientation::Up,
                ),
            );
            expansion.cells.retain(|cell| *cell != top);
            expansion.columns[i].surface_y = c.surface_y - 1;
            relief.terraces.push(below);
            continue;
        }
        // 2. Dirt cuts: a few interior columns lose their grass.
        if !c.ring && expansion_hash(terrain.seed, c.x, c.z, CUT_SALT) < 0.2 {
            world.remove(top);
            expansion.cells.retain(|cell| *cell != top);
            let below = IVec3::new(c.x, c.surface_y - 1, c.z);
            expansion.columns[i].surface_y = c.surface_y - 1;
            relief.dirt_cuts.push(below);
            continue;
        }
        // 3. Rises: a few interior columns at the pad level gain a grass cell.
        if !c.ring && c.surface_y == s && expansion_hash(terrain.seed, c.x, c.z, RISE_SALT) < 0.12 {
            world.insert(
                top,
                BlockInstance::new(
                    BlockType::Dirt,
                    stratum_material(BlockType::Dirt),
                    Orientation::Up,
                ),
            );
            let above = IVec3::new(c.x, c.surface_y + 1, c.z);
            world.insert(
                above,
                BlockInstance::new(
                    BlockType::Grass,
                    stratum_material(BlockType::Grass),
                    Orientation::Up,
                ),
            );
            expansion.cells.push(above);
            expansion.columns[i].surface_y = c.surface_y + 1;
            relief.rises.push(above);
        }
    }
    // 4. Stone exposures on the visible flanks: dirt one or two below the
    //    grass of ring columns becomes stone where the hash says so.
    for c in expansion.columns.clone() {
        if !c.ring || pad.contains(c.x, c.z) {
            continue;
        }
        let roll = expansion_hash(terrain.seed, c.x, c.z, EXPOSURE_SALT);
        if roll < 0.35 {
            let d = if roll < 0.15 { 1 } else { 2 };
            let cell = IVec3::new(c.x, c.surface_y - d, c.z);
            if world.get(cell).map(|b| b.block_type()) == Some(BlockType::Dirt) {
                world.insert(
                    cell,
                    BlockInstance::new(
                        BlockType::Stone,
                        stratum_material(BlockType::Stone),
                        Orientation::Up,
                    ),
                );
                relief.stone_exposures.push(cell);
            }
        }
    }
    // 5. No column may stand two above every neighbour: lower such spikes.
    let heights: std::collections::HashMap<(i32, i32), i32> = expansion
        .columns
        .iter()
        .map(|c| ((c.x, c.z), c.surface_y))
        .collect();
    for i in 0..n {
        let c = expansion.columns[i];
        if pad.contains(c.x, c.z) {
            continue;
        }
        let neighbours: Vec<i32> = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter_map(|(dx, dz)| heights.get(&(c.x + dx, c.z + dz)).copied())
            .collect();
        if !neighbours.is_empty() && neighbours.iter().all(|h| c.surface_y >= h + 2) {
            let top = IVec3::new(c.x, c.surface_y, c.z);
            world.remove(top);
            expansion.cells.retain(|cell| *cell != top);
            let below = IVec3::new(c.x, c.surface_y - 1, c.z);
            world.insert(
                below,
                BlockInstance::new(
                    BlockType::Grass,
                    stratum_material(BlockType::Grass),
                    Orientation::Up,
                ),
            );
            expansion.columns[i].surface_y = c.surface_y - 1;
            relief.terraces.push(below);
        }
    }
    relief
}

// ---------------------------------------------------------------------
// Approach path (C179)
// ---------------------------------------------------------------------

/// The paved approach from the existing path to the castle site.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverworldApproach {
    /// Paved cells of the main route (cobblestone and stone), in walking
    /// order from the anchor to the pad's west edge.
    pub main: Vec<IVec3>,
    /// The forecourt in front of the pad (outside it).
    pub forecourt: Vec<IVec3>,
    /// Timber markers beside the route (fence posts and a log or two).
    pub accents: Vec<IVec3>,
}

impl OverworldApproach {
    pub fn cells(&self) -> Vec<IVec3> {
        self.main
            .iter()
            .chain(self.forecourt.iter())
            .copied()
            .collect()
    }
}

fn is_ground(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::Grass
            | BlockType::Dirt
            | BlockType::Stone
            | BlockType::Sand
            | BlockType::Cobblestone
    )
}

/// The highest ground cell of a column (leaves and trunks above it do not
/// count: the path runs under the canopy).
pub fn ground_cell(world: &VoxelWorld, x: i32, z: i32, top: i32, bottom: i32) -> Option<IVec3> {
    (bottom..=top)
        .rev()
        .map(|y| IVec3::new(x, y, z))
        .find(|c| world.get(*c).is_some_and(|b| is_ground(b.block_type())))
}

/// Paves the approach corridor: every corridor column's ground cell
/// becomes cobblestone (stone every few cells), two rows wide east along
/// the cutaway's north edge and two columns wide north to the pad's west
/// edge; a three-by-three forecourt of cobblestone stands in front of the
/// pad (outside it); two fence posts flank the route's start and a log
/// marks each forecourt corner. Only ground cells are replaced, so the
/// voxel count barely moves and nothing of the house, pond, trees, portal
/// or descent is touched.
pub fn pave_overworld_approach(
    terrain: &TerrainConfig,
    layout: &WorldExpansionLayout,
    world: &mut VoxelWorld,
) -> OverworldApproach {
    let mut approach = OverworldApproach::default();
    let top = terrain.max_surface_height() + 16;
    let bottom = terrain.deepslate_level - 8;
    let pave = |world: &mut VoxelWorld, cell: IVec3, stone: bool| {
        let (block_type, material) = if stone {
            (BlockType::Stone, stratum_material(BlockType::Stone))
        } else {
            (BlockType::Cobblestone, cobblestone_material_id())
        };
        world.insert(
            cell,
            BlockInstance::new(block_type, material, Orientation::Up),
        );
    };
    for (x, z) in approach_columns(layout) {
        let Some(cell) = ground_cell(world, x, z, top, bottom) else {
            continue;
        };
        let stone = expansion_hash(terrain.seed, x, z, PAVING_SALT) < 0.2;
        pave(world, cell, stone);
        approach.main.push(cell);
    }
    // Forecourt: three columns west of the pad around the target row.
    let pad = layout.overworld_castle_pad;
    let target = layout.overworld_path_target;
    for z in (target.z - 1)..=(target.z + 1) {
        for x in (pad.min_x - 3)..pad.min_x {
            let Some(cell) = ground_cell(world, x, z, top, bottom) else {
                continue;
            };
            if !approach.main.contains(&cell) {
                pave(world, cell, false);
                approach.forecourt.push(cell);
            }
        }
    }
    // Accents: fence posts flanking the start of the corridor (north side,
    // beside the route) and a log at each forecourt corner.
    let anchor = layout.overworld_path_anchor;
    for x in [anchor.x + 2, anchor.x + 9] {
        let z = anchor.z - 2;
        if let Some(ground) = ground_cell(world, x, z, top, bottom) {
            let post = IVec3::new(x, ground.y + 1, z);
            if !world.contains(post) {
                world.insert(
                    post,
                    BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::Up),
                );
                approach.accents.push(post);
            }
        }
    }
    for z in [target.z - 2, target.z + 2] {
        let x = pad.min_x - 3;
        if let Some(ground) = ground_cell(world, x, z, top, bottom) {
            let log = IVec3::new(x, ground.y + 1, z);
            if !world.contains(log) {
                world.insert(
                    log,
                    BlockInstance::new(BlockType::Log, log_material_id(), Orientation::Up),
                );
                approach.accents.push(log);
            }
        }
    }
    approach
}

// ---------------------------------------------------------------------
// Red-Black mass (C180)
// ---------------------------------------------------------------------

/// One column of the lobe's lower shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowerColumn {
    pub x: i32,
    pub z: i32,
    /// The lowest cell of the column: its -Y face is Red-Black ground.
    pub bottom_y: i32,
    /// On the lower shell's outline (its flank is visible).
    pub ring: bool,
}

/// The lower half of the lobe as built: an underside layer that steps from
/// the rim level of the existing lower mass down (toward +Y, the inverted
/// viewer's down) to the fortress pad level, and dark walls on the
/// outline up to the Overworld strata. The interior stays hollow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackExpansion {
    pub columns: Vec<LowerColumn>,
    /// The underside cells (one per column).
    pub underside: Vec<IVec3>,
    /// The wall cells of the outline.
    pub walls: Vec<IVec3>,
}

impl RedBlackExpansion {
    pub fn column(&self, x: i32, z: i32) -> Option<&LowerColumn> {
        self.columns.iter().find(|c| c.x == x && c.z == z)
    }

    pub fn cells(&self) -> Vec<IVec3> {
        self.underside
            .iter()
            .chain(self.walls.iter())
            .copied()
            .collect()
    }
}

/// The underside level of lobe column `x`: the rim level of the existing
/// lower mass west of the terrace, one cell lower (toward +Y) per terrace
/// column, and the fortress pad level from the pad's west edge on.
pub fn lower_bottom_y(layout: &WorldExpansionLayout, x: i32) -> i32 {
    let pad = layout.fortress_pad_bottom_y;
    let steps = layout.fortress_terrace_max_x - layout.fortress_terrace_min_x + 1;
    if x > layout.fortress_terrace_max_x {
        pad
    } else if x < layout.fortress_terrace_min_x {
        pad - steps - 1
    } else {
        pad - (layout.fortress_terrace_max_x - x) - 1
    }
}

/// The easternmost cell of the existing lower mass at the rim level for
/// row `z`, so the lower shell starts right after it and its underside
/// continues the rim.
pub fn lower_rim_x(layout: &WorldExpansionLayout, world: &VoxelWorld, z: i32) -> Option<i32> {
    let rim = lower_bottom_y(layout, layout.fortress_terrace_min_x - 1);
    (0..layout.red_black_extension.min_x)
        .rev()
        .find(|&x| ((rim - 2)..=(rim + 3)).any(|y| world.contains(IVec3::new(x, y, z))))
}

/// Builds the lobe's lower shell under the upper terrain of
/// `expansion`: for every lower column an underside cell at
/// `lower_bottom_y`, and on the outline a wall of the same deepslate,
/// basalt and polished blackstone mix as the certified lower mass, from the
/// underside up to the island's own cells or the Overworld strata. All
/// cells are `Down`-oriented (the inverted realm's functional up is -Y).
pub fn build_red_black_expansion(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    layout: &WorldExpansionLayout,
    upper: &OverworldExpansion,
    world: &mut VoxelWorld,
) -> RedBlackExpansion {
    let mut lower = RedBlackExpansion::default();
    let ext = layout.red_black_extension;
    // Plan: row `z` runs from the lower rim (or the upper row's first
    // column, whichever is further west) to the upper row's last column.
    let mut rows: Vec<(i32, i32, i32)> = Vec::new();
    for z in ext.min_z..=ext.max_z {
        let upper_cols: Vec<i32> = upper
            .columns
            .iter()
            .filter(|c| c.z == z)
            .map(|c| c.x)
            .collect();
        let Some(&last) = upper_cols.iter().max() else {
            continue;
        };
        let upper_first = *upper_cols.iter().min().unwrap();
        let first = lower_rim_x(layout, world, z).map_or(upper_first, |x| (x + 1).min(upper_first));
        rows.push((z, first, last));
    }
    let row_of = |z: i32| rows.iter().find(|r| r.0 == z).map(|r| (r.1, r.2));
    let top_cap = terrain.deepslate_level;
    for &(z, first, last) in &rows {
        for x in first..=last {
            let bottom_y = lower_bottom_y(layout, x);
            let ring = x == first
                || x == last
                || row_of(z - 1).is_none_or(|(f, l)| x < f || x > l)
                || row_of(z + 1).is_none_or(|(f, l)| x < f || x > l);
            lower.columns.push(LowerColumn {
                x,
                z,
                bottom_y,
                ring,
            });
            let underside = IVec3::new(x, bottom_y, z);
            if !world.contains(underside) {
                let (block_type, material) = lower_mass_block(terrain, rhombus, x, bottom_y, z);
                world.insert(
                    underside,
                    BlockInstance::new(block_type, material, Orientation::Down),
                );
                lower.underside.push(underside);
            }
            if !ring {
                continue;
            }
            // Wall: from above the underside up to the cell below the
            // column's lowest existing cell (island skirt or Overworld
            // strata), capped at the deepslate level.
            let lowest_existing =
                ((bottom_y + 1)..=(top_cap + 8)).find(|&y| world.contains(IVec3::new(x, y, z)));
            let wall_top = lowest_existing.map_or(top_cap, |y| (y - 1).min(top_cap));
            for y in (bottom_y + 1)..=wall_top {
                let cell = IVec3::new(x, y, z);
                if world.contains(cell) {
                    continue;
                }
                let (block_type, material) = lower_mass_block(terrain, rhombus, x, y, z);
                world.insert(
                    cell,
                    BlockInstance::new(block_type, material, Orientation::Down),
                );
                lower.walls.push(cell);
            }
        }
    }
    lower
}

// ---------------------------------------------------------------------
// Red-Black surface (C181)
// ---------------------------------------------------------------------

/// Share of lower columns (outside the pad) whose ground steps one cell
/// further from the mass (relief of the underside).
pub const LOWER_SURFACE_RELIEF_SHARE: f32 = 0.15;
/// Share of surface cells that take a family brick of a family other than
/// the sector's own, so all three families read across the expansion.
pub const FAMILY_SCATTER_SHARE: f32 = 0.10;

/// The new -Y-facing surface as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackSurfaceExtension {
    /// The ground cell of every lower column (all `Down`).
    pub cells: Vec<IVec3>,
    /// Cells added one further from the mass (the raised relief).
    pub raised: Vec<IVec3>,
}

/// Covers the lower shell with Red-Black ground: every underside cell
/// becomes a `Down` surface block from the same deterministic mix the
/// certified underside uses (mycelium, smooth basalt, polished blackstone,
/// the sector's family bricks), with a restrained scatter of the other two
/// families' bricks so the expansion reads as one blended dark surface
/// rather than colour rectangles. Outside the pad a share of columns gains
/// one more cell toward -Y (relief); the pad stays level.
pub fn extend_red_black_surface(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    layout: &WorldExpansionLayout,
    lower: &mut RedBlackExpansion,
    world: &mut VoxelWorld,
) -> RedBlackSurfaceExtension {
    let mut surface = RedBlackSurfaceExtension::default();
    let pad = layout.red_black_fortress_pad;
    for i in 0..lower.columns.len() {
        let c = lower.columns[i];
        let mut ground = IVec3::new(c.x, c.bottom_y, c.z);
        if !pad.contains(c.x, c.z)
            && expansion_hash(terrain.seed, c.x, c.z, SURFACE_RELIEF_SALT)
                < LOWER_SURFACE_RELIEF_SHARE
        {
            let deeper = IVec3::new(c.x, c.bottom_y - 1, c.z);
            if !world.contains(deeper) {
                // The old ground stays as structure; the new cell is the ground.
                let (block_type, material) =
                    lower_mass_block(terrain, rhombus, c.x, c.bottom_y, c.z);
                world.insert(
                    ground,
                    BlockInstance::new(block_type, material, Orientation::Down),
                );
                ground = deeper;
                lower.columns[i].bottom_y = deeper.y;
                surface.raised.push(deeper);
            }
        }
        let (mut block_type, mut material) = surface_block(terrain, rhombus, c.x, c.z);
        let scatter = expansion_hash(terrain.seed, c.x, c.z, FAMILY_SCATTER_SALT);
        if scatter < FAMILY_SCATTER_SHARE {
            let family = if scatter < FAMILY_SCATTER_SHARE / 3.0 {
                Family::Crimson
            } else if scatter < 2.0 * FAMILY_SCATTER_SHARE / 3.0 {
                Family::Violet
            } else {
                Family::Orange
            };
            let bricks = family.bricks();
            block_type = bricks.0;
            material = bricks.1;
        }
        world.insert(
            ground,
            BlockInstance::new(block_type, material, Orientation::Down),
        );
        surface.cells.push(ground);
    }
    surface
}
