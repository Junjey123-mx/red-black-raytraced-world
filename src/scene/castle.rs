//! Gate 16: the compact Overworld castle on the Gate 15 castle pad.
//!
//! Everything here is derived from `WorldExpansionLayout`: the castle pad
//! gives the footprint, the approach path gives the gate, and the four
//! corners give the towers. The layout (C185) is a contract; the builders
//! that follow place blocks stage by stage.

#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::expansion::{WorldExpansionLayout, expansion_hash};
use crate::scene::material_gallery::deepslate_bricks_material_id;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::{ColumnRect, HOUSE_FOOTPRINT};
use crate::scene::overworld_blocks::{cobblestone_material_id, stone_block_material_id};
use crate::scene::terrain::TerrainConfig;
use crate::scene::voxel_world::VoxelWorld;

/// Side of each square corner tower (columns).
pub const TOWER_SIDE: i32 = 3;
/// Courses of the outer curtain wall above the ground.
pub const WALL_HEIGHT: i32 = 3;
/// Courses of the keep shell above the ground (two floors and a roof).
pub const KEEP_HEIGHT: i32 = 6;
/// Courses of each tower shell above the ground.
pub const TOWER_HEIGHT: i32 = 7;
/// Courses from the ground floor to the upper floor of the keep.
pub const UPPER_FLOOR_RISE: i32 = 3;
/// Rows of the gate opening (two wide for the collision volume).
pub const GATE_WIDTH: i32 = 2;
/// Height of the door and of every passage (two cells of head room).
pub const PASSAGE_HEIGHT: i32 = 2;
/// Columns the gatehouse reaches into the courtyard.
pub const GATEHOUSE_DEPTH: i32 = 2;
/// Width of the keep (columns along x, outer walls included).
pub const KEEP_WIDTH: i32 = 6;

/// A door: two cells, one above the other, in a wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Doorway {
    /// The lower cell (the ground is one below).
    pub base: IVec3,
    /// Columns (rows along z) the opening spans.
    pub width: i32,
    /// Which way the door faces (out of the building).
    pub facing: Orientation,
}

impl Doorway {
    /// Every cell of the opening.
    pub fn cells(&self) -> Vec<IVec3> {
        (0..self.width)
            .flat_map(|dz| {
                (0..PASSAGE_HEIGHT)
                    .map(move |dy| IVec3::new(self.base.x, self.base.y + dy, self.base.z + dz))
            })
            .collect()
    }

    /// Columns of the opening.
    pub fn columns(&self) -> Vec<(i32, i32)> {
        (0..self.width)
            .map(|dz| (self.base.x, self.base.z + dz))
            .collect()
    }
}

/// The vertical contract of the castle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorLevels {
    /// The pad surface: foundation and floors replace this row.
    pub ground_y: i32,
    /// First course above the ground.
    pub base_y: i32,
    /// Top course of the curtain wall; merlons stand one above.
    pub wall_top_y: i32,
    /// The keep's upper floor (planks).
    pub upper_floor_y: i32,
    /// The keep roof; merlons stand one above.
    pub keep_roof_y: i32,
    /// Top course of the towers; merlons stand one above.
    pub tower_top_y: i32,
    /// Highest cell of the castle (tower merlons).
    pub max_y: i32,
}

/// One stair tread of the interior stair.
pub type Tread = (IVec3, Orientation);

/// Where the castle goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverworldCastleLayout {
    /// The castle pad: nothing of the castle stands outside it.
    pub footprint: ColumnRect,
    pub levels: FloorLevels,
    /// The gate in the west wall, facing the Gate 15 approach.
    pub gate: Doorway,
    /// The gatehouse block around the gate (walls, passage, roof).
    pub gatehouse: ColumnRect,
    /// The keep (outer walls included); its east wall is the curtain wall.
    pub keep: ColumnRect,
    /// The keep's door, from the courtyard.
    pub keep_door: Doorway,
    /// The keep's interior columns (the hall below, the room above).
    pub hall: ColumnRect,
    /// The four corner towers, outer walls included.
    pub towers: Vec<ColumnRect>,
    /// Open columns inside the walls (not keep, tower or gatehouse).
    pub courtyard: Vec<(i32, i32)>,
    /// Treads of the interior stair, from the hall up to the upper floor.
    pub stair: Vec<Tread>,
    /// Columns of the upper floor left open above the stair.
    pub stair_well: Vec<(i32, i32)>,
    /// Window cells (glass) in the keep and towers.
    pub windows: Vec<IVec3>,
    /// Ground cells of the canonical route: forecourt, gate, courtyard,
    /// keep door, hall, stair, upper landing.
    pub interior_route: Vec<IVec3>,
}

impl OverworldCastleLayout {
    /// Derives the castle from the Gate 15 pad and approach.
    pub fn derive(expansion: &WorldExpansionLayout) -> Self {
        let pad = expansion.overworld_castle_pad;
        let ground_y = expansion.castle_pad_surface_y;
        let base_y = ground_y + 1;
        let levels = FloorLevels {
            ground_y,
            base_y,
            wall_top_y: ground_y + WALL_HEIGHT,
            upper_floor_y: ground_y + UPPER_FLOOR_RISE,
            keep_roof_y: ground_y + KEEP_HEIGHT,
            tower_top_y: ground_y + TOWER_HEIGHT,
            max_y: ground_y + TOWER_HEIGHT + 1,
        };

        // The gate: the two rows the approach arrives at, in the west wall.
        let target_z = expansion.overworld_path_target.z;
        let gate = Doorway {
            base: IVec3::new(pad.min_x, base_y, target_z),
            width: GATE_WIDTH,
            facing: Orientation::West,
        };
        let gatehouse = ColumnRect::new(
            pad.min_x,
            target_z - 1,
            pad.min_x + GATEHOUSE_DEPTH - 1,
            target_z + GATE_WIDTH,
        );

        // Towers: one per corner.
        let t = TOWER_SIDE - 1;
        let towers = vec![
            ColumnRect::new(pad.min_x, pad.min_z, pad.min_x + t, pad.min_z + t),
            ColumnRect::new(pad.max_x - t, pad.min_z, pad.max_x, pad.min_z + t),
            ColumnRect::new(pad.min_x, pad.max_z - t, pad.min_x + t, pad.max_z),
            ColumnRect::new(pad.max_x - t, pad.max_z - t, pad.max_x, pad.max_z),
        ];

        // The keep: against the east wall, between the two east towers.
        let keep = ColumnRect::new(
            pad.max_x - KEEP_WIDTH + 1,
            pad.min_z + t,
            pad.max_x,
            pad.max_z - t,
        );
        let hall = ColumnRect::new(
            keep.min_x + 1,
            keep.min_z + 1,
            keep.max_x - 1,
            keep.max_z - 1,
        );
        let keep_door = Doorway {
            base: IVec3::new(keep.min_x, base_y, hall.min_z),
            width: 1,
            facing: Orientation::West,
        };

        // The stair: along the hall's east wall, climbing south from the
        // corridor row; the well above it stays open.
        let stair_x = hall.max_x;
        let stair = vec![
            (
                IVec3::new(stair_x, base_y, hall.min_z + 1),
                Orientation::North,
            ),
            (
                IVec3::new(stair_x, base_y + 1, hall.min_z + 2),
                Orientation::North,
            ),
        ];
        let stair_well = vec![(stair_x, hall.min_z + 1), (stair_x, hall.min_z + 2)];

        // Courtyard: every footprint column not taken by a building.
        let mut courtyard = Vec::new();
        for z in pad.min_z..=pad.max_z {
            for x in pad.min_x..=pad.max_x {
                let on_wall = x == pad.min_x || x == pad.max_x || z == pad.min_z || z == pad.max_z;
                let taken = on_wall
                    || keep.contains(x, z)
                    || gatehouse.contains(x, z)
                    || towers.iter().any(|r| r.contains(x, z));
                if !taken {
                    courtyard.push((x, z));
                }
            }
        }

        // Windows: narrow bays at eye level on both keep floors and high on
        // each tower's two outward faces.
        let mut windows = Vec::new();
        let eye_low = base_y + 1;
        let eye_high = levels.upper_floor_y + 2;
        // Keep west wall (courtyard side), away from the door row.
        windows.push(IVec3::new(keep.min_x, eye_low, hall.max_z - 1));
        windows.push(IVec3::new(keep.min_x, eye_high, hall.min_z + 1));
        windows.push(IVec3::new(keep.min_x, eye_high, hall.max_z - 1));
        // Keep north and south walls, between the towers.
        for x in [hall.min_x, hall.min_x + 1] {
            windows.push(IVec3::new(x, eye_low, keep.min_z));
            windows.push(IVec3::new(x, eye_low, keep.max_z));
            windows.push(IVec3::new(x, eye_high, keep.min_z));
            windows.push(IVec3::new(x, eye_high, keep.max_z));
        }
        // Keep east wall (the curtain wall), upper floor only.
        windows.push(IVec3::new(keep.max_x, eye_high, hall.min_z + 1));
        windows.push(IVec3::new(keep.max_x, eye_high, hall.max_z - 1));
        // Towers: the middle of each outward face.
        for r in &towers {
            let mid_x = (r.min_x + r.max_x) / 2;
            let mid_z = (r.min_z + r.max_z) / 2;
            let face_z = if r.min_z == pad.min_z {
                r.min_z
            } else {
                r.max_z
            };
            let face_x = if r.min_x == pad.min_x {
                r.min_x
            } else {
                r.max_x
            };
            windows.push(IVec3::new(mid_x, eye_high, face_z));
            windows.push(IVec3::new(face_x, eye_high, mid_z));
        }

        // The canonical route, as ground cells the eye stands two above.
        let mut interior_route = Vec::new();
        let route_z = target_z;
        let anchor = expansion.overworld_path_target;
        interior_route.push(IVec3::new(anchor.x - 2, ground_y, route_z));
        interior_route.push(IVec3::new(anchor.x - 1, ground_y, route_z));
        for x in gatehouse.min_x..=gatehouse.max_x {
            interior_route.push(IVec3::new(x, ground_y, route_z));
        }
        for x in (gatehouse.max_x + 1)..keep.min_x {
            interior_route.push(IVec3::new(x, ground_y, route_z));
        }
        interior_route.push(IVec3::new(keep.min_x - 1, ground_y, hall.min_z));
        interior_route.push(IVec3::new(keep.min_x, ground_y, hall.min_z));
        for x in hall.min_x..=hall.max_x {
            interior_route.push(IVec3::new(x, ground_y, hall.min_z));
        }
        for (cell, _) in &stair {
            interior_route.push(*cell);
        }
        interior_route.push(IVec3::new(stair_x, levels.upper_floor_y, hall.min_z + 3));
        interior_route.push(IVec3::new(hall.min_x, levels.upper_floor_y, hall.min_z + 3));

        Self {
            footprint: pad,
            levels,
            gate,
            gatehouse,
            keep,
            keep_door,
            hall,
            towers,
            courtyard,
            stair,
            stair_well,
            windows,
            interior_route,
        }
    }

    /// Whether a column is on the curtain wall (the footprint's outline).
    pub fn is_curtain_column(&self, x: i32, z: i32) -> bool {
        let f = self.footprint;
        f.contains(x, z) && (x == f.min_x || x == f.max_x || z == f.min_z || z == f.max_z)
    }

    /// The tower a column belongs to, if any.
    pub fn tower_at(&self, x: i32, z: i32) -> Option<&ColumnRect> {
        self.towers.iter().find(|r| r.contains(x, z))
    }

    /// Whether a column is on a tower's outline.
    pub fn is_tower_shell(&self, x: i32, z: i32) -> bool {
        self.tower_at(x, z)
            .is_some_and(|r| x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z)
    }

    /// Whether a column is on the keep's outline.
    pub fn is_keep_shell(&self, x: i32, z: i32) -> bool {
        let k = self.keep;
        k.contains(x, z) && (x == k.min_x || x == k.max_x || z == k.min_z || z == k.max_z)
    }

    /// Whether a column is one of the keep's four corners.
    pub fn is_keep_corner(&self, x: i32, z: i32) -> bool {
        let k = self.keep;
        (x == k.min_x || x == k.max_x) && (z == k.min_z || z == k.max_z)
    }

    /// The gatehouse passage: the columns between its two side walls.
    pub fn gatehouse_passage(&self) -> Vec<(i32, i32)> {
        let g = self.gatehouse;
        (g.min_x..=g.max_x)
            .flat_map(|x| ((g.min_z + 1)..g.max_z).map(move |z| (x, z)))
            .collect()
    }

    /// Checks the contract against the pad and the rest of the world.
    pub fn validate(&self, expansion: &WorldExpansionLayout) -> Result<(), String> {
        let pad = expansion.overworld_castle_pad;
        let inside = |r: &ColumnRect| {
            r.min_x >= pad.min_x
                && r.max_x <= pad.max_x
                && r.min_z >= pad.min_z
                && r.max_z <= pad.max_z
        };
        if !inside(&self.keep) || !inside(&self.gatehouse) || !self.towers.iter().all(inside) {
            return Err("a building leaves the pad".into());
        }
        if self.towers.len() != 4 {
            return Err(format!("{} towers", self.towers.len()));
        }
        for (i, a) in self.towers.iter().enumerate() {
            for b in &self.towers[i + 1..] {
                if a.max_x >= b.min_x
                    && b.max_x >= a.min_x
                    && a.max_z >= b.min_z
                    && b.max_z >= a.min_z
                {
                    return Err("towers overlap".into());
                }
            }
            if a.width() != TOWER_SIDE || a.depth() != TOWER_SIDE {
                return Err("tower size".into());
            }
        }
        if self.gate.base.x != pad.min_x
            || !(self.gate.base.z..self.gate.base.z + self.gate.width)
                .contains(&expansion.overworld_path_target.z)
        {
            return Err("the gate does not face the approach".into());
        }
        if self.hall.width() < 3 || self.hall.depth() < 3 {
            return Err("the hall is too small".into());
        }
        if self.courtyard.len() < 8 {
            return Err("the courtyard is too small".into());
        }
        let house = HOUSE_FOOTPRINT.grown(2);
        if house.max_x >= pad.min_x
            && house.min_x <= pad.max_x
            && house.max_z >= pad.min_z
            && house.min_z <= pad.max_z
        {
            return Err("the castle overlaps the house".into());
        }
        if self.levels.max_y > 12 {
            return Err("the castle rises above the world box".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Foundation, courtyard and curtain wall (C186)
// ---------------------------------------------------------------------

const MASONRY_SALT: u32 = 0x16C0_0001;
/// Share of curtain-wall cells laid in cobblestone instead of stone.
pub const COBBLE_SHARE: f32 = 0.35;

fn up(block_type: BlockType, material: MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Up)
}

/// Stone or cobblestone for a masonry cell, by hash, so the walls read as
/// rubble-and-ashlar rather than a flat colour.
pub fn masonry(seed: u32, cell: IVec3) -> BlockInstance {
    if expansion_hash(seed, cell.x * 3 + cell.y, cell.z * 5 - cell.y, MASONRY_SALT) < COBBLE_SHARE {
        up(BlockType::Cobblestone, cobblestone_material_id())
    } else {
        up(BlockType::Stone, stone_block_material_id())
    }
}

/// The castle's base, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleFoundation {
    /// Ground cells under every wall, tower and keep outline (deepslate
    /// bricks), replacing the pad's grass.
    pub footings: Vec<IVec3>,
    /// Ground cells of the courtyard, gatehouse passage, hall and tower
    /// cores (cobblestone and stone), replacing the pad's grass.
    pub floors: Vec<IVec3>,
    /// Curtain-wall cells (stone and cobblestone), hollow behind.
    pub walls: Vec<IVec3>,
    /// The cells of the gate opening, left as air.
    pub gate_opening: Vec<IVec3>,
}

impl CastleFoundation {
    pub fn cells(&self) -> Vec<IVec3> {
        self.footings
            .iter()
            .chain(self.floors.iter())
            .chain(self.walls.iter())
            .copied()
            .collect()
    }
}

/// Lays the foundation and raises the curtain wall.
///
/// The pad's grass row becomes the floor plan: deepslate-brick footings
/// under every outline (curtain, towers, keep), cobblestone across the
/// courtyard and the gatehouse passage, stone inside the keep and the
/// tower cores. The curtain wall rises `WALL_HEIGHT` courses on the
/// footprint's outline, one cell thick with nothing behind it, and the
/// gate opening stays open for the door.
pub fn build_castle_foundation(
    terrain: &TerrainConfig,
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleFoundation {
    let mut built = CastleFoundation::default();
    let f = layout.footprint;
    let g = layout.levels.ground_y;
    let passage = layout.gatehouse_passage();
    for z in f.min_z..=f.max_z {
        for x in f.min_x..=f.max_x {
            let cell = IVec3::new(x, g, z);
            // The gate threshold is paved: the passage runs over it.
            let outline = !passage.contains(&(x, z))
                && (layout.is_curtain_column(x, z)
                    || layout.is_tower_shell(x, z)
                    || layout.is_keep_shell(x, z)
                    || layout.gatehouse.contains(x, z));
            if outline {
                world.insert(
                    cell,
                    up(BlockType::DeepslateBricks, deepslate_bricks_material_id()),
                );
                built.footings.push(cell);
            } else if layout.hall.contains(x, z) || layout.tower_at(x, z).is_some() {
                world.insert(cell, up(BlockType::Stone, stone_block_material_id()));
                built.floors.push(cell);
            } else {
                world.insert(cell, up(BlockType::Cobblestone, cobblestone_material_id()));
                built.floors.push(cell);
            }
        }
    }
    let gate_cells = layout.gate.cells();
    for z in f.min_z..=f.max_z {
        for x in f.min_x..=f.max_x {
            if !layout.is_curtain_column(x, z) {
                continue;
            }
            for y in layout.levels.base_y..=layout.levels.wall_top_y {
                let cell = IVec3::new(x, y, z);
                if gate_cells.contains(&cell) {
                    world.remove(cell);
                    built.gate_opening.push(cell);
                    continue;
                }
                world.insert(cell, masonry(terrain.seed, cell));
                built.walls.push(cell);
            }
        }
    }
    built
}

// ---------------------------------------------------------------------
// Towers and battlements (C187)
// ---------------------------------------------------------------------

/// Merlons stand on every other wall cell.
pub const MERLON_PERIOD: i32 = 2;

/// The towers and crenellations, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleTowers {
    /// Shell cells of the four towers (deepslate-brick base course, then
    /// stone and cobblestone), hollow inside.
    pub shells: Vec<IVec3>,
    /// The open shaft of each tower: `(x, z)` of its core column.
    pub cores: Vec<(i32, i32)>,
    /// Merlons on the tower tops (the corners of each tower).
    pub tower_merlons: Vec<IVec3>,
    /// Merlons along the curtain wall between the towers.
    pub wall_merlons: Vec<IVec3>,
}

impl CastleTowers {
    pub fn cells(&self) -> Vec<IVec3> {
        self.shells
            .iter()
            .chain(self.tower_merlons.iter())
            .chain(self.wall_merlons.iter())
            .copied()
            .collect()
    }
}

/// Raises the four corner towers and the battlements.
///
/// Each tower is a one-cell-thick shell around an open core from the
/// ground to `tower_top_y`, with a deepslate-brick base course and stone
/// or cobblestone above; four merlons stand on its corners one course
/// higher. The curtain wall between the towers carries a merlon on every
/// other cell. Nothing is added inside the courtyard, the gate or the
/// keep's footprint.
pub fn build_castle_towers(
    terrain: &TerrainConfig,
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleTowers {
    let mut built = CastleTowers::default();
    let lv = layout.levels;
    for r in &layout.towers {
        for z in r.min_z..=r.max_z {
            for x in r.min_x..=r.max_x {
                let shell = x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z;
                if !shell {
                    built.cores.push((x, z));
                    for y in lv.base_y..=lv.max_y {
                        world.remove(IVec3::new(x, y, z));
                    }
                    continue;
                }
                for y in lv.base_y..=lv.tower_top_y {
                    let cell = IVec3::new(x, y, z);
                    let block = if y == lv.base_y {
                        up(BlockType::DeepslateBricks, deepslate_bricks_material_id())
                    } else {
                        masonry(terrain.seed, cell)
                    };
                    world.insert(cell, block);
                    built.shells.push(cell);
                }
                let corner = (x == r.min_x || x == r.max_x) && (z == r.min_z || z == r.max_z);
                if corner {
                    let cell = IVec3::new(x, lv.tower_top_y + 1, z);
                    world.insert(cell, masonry(terrain.seed, cell));
                    built.tower_merlons.push(cell);
                }
            }
        }
    }
    // Curtain merlons between the towers, skipping the keep and gatehouse
    // (they carry their own tops).
    let f = layout.footprint;
    for z in f.min_z..=f.max_z {
        for x in f.min_x..=f.max_x {
            if !layout.is_curtain_column(x, z)
                || layout.tower_at(x, z).is_some()
                || layout.keep.contains(x, z)
                || layout.gatehouse.contains(x, z)
            {
                continue;
            }
            let along = if z == f.min_z || z == f.max_z { x } else { z };
            if along % MERLON_PERIOD != f.min_x % MERLON_PERIOD {
                continue;
            }
            let cell = IVec3::new(x, lv.wall_top_y + 1, z);
            world.insert(cell, masonry(terrain.seed, cell));
            built.wall_merlons.push(cell);
        }
    }
    built
}
