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
use crate::scene::descent::InvertedRouteLayout;
use crate::scene::overworld::{ColumnRect, HOUSE_FOOTPRINT, PathLayout, pond_basin_columns};
use crate::scene::rhombus::RhombusConfig;
use crate::scene::terrain::TerrainConfig;
use crate::scene::voxel_world::VoxelWorld;

/// Columns the lobe adds east of the current bounds.
pub const EXPANSION_WIDTH: i32 = 12;
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
        let east = world
            .bounds()
            .map(|b| b.max_exclusive.x)
            .unwrap_or(terrain.width);
        let wall_z = rhombus.portal.wall_z;
        let overworld_extension = ColumnRect::new(
            east,
            wall_z - EXPANSION_DEPTH + 1,
            east + EXPANSION_WIDTH - 1,
            wall_z,
        );
        // The pads keep one column of terrain around them inside the lobe.
        let pad = ColumnRect::new(
            overworld_extension.max_x - PAD_SIDE + 1,
            overworld_extension.min_z + 1,
            overworld_extension.max_x,
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
