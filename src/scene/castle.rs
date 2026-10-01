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
use crate::scene::material_gallery::{
    deepslate_bricks_material_id, glass_material_id, redstone_lamp_material_id,
};
use crate::scene::orientation::Orientation;
use crate::scene::overworld::{ColumnRect, HOUSE_FOOTPRINT};
use crate::scene::overworld_blocks::{
    cobblestone_material_id, double_wood_slab_material_id, fence_material_id, log_material_id,
    stone_block_material_id, wood_door_bottom_material_id, wood_door_top_material_id,
    wood_planks_material_id, wood_stairs_material_id,
};
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
        // Keep west wall (courtyard side): one low bay away from the door
        // row, two high bays a wall cell apart.
        windows.push(IVec3::new(keep.min_x, eye_low, hall.max_z - 1));
        windows.push(IVec3::new(keep.min_x, eye_high, hall.min_z + 1));
        windows.push(IVec3::new(keep.min_x, eye_high, hall.max_z));
        // Keep north and south walls, the free bay between the towers.
        windows.push(IVec3::new(hall.min_x, eye_low, keep.min_z));
        windows.push(IVec3::new(hall.min_x, eye_low, keep.max_z));
        windows.push(IVec3::new(hall.min_x, eye_high, keep.min_z));
        windows.push(IVec3::new(hall.min_x, eye_high, keep.max_z));
        // Keep east wall (the curtain wall), upper floor only.
        windows.push(IVec3::new(keep.max_x, eye_high, hall.min_z + 1));
        windows.push(IVec3::new(keep.max_x, eye_high, hall.max_z));
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

// ---------------------------------------------------------------------
// Gatehouse (C188)
// ---------------------------------------------------------------------

/// The gatehouse, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleGatehouse {
    /// Log posts at the inner corners of the passage.
    pub posts: Vec<IVec3>,
    /// Masonry of the side walls inside the curtain.
    pub walls: Vec<IVec3>,
    /// Plank roof over the passage.
    pub roof: Vec<IVec3>,
    /// The door cells (`WoodDoor`, camera-passable), lower then upper.
    pub door: Vec<IVec3>,
    /// Fence rail on the roof's outer edge.
    pub rail: Vec<IVec3>,
}

impl CastleGatehouse {
    pub fn cells(&self) -> Vec<IVec3> {
        self.posts
            .iter()
            .chain(self.walls.iter())
            .chain(self.roof.iter())
            .chain(self.door.iter())
            .chain(self.rail.iter())
            .copied()
            .collect()
    }
}

/// Builds the gatehouse around the gate opening.
///
/// The passage runs east from the curtain through the gatehouse block;
/// its two side rows are masonry on the curtain and log posts inside,
/// `WALL_HEIGHT` courses high, roofed with planks one course above the
/// curtain top and railed with fence on the outer edge. The gate opening
/// receives the `WoodDoor`, two wide and two high, facing west: visible,
/// closed, and passable to the camera as everywhere else in the world.
pub fn build_castle_gatehouse(
    terrain: &TerrainConfig,
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleGatehouse {
    let mut built = CastleGatehouse::default();
    let g = layout.gatehouse;
    let lv = layout.levels;
    for z in [g.min_z, g.max_z] {
        for x in g.min_x..=g.max_x {
            for y in lv.base_y..=lv.wall_top_y {
                let cell = IVec3::new(x, y, z);
                if layout.is_curtain_column(x, z) {
                    world.insert(cell, masonry(terrain.seed, cell));
                    built.walls.push(cell);
                } else {
                    world.insert(cell, up(BlockType::Log, log_material_id()));
                    built.posts.push(cell);
                }
            }
        }
    }
    let roof_y = lv.wall_top_y + 1;
    for z in g.min_z..=g.max_z {
        for x in g.min_x..=g.max_x {
            let cell = IVec3::new(x, roof_y, z);
            world.insert(cell, up(BlockType::WoodPlanks, wood_planks_material_id()));
            built.roof.push(cell);
        }
    }
    for z in [g.min_z, g.max_z] {
        let cell = IVec3::new(g.min_x, roof_y + 1, z);
        world.insert(
            cell,
            BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::West),
        );
        built.rail.push(cell);
    }
    let door = layout.gate;
    for dz in 0..door.width {
        for (dy, material) in [
            (0, wood_door_bottom_material_id()),
            (1, wood_door_top_material_id()),
        ] {
            let cell = IVec3::new(door.base.x, door.base.y + dy, door.base.z + dz);
            world.insert(
                cell,
                BlockInstance::new(BlockType::WoodDoor, material, door.facing),
            );
            built.door.push(cell);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Keep (C189)
// ---------------------------------------------------------------------

/// The keep, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleKeep {
    /// Masonry of the shell (the outline, ground to roof).
    pub shell: Vec<IVec3>,
    /// Log corner posts (the corners not shared with a tower).
    pub corners: Vec<IVec3>,
    /// The keep's door cells (`WoodDoor`, passable), lower then upper.
    pub door: Vec<IVec3>,
    /// Ground-floor planks (replacing the stone floor of the hall).
    pub ground_floor: Vec<IVec3>,
    /// Upper-floor planks, the stair well left open.
    pub upper_floor: Vec<IVec3>,
    /// Log beams carrying the upper floor (its two edge rows).
    pub beams: Vec<IVec3>,
    /// The roof (double slabs) closing the keep.
    pub roof: Vec<IVec3>,
}

impl CastleKeep {
    pub fn cells(&self) -> Vec<IVec3> {
        self.shell
            .iter()
            .chain(self.corners.iter())
            .chain(self.door.iter())
            .chain(self.ground_floor.iter())
            .chain(self.upper_floor.iter())
            .chain(self.beams.iter())
            .chain(self.roof.iter())
            .copied()
            .collect()
    }

    /// The upper floor's walkable cells (planks and beams).
    pub fn upper_deck(&self) -> Vec<IVec3> {
        self.upper_floor
            .iter()
            .chain(self.beams.iter())
            .copied()
            .collect()
    }
}

/// Builds the keep: a hollow masonry shell from the ground to the roof
/// with log posts on its free corners, a west door onto the courtyard,
/// a plank ground floor, a timber upper floor (log beams on the edge rows,
/// planks between, the stair well open) and a double-slab roof. Nothing
/// fills either room.
pub fn build_castle_keep(
    terrain: &TerrainConfig,
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleKeep {
    let mut built = CastleKeep::default();
    let k = layout.keep;
    let lv = layout.levels;
    let door_cells = layout.keep_door.cells();
    for z in k.min_z..=k.max_z {
        for x in k.min_x..=k.max_x {
            // Tower cells already stand (with their own base course).
            if !layout.is_keep_shell(x, z) || layout.tower_at(x, z).is_some() {
                continue;
            }
            let log_corner = layout.is_keep_corner(x, z) && layout.tower_at(x, z).is_none();
            for y in lv.base_y..=lv.keep_roof_y {
                let cell = IVec3::new(x, y, z);
                if door_cells.contains(&cell) {
                    continue;
                }
                if log_corner {
                    world.insert(cell, up(BlockType::Log, log_material_id()));
                    built.corners.push(cell);
                } else {
                    world.insert(cell, masonry(terrain.seed, cell));
                    built.shell.push(cell);
                }
            }
        }
    }
    let door = layout.keep_door;
    for (dy, material) in [
        (0, wood_door_bottom_material_id()),
        (1, wood_door_top_material_id()),
    ] {
        let cell = IVec3::new(door.base.x, door.base.y + dy, door.base.z);
        world.insert(
            cell,
            BlockInstance::new(BlockType::WoodDoor, material, door.facing),
        );
        built.door.push(cell);
    }
    let h = layout.hall;
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            let floor = IVec3::new(x, lv.ground_y, z);
            world.insert(floor, up(BlockType::WoodPlanks, wood_planks_material_id()));
            built.ground_floor.push(floor);
            for y in lv.base_y..lv.upper_floor_y {
                world.remove(IVec3::new(x, y, z));
            }
            if !layout.stair_well.contains(&(x, z)) {
                let cell = IVec3::new(x, lv.upper_floor_y, z);
                if z == h.min_z || z == h.max_z {
                    world.insert(cell, up(BlockType::Log, log_material_id()));
                    built.beams.push(cell);
                } else {
                    world.insert(cell, up(BlockType::WoodPlanks, wood_planks_material_id()));
                    built.upper_floor.push(cell);
                }
            }
            for y in (lv.upper_floor_y + 1)..lv.keep_roof_y {
                world.remove(IVec3::new(x, y, z));
            }
            let roof = IVec3::new(x, lv.keep_roof_y, z);
            world.insert(
                roof,
                up(BlockType::DoubleWoodSlab, double_wood_slab_material_id()),
            );
            built.roof.push(roof);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Windows (C190)
// ---------------------------------------------------------------------

/// The castle's glazing, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleWindows {
    /// Glass cells set into the keep and tower shells.
    pub glass: Vec<IVec3>,
}

/// Sets glass into the window bays of the layout: each listed shell cell
/// becomes `Glass` (visible, refractive, camera-solid) provided it is a
/// masonry cell with masonry above and below, so no opening weakens a
/// corner post, a door or a wall top.
pub fn glaze_castle_windows(
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleWindows {
    let mut built = CastleWindows::default();
    let is_masonry = |world: &VoxelWorld, cell: IVec3| {
        world.get(cell).is_some_and(|b| {
            matches!(
                b.block_type(),
                BlockType::Stone | BlockType::Cobblestone | BlockType::DeepslateBricks
            )
        })
    };
    for cell in &layout.windows {
        let above = IVec3::new(cell.x, cell.y + 1, cell.z);
        let below = IVec3::new(cell.x, cell.y - 1, cell.z);
        if !is_masonry(world, *cell) || !world.contains(above) || !is_masonry(world, below) {
            continue;
        }
        world.insert(*cell, up(BlockType::Glass, glass_material_id()));
        built.glass.push(*cell);
    }
    built
}

// ---------------------------------------------------------------------
// Interior stair (C191)
// ---------------------------------------------------------------------

/// The interior stair, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleStairs {
    /// The treads (`WoodStairs`), lowest first, with their orientation.
    pub treads: Vec<Tread>,
    /// Fence rail on the upper floor along the open well.
    pub railing: Vec<IVec3>,
    /// The upper-floor cell the stair lands on.
    pub landing: IVec3,
}

impl CastleStairs {
    pub fn cells(&self) -> Vec<IVec3> {
        self.treads
            .iter()
            .map(|(c, _)| *c)
            .chain(self.railing.iter().copied())
            .collect()
    }
}

/// Sets the interior stair of the keep: the layout's treads as `North`
/// wood stairs (raised half to the south, so the climb runs south from the
/// corridor row), the well above them left open, and a fence rail on the
/// upper floor along the well's inner edge. The landing is the upper-floor
/// cell past the last tread.
pub fn build_castle_stairs(layout: &OverworldCastleLayout, world: &mut VoxelWorld) -> CastleStairs {
    let mut built = CastleStairs::default();
    for (cell, orientation) in &layout.stair {
        world.insert(
            *cell,
            BlockInstance::new(
                BlockType::WoodStairs,
                wood_stairs_material_id(),
                *orientation,
            ),
        );
        for dy in 1..=2 {
            world.remove(IVec3::new(cell.x, cell.y + dy, cell.z));
        }
        built.treads.push((*cell, *orientation));
    }
    let last = layout
        .stair
        .last()
        .map(|(c, _)| *c)
        .unwrap_or(layout.keep_door.base);
    built.landing = IVec3::new(last.x, layout.levels.upper_floor_y, last.z + 1);
    for (x, z) in &layout.stair_well {
        let cell = IVec3::new(x - 1, layout.levels.upper_floor_y + 1, *z);
        if layout.hall.contains(cell.x, cell.z)
            && world.contains(IVec3::new(cell.x, cell.y - 1, cell.z))
            && !world.contains(cell)
        {
            world.insert(
                cell,
                BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::East),
            );
            built.railing.push(cell);
        }
    }
    built
}

// ---------------------------------------------------------------------
// Furniture (C192)
// ---------------------------------------------------------------------

/// A seat: a `WoodStairs` block whose raised half is the backrest, so its
/// open side faces the table.
pub type Seat = (IVec3, Orientation);

/// The great hall's furniture, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleFurniture {
    /// Chairs (`WoodStairs`), ground floor and upper room.
    pub chairs: Vec<Seat>,
    /// Table tops (`DoubleWoodSlab`).
    pub tables: Vec<IVec3>,
    /// Benches (`DoubleWoodSlab`) against the wall.
    pub benches: Vec<IVec3>,
}

impl CastleFurniture {
    pub fn cells(&self) -> Vec<IVec3> {
        self.chairs
            .iter()
            .map(|(c, _)| *c)
            .chain(self.tables.iter().copied())
            .chain(self.benches.iter().copied())
            .collect()
    }
}

/// The direction a seat's open side faces (the raised half is opposite):
/// canonical `South` stairs are raised to the north, so they face south.
pub fn seat_faces(orientation: Orientation) -> (i32, i32) {
    match orientation {
        Orientation::South | Orientation::Up | Orientation::Down => (0, 1),
        Orientation::North => (0, -1),
        Orientation::East => (1, 0),
        Orientation::West => (-1, 0),
    }
}

/// Furnishes the keep as a great hall: a two-cell slab table in the middle
/// of the ground floor with five stair chairs around it (west, east and
/// south), a bench against the west wall, and upstairs a one-cell table
/// with a chair. The corridor row inside the door, the stair, its landing
/// and every window stay clear.
pub fn furnish_castle_hall(
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleFurniture {
    let mut built = CastleFurniture::default();
    let h = layout.hall;
    let lv = layout.levels;
    let chair =
        |world: &mut VoxelWorld, built: &mut CastleFurniture, cell: IVec3, o: Orientation| {
            if world.contains(cell) {
                return;
            }
            world.insert(
                cell,
                BlockInstance::new(BlockType::WoodStairs, wood_stairs_material_id(), o),
            );
            built.chairs.push((cell, o));
        };
    let slab = |world: &mut VoxelWorld, cell: IVec3| -> bool {
        if world.contains(cell) {
            return false;
        }
        world.insert(
            cell,
            up(BlockType::DoubleWoodSlab, double_wood_slab_material_id()),
        );
        true
    };
    // Ground floor: table along x = min + 1, rows min + 1 and min + 2.
    let tx = h.min_x + 1;
    let y = lv.base_y;
    for z in [h.min_z + 1, h.min_z + 2] {
        let cell = IVec3::new(tx, y, z);
        if slab(world, cell) {
            built.tables.push(cell);
        }
        chair(
            world,
            &mut built,
            IVec3::new(tx - 1, y, z),
            Orientation::East,
        );
        chair(
            world,
            &mut built,
            IVec3::new(tx + 1, y, z),
            Orientation::West,
        );
    }
    chair(
        world,
        &mut built,
        IVec3::new(tx, y, h.min_z + 3),
        Orientation::North,
    );
    let bench = IVec3::new(h.min_x, y, h.max_z);
    if slab(world, bench) {
        built.benches.push(bench);
    }
    // Upper room: a small table with one chair, off the landing row.
    let uy = lv.upper_floor_y + 1;
    let table = IVec3::new(tx, uy, h.min_z + 2);
    if world.contains(IVec3::new(table.x, table.y - 1, table.z)) && slab(world, table) {
        built.tables.push(table);
        chair(
            world,
            &mut built,
            IVec3::new(tx - 1, uy, h.min_z + 2),
            Orientation::East,
        );
    }
    built
}

// ---------------------------------------------------------------------
// Details (C193)
// ---------------------------------------------------------------------

/// The finishing details, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastleDetails {
    /// Lit redstone lamps: sconces set into walls and lamps on posts.
    pub lamps: Vec<IVec3>,
    /// Merlons along the keep's roof line (every other cell).
    pub keep_merlons: Vec<IVec3>,
    /// Fence posts and rails in the courtyard alleys.
    pub rails: Vec<IVec3>,
}

impl CastleDetails {
    pub fn cells(&self) -> Vec<IVec3> {
        self.lamps
            .iter()
            .chain(self.keep_merlons.iter())
            .chain(self.rails.iter())
            .copied()
            .collect()
    }
}

/// Finishes the castle: lamp sconces replace one masonry cell on each side
/// above the gate, two on the hall's east wall at eye level and one in the
/// upper room's west wall; the north and south alleys beside the keep are
/// lined with fence rails ending in a lamp post each; and merlons give the
/// keep roof the curtain wall's rhythm. Lamps are emissive blocks
/// only: no point light is added.
pub fn detail_castle(
    terrain: &TerrainConfig,
    layout: &OverworldCastleLayout,
    world: &mut VoxelWorld,
) -> CastleDetails {
    let mut built = CastleDetails::default();
    let lv = layout.levels;
    let lamp = || up(BlockType::RedstoneLampLit, redstone_lamp_material_id());
    let masonry_at = |world: &VoxelWorld, cell: IVec3| {
        world.get(cell).is_some_and(|b| {
            matches!(
                b.block_type(),
                BlockType::Stone | BlockType::Cobblestone | BlockType::DeepslateBricks
            )
        })
    };
    // Sconces: above the gate, in the hall's east wall, in the upper room.
    let g = layout.gatehouse;
    let mut sconces = vec![
        IVec3::new(g.min_x, lv.wall_top_y, g.min_z),
        IVec3::new(g.min_x, lv.wall_top_y, g.max_z),
        IVec3::new(layout.keep.max_x, lv.base_y + 1, layout.hall.min_z + 1),
        IVec3::new(layout.keep.max_x, lv.base_y + 1, layout.hall.max_z),
        IVec3::new(
            layout.keep.min_x,
            lv.upper_floor_y + 2,
            layout.hall.min_z + 2,
        ),
    ];
    sconces.retain(|c| masonry_at(world, *c) && !layout.windows.contains(c));
    for cell in sconces {
        world.insert(cell, lamp());
        built.lamps.push(cell);
    }
    // Lamp posts at the far end of the north and south alleys (the rails
    // below line the alley up to them, so no walkable pocket is cut off).
    let post_x = layout.keep.min_x + 2;
    for z in [layout.footprint.min_z + 1, layout.footprint.max_z - 1] {
        if !layout.courtyard.contains(&(post_x, z)) {
            continue;
        }
        let post = IVec3::new(post_x, lv.base_y, z);
        let top = IVec3::new(post_x, lv.base_y + 1, z);
        if world.contains(post) || world.contains(top) {
            continue;
        }
        world.insert(
            post,
            BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::Up),
        );
        world.insert(top, lamp());
        built.rails.push(post);
        built.lamps.push(top);
    }
    // Keep merlons: the roof line, every other cell, free corners and
    // tower cells excepted.
    let k = layout.keep;
    for z in k.min_z..=k.max_z {
        for x in k.min_x..=k.max_x {
            if !layout.is_keep_shell(x, z) || layout.tower_at(x, z).is_some() {
                continue;
            }
            let along = if z == k.min_z || z == k.max_z { x } else { z };
            if along % MERLON_PERIOD != k.min_x % MERLON_PERIOD {
                continue;
            }
            let cell = IVec3::new(x, lv.keep_roof_y + 1, z);
            if world.contains(cell) || !world.contains(IVec3::new(x, lv.keep_roof_y, z)) {
                continue;
            }
            world.insert(cell, masonry(terrain.seed, cell));
            built.keep_merlons.push(cell);
        }
    }
    // Alley rails: along the keep's north and south faces, one cell out,
    // from the alley mouth to the lamp post.
    for z in [k.min_z - 1, k.max_z + 1] {
        for x in k.min_x..(k.min_x + 2) {
            if !layout.courtyard.contains(&(x, z)) {
                continue;
            }
            let cell = IVec3::new(x, lv.base_y, z);
            if world.contains(cell) {
                continue;
            }
            world.insert(
                cell,
                BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::Up),
            );
            built.rails.push(cell);
        }
    }
    built
}
