// The spatial contract of the final diorama: one floating diamond whose
// upper half is the approved Overworld and whose lower half is the inverted
// Red-Black world, joined at a narrow waist that holds the portal. Every
// coordinate here is derived from the Gate 12 terrain (footprint, mass
// bottom, reserved descent endpoint) so the upper scene is never rebuilt;
// this module only names where the remaining composition goes.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::{
    DESCENT_ENDPOINT, HOUSE_FOOTPRINT, PATH_RESERVE, pond_basin_columns,
};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::generator::{footprint_radius, mass_bottom};

/// The quadrant of columns omitted from the diorama so its interior reads
/// as a physical cutaway: every column with `x >= min_x` and `z >= min_z`
/// (the south-east corner, toward the camera).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutawayRegion {
    pub min_x: i32,
    pub min_z: i32,
}

impl CutawayRegion {
    pub fn contains_column(&self, x: i32, z: i32) -> bool {
        x >= self.min_x && z >= self.min_z
    }
}

/// Where the portal stands: a vertical frame in the north wall of the
/// cutaway (`z == wall_z`), spanning `min_x..=max_x` and `min_y..=max_y`,
/// facing the open side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalAnchor {
    pub wall_z: i32,
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub facing: Orientation,
}

impl PortalAnchor {
    pub fn width(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn height(&self) -> i32 {
        self.max_y - self.min_y + 1
    }

    /// Column at the middle of the frame.
    pub fn center_x(&self) -> i32 {
        (self.min_x + self.max_x) / 2
    }

    /// `true` for the empty cells inside the frame (where the core goes).
    pub fn is_opening(&self, x: i32, y: i32) -> bool {
        x > self.min_x && x < self.max_x && y > self.min_y && y < self.max_y
    }

    /// `true` for the cells of the frame itself.
    pub fn is_frame(&self, x: i32, y: i32) -> bool {
        x >= self.min_x
            && x <= self.max_x
            && y >= self.min_y
            && y <= self.max_y
            && !self.is_opening(x, y)
    }
}

/// The macro layout of the diamond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhombusConfig {
    /// Column of the footprint center (also the portal's and tip's axis).
    pub center_x: i32,
    pub center_z: i32,
    /// Highest Overworld surface cell (the grass on the tallest hill).
    pub upper_surface_reference: i32,
    /// Lowest cell of the Gate 12 terrain at the center of the footprint;
    /// the upper taper starts one row under it.
    pub upper_mass_bottom: i32,
    /// Narrowest row of the diamond: the middle of the portal.
    pub waist_y: i32,
    /// Row of the cutaway floor (the portal sill and the ledge the upper
    /// descent lands on); the lower cutaway resumes one row under it.
    pub shelf_y: i32,
    /// Widest row of the lower half.
    pub lower_widest_y: i32,
    /// Lowest cell of the diamond.
    pub lower_tip_y: i32,
    pub cutaway: CutawayRegion,
    pub portal: PortalAnchor,
    /// Column where the upper descent begins: the Gate 12 path endpoint.
    pub upper_descent_anchor: (i32, i32),
    /// First cell of the inverted descent, hanging under the shelf just
    /// inside the cutaway.
    pub lower_descent_anchor: IVec3,
}

/// Normalized footprint radius (see `footprint_radius`) of the Gate 12
/// terrain's innermost bottom row; the upper taper continues from it.
pub const UPPER_TAPER_START_RADIUS: f32 = 0.55;

/// Radius at the waist.
pub const WAIST_RADIUS: f32 = 0.50;

/// Radius at the widest row of the lower half.
pub const LOWER_WIDEST_RADIUS: f32 = 0.85;

/// Radius at the tip: a single cell.
pub const LOWER_TIP_RADIUS: f32 = 0.02;

impl RhombusConfig {
    /// The official layout, derived from the official terrain.
    pub fn official() -> Self {
        Self::derive(&TerrainConfig::official())
    }

    /// Derives the layout from `terrain`: center and surface from the
    /// footprint, the taper from its mass bottom, the descent start from the
    /// reserved path endpoint, and the portal in the cutaway's north wall
    /// just south of the footprint center at the waist.
    pub fn derive(terrain: &TerrainConfig) -> Self {
        let center_x = terrain.width / 2;
        let center_z = terrain.depth / 2;
        let bottom = mass_bottom(terrain);
        let waist_y = bottom - 4;
        let shelf_y = waist_y - 3;
        let cutaway = CutawayRegion {
            min_x: center_x + 3,
            min_z: center_z + 3,
        };
        let portal = PortalAnchor {
            wall_z: cutaway.min_z - 1,
            min_x: cutaway.min_x - 1,
            max_x: cutaway.min_x + 3,
            min_y: shelf_y,
            max_y: shelf_y + 5,
            facing: Orientation::South,
        };
        Self {
            center_x,
            center_z,
            upper_surface_reference: terrain.max_surface_height(),
            upper_mass_bottom: bottom,
            waist_y,
            shelf_y,
            lower_widest_y: shelf_y - 6,
            lower_tip_y: shelf_y - 12,
            cutaway,
            portal,
            upper_descent_anchor: DESCENT_ENDPOINT,
            lower_descent_anchor: IVec3::new(cutaway.min_x, shelf_y - 1, cutaway.min_z),
        }
    }

    /// Normalized footprint radius of the diamond's section at row `y`:
    /// the terrain's own footprint above the mass bottom, a taper down to
    /// `WAIST_RADIUS` at the waist, a widening to `LOWER_WIDEST_RADIUS` and a
    /// taper to a single cell at `lower_tip_y`. `None` outside the diamond.
    pub fn section_radius(&self, y: i32) -> Option<f32> {
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        if y > self.upper_surface_reference || y < self.lower_tip_y {
            None
        } else if y >= self.upper_mass_bottom {
            Some(1.0)
        } else if y >= self.waist_y {
            let t = (self.upper_mass_bottom - y) as f32
                / (self.upper_mass_bottom - self.waist_y) as f32;
            Some(lerp(UPPER_TAPER_START_RADIUS, WAIST_RADIUS, t))
        } else if y >= self.lower_widest_y {
            let t = (self.waist_y - y) as f32 / (self.waist_y - self.lower_widest_y) as f32;
            Some(lerp(WAIST_RADIUS, LOWER_WIDEST_RADIUS, t))
        } else {
            let t =
                (self.lower_widest_y - y) as f32 / (self.lower_widest_y - self.lower_tip_y) as f32;
            Some(lerp(LOWER_WIDEST_RADIUS, LOWER_TIP_RADIUS, t * t.sqrt()))
        }
    }

    /// `true` when column `(x, z)` lies inside the diamond's section at row
    /// `y` (ignoring the cutaway).
    pub fn contains(&self, terrain: &TerrainConfig, x: i32, y: i32, z: i32) -> bool {
        match self.section_radius(y) {
            Some(radius) => footprint_radius(terrain, x, z) <= radius,
            None => false,
        }
    }

    /// Number of columns of the section at row `y`.
    pub fn section_area(&self, terrain: &TerrainConfig, y: i32) -> usize {
        (-4..terrain.depth + 4)
            .flat_map(|z| (-4..terrain.width + 4).map(move |x| (x, z)))
            .filter(|&(x, z)| self.contains(terrain, x, y, z))
            .count()
    }

    /// Width (along `x`) of the section at row `y` through the center.
    pub fn section_width(&self, terrain: &TerrainConfig, y: i32) -> i32 {
        (-4..terrain.width + 4)
            .filter(|&x| self.contains(terrain, x, y, self.center_z))
            .count() as i32
    }

    /// `true` when the cell belongs to the omitted cutaway: the south-east
    /// quadrant above the shelf (upper cutaway) or under it (lower cutaway);
    /// the shelf row itself stays.
    pub fn is_cut(&self, x: i32, y: i32, z: i32) -> bool {
        self.cutaway.contains_column(x, z) && y != self.shelf_y
    }

    /// Every invariant of the contract, as `Err(reason)` when broken.
    pub fn validate(&self, terrain: &TerrainConfig) -> Result<(), String> {
        let p = self.portal;
        if self.waist_y >= self.upper_mass_bottom {
            return Err("the waist must lie under the terrain".into());
        }
        if self.lower_tip_y >= self.waist_y || self.lower_widest_y >= self.waist_y {
            return Err("the lower half must lie under the waist".into());
        }
        if self.lower_tip_y >= self.lower_widest_y {
            return Err("the tip must lie under the widest lower row".into());
        }
        if p.max_y >= terrain.min_surface_height() {
            return Err("the portal must lie under the surface".into());
        }
        if !(p.min_y..=p.max_y).contains(&self.waist_y) {
            return Err("the waist must cross the portal".into());
        }
        for x in p.min_x..=p.max_x {
            for y in p.min_y..=p.max_y {
                if !self.contains(terrain, x, y, p.wall_z) {
                    return Err(format!("portal cell ({x}, {y}) outside the diamond"));
                }
            }
        }
        if p.width() < 3 || p.height() < 4 {
            return Err("the portal needs an opening".into());
        }
        if !self.cutaway.contains_column(p.min_x + 1, p.wall_z + 1) {
            return Err("the portal must face the cutaway".into());
        }
        let (ex, ez) = self.upper_descent_anchor;
        if self.cutaway.contains_column(ex, ez) {
            return Err("the descent endpoint must stay on solid ground".into());
        }
        if !PATH_RESERVE.contains(ex, ez) {
            return Err("the descent endpoint must be the reserved one".into());
        }
        for (x, z) in pond_basin_columns() {
            if self.cutaway.contains_column(x, z) {
                return Err("the cutaway must not touch the pond".into());
            }
        }
        let h = HOUSE_FOOTPRINT.grown(1);
        for z in h.min_z..=h.max_z {
            for x in h.min_x..=h.max_x {
                if self.cutaway.contains_column(x, z) {
                    return Err("the cutaway must not touch the house".into());
                }
            }
        }
        Ok(())
    }
}
