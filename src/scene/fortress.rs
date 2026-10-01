//! Gate 17: the inverted Red-Black fortress on the Gate 15 fortress pad.
//!
//! Functional up is `-Y`: the pad row is the ground, courses are counted
//! downward (more negative `y` is higher for the Red-Black viewer), floors
//! are the lowest cell of a column and every block is `Orientation::Down`.
//! The layout (C195) is a contract derived from `WorldExpansionLayout`;
//! the builders that follow place blocks stage by stage.

#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::expansion::{WorldExpansionLayout, expansion_hash, lower_bottom_y};
use crate::scene::orientation::Orientation;
use crate::scene::overworld::ColumnRect;
use crate::scene::overworld_blocks::{
    deepslate_material_id, polished_blackstone_bricks_material_id, smooth_basalt_material_id,
};
use crate::scene::red_black_maze::Family;
use crate::scene::terrain::TerrainConfig;
use crate::scene::voxel_world::VoxelWorld;

/// Side of each square tower (columns).
pub const FORTRESS_TOWER_SIDE: i32 = 3;
/// Courses of the curtain wall (counted downward from the ground).
pub const FORTRESS_WALL_COURSES: i32 = 3;
/// Courses of the keep shell (two floors and a roof).
pub const FORTRESS_KEEP_COURSES: i32 = 6;
/// Courses of each tower shell.
pub const FORTRESS_TOWER_COURSES: i32 = 8;
/// Courses from the hall floor to the upper floor.
pub const FORTRESS_UPPER_FLOOR_RISE: i32 = 3;
/// Width of the keep (columns along x, outer walls included).
pub const FORTRESS_KEEP_WIDTH: i32 = 6;
/// Columns the gate corridor reaches into the courtyard.
pub const FORTRESS_GATE_DEPTH: i32 = 2;

/// The vertical contract of the fortress, in world `y` (functional up is
/// `-Y`, so "higher" means more negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FortressLevels {
    /// The pad row: floors replace this row; its -Y face is the ground.
    pub ground_y: i32,
    /// First course above the ground (one below in world y).
    pub base_y: i32,
    /// Last course of the curtain wall; finials stand one further.
    pub wall_top_y: i32,
    /// The keep's upper floor.
    pub upper_floor_y: i32,
    /// The keep's roof; finials one further.
    pub keep_roof_y: i32,
    /// Last course of the towers; finials one further.
    pub tower_top_y: i32,
    /// The functionally highest cell of the fortress (tower finials).
    pub max_y: i32,
}

/// A tower of the fortress and the family it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FortressTower {
    pub family: Family,
    pub bounds: ColumnRect,
    /// The opening from the courtyard into the core (lower cell, two high).
    pub entrance: Doorway,
}

impl FortressTower {
    /// The open core column.
    pub fn core(&self) -> (i32, i32) {
        (
            (self.bounds.min_x + self.bounds.max_x) / 2,
            (self.bounds.min_z + self.bounds.max_z) / 2,
        )
    }

    /// Whether a column is on the tower's outline.
    pub fn is_shell(&self, x: i32, z: i32) -> bool {
        let r = self.bounds;
        r.contains(x, z) && (x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z)
    }

    /// Whether a column is one of the tower's corners.
    pub fn is_corner(&self, x: i32, z: i32) -> bool {
        let r = self.bounds;
        (x == r.min_x || x == r.max_x) && (z == r.min_z || z == r.max_z)
    }
}

/// A doorway of the inverted world: its cells run functionally upward,
/// that is toward more negative `y`, from the base cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Doorway {
    /// The cell next to the ground (the ground is one above in world y).
    pub base: IVec3,
    /// Columns (rows along z) the opening spans.
    pub width: i32,
    /// Which way the opening faces (out of the building).
    pub facing: Orientation,
}

impl Doorway {
    /// Every cell of the opening (two high).
    pub fn cells(&self) -> Vec<IVec3> {
        (0..self.width)
            .flat_map(|dz| {
                (0..2).map(move |dy| IVec3::new(self.base.x, self.base.y - dy, self.base.z + dz))
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

/// A functional "up" step in the inverted world: one cell more negative.
pub fn above(cell: IVec3, courses: i32) -> IVec3 {
    IVec3::new(cell.x, cell.y - courses, cell.z)
}

/// Where the fortress goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedBlackFortressLayout {
    /// The fortress pad: nothing of the fortress stands outside it.
    pub footprint: ColumnRect,
    pub levels: FortressLevels,
    /// The gate in the west wall, facing the Gate 15 lower approach.
    pub gate: Doorway,
    /// The gate block (arch pillars, corridor, lintel).
    pub gatehouse: ColumnRect,
    /// The central keep (outer walls included); its east wall is the curtain.
    pub central_keep: ColumnRect,
    /// The keep's door from the courtyard (on the south row of the hall).
    pub keep_door: Doorway,
    /// The keep's interior columns.
    pub hall: ColumnRect,
    pub crimson_tower: FortressTower,
    pub orange_tower: FortressTower,
    pub violet_tower: FortressTower,
    /// Open columns inside the walls.
    pub courtyard: Vec<(i32, i32)>,
    /// Treads of the keep stair (`Down` stairs climb north), lowest first.
    pub stair: Vec<(IVec3, Orientation)>,
    /// Columns of the upper floor left open above the stair.
    pub stair_well: Vec<(i32, i32)>,
    /// Ground cells of the routes the viewer stands under: the main route
    /// (approach, gate, courtyard, keep door, hall, stair, upper room) and
    /// one route per tower (courtyard to the tower core).
    pub interior_routes: Vec<Vec<IVec3>>,
}

impl RedBlackFortressLayout {
    /// Derives the fortress from the Gate 15 pad and lower approach.
    pub fn derive(expansion: &WorldExpansionLayout) -> Self {
        let pad = expansion.red_black_fortress_pad;
        let ground_y = expansion.fortress_pad_bottom_y;
        let base_y = ground_y - 1;
        let levels = FortressLevels {
            ground_y,
            base_y,
            wall_top_y: ground_y - FORTRESS_WALL_COURSES,
            upper_floor_y: ground_y - FORTRESS_UPPER_FLOOR_RISE,
            keep_roof_y: ground_y - FORTRESS_KEEP_COURSES,
            tower_top_y: ground_y - FORTRESS_TOWER_COURSES,
            max_y: ground_y - FORTRESS_TOWER_COURSES - 1,
        };
        let target = expansion.red_black_path_target;
        let gate = Doorway {
            base: IVec3::new(pad.min_x, base_y, target.z),
            width: 2,
            facing: Orientation::West,
        };
        let gatehouse = ColumnRect::new(
            pad.min_x,
            target.z - 1,
            pad.min_x + FORTRESS_GATE_DEPTH - 1,
            target.z + gate.width,
        );
        let t = FORTRESS_TOWER_SIDE - 1;
        let tower = |family, min_x, min_z, entrance: IVec3| FortressTower {
            family,
            bounds: ColumnRect::new(min_x, min_z, min_x + t, min_z + t),
            entrance: Doorway {
                base: IVec3::new(entrance.x, base_y, entrance.z),
                width: 1,
                facing: if entrance.x == min_x + t {
                    Orientation::East
                } else {
                    Orientation::West
                },
            },
        };
        // Crimson north-west, Orange north-east, Violet south-west; the
        // south-east corner belongs to the keep's curtain.
        let crimson_tower = tower(
            Family::Crimson,
            pad.min_x,
            pad.min_z,
            IVec3::new(pad.min_x + t, 0, pad.min_z + 1),
        );
        let orange_tower = tower(
            Family::Orange,
            pad.max_x - t,
            pad.min_z,
            IVec3::new(pad.max_x - t, 0, pad.min_z + 1),
        );
        let violet_tower = tower(
            Family::Violet,
            pad.min_x,
            pad.max_z - t,
            IVec3::new(pad.min_x + t, 0, pad.max_z - 1),
        );
        let central_keep = ColumnRect::new(
            pad.max_x - FORTRESS_KEEP_WIDTH + 1,
            pad.min_z + t,
            pad.max_x,
            pad.max_z - t,
        );
        let hall = ColumnRect::new(
            central_keep.min_x + 1,
            central_keep.min_z + 1,
            central_keep.max_x - 1,
            central_keep.max_z - 1,
        );
        // The door opens on the hall's south row; `Down` stairs climb north
        // from that row along the east wall.
        let keep_door = Doorway {
            base: IVec3::new(central_keep.min_x, base_y, hall.max_z),
            width: 1,
            facing: Orientation::West,
        };
        let stair_x = hall.max_x;
        let stair = vec![
            (
                IVec3::new(stair_x, base_y, hall.max_z - 1),
                Orientation::Down,
            ),
            (
                IVec3::new(stair_x, base_y - 1, hall.max_z - 2),
                Orientation::Down,
            ),
        ];
        let stair_well = vec![(stair_x, hall.max_z - 1), (stair_x, hall.max_z - 2)];

        let towers = [crimson_tower, orange_tower, violet_tower];
        let mut courtyard = Vec::new();
        for z in pad.min_z..=pad.max_z {
            for x in pad.min_x..=pad.max_x {
                let on_wall = x == pad.min_x || x == pad.max_x || z == pad.min_z || z == pad.max_z;
                let taken = on_wall
                    || central_keep.contains(x, z)
                    || gatehouse.contains(x, z)
                    || towers.iter().any(|t| t.bounds.contains(x, z));
                if !taken {
                    courtyard.push((x, z));
                }
            }
        }

        // Main route: forecourt, gate corridor, courtyard, keep door, hall
        // (south row), stair, landing, upper room.
        // The two forecourt cells stand on the Gate 15 terrace, whose
        // ground steps with x.
        let mut main = Vec::new();
        for x in [target.x - 2, target.x - 1] {
            main.push(IVec3::new(x, lower_bottom_y(expansion, x), target.z));
        }
        for x in gatehouse.min_x..=gatehouse.max_x {
            main.push(IVec3::new(x, ground_y, target.z));
        }
        for x in (gatehouse.max_x + 1)..central_keep.min_x {
            main.push(IVec3::new(x, ground_y, target.z));
        }
        main.push(IVec3::new(central_keep.min_x - 1, ground_y, hall.max_z));
        main.push(IVec3::new(central_keep.min_x, ground_y, hall.max_z));
        for x in hall.min_x..=hall.max_x {
            main.push(IVec3::new(x, ground_y, hall.max_z));
        }
        for (cell, _) in &stair {
            main.push(*cell);
        }
        main.push(IVec3::new(stair_x, levels.upper_floor_y, hall.min_z));
        main.push(IVec3::new(hall.min_x, levels.upper_floor_y, hall.min_z));
        // Tower routes: from the courtyard cell in front of the entrance
        // through the opening into the core.
        let mut routes = vec![main];
        for t in &towers {
            let e = t.entrance.base;
            let outside_x = if t.entrance.facing == Orientation::East {
                e.x + 1
            } else {
                e.x - 1
            };
            let (cx, cz) = t.core();
            routes.push(vec![
                IVec3::new(outside_x, ground_y, e.z),
                IVec3::new(e.x, ground_y, e.z),
                IVec3::new(cx, ground_y, cz),
            ]);
        }

        Self {
            footprint: pad,
            levels,
            gate,
            gatehouse,
            central_keep,
            keep_door,
            hall,
            crimson_tower,
            orange_tower,
            violet_tower,
            courtyard,
            stair,
            stair_well,
            interior_routes: routes,
        }
    }

    pub fn towers(&self) -> [&FortressTower; 3] {
        [&self.crimson_tower, &self.orange_tower, &self.violet_tower]
    }

    /// Whether a column is on the curtain wall (the footprint's outline).
    pub fn is_curtain_column(&self, x: i32, z: i32) -> bool {
        let f = self.footprint;
        f.contains(x, z) && (x == f.min_x || x == f.max_x || z == f.min_z || z == f.max_z)
    }

    pub fn tower_at(&self, x: i32, z: i32) -> Option<&FortressTower> {
        self.towers().into_iter().find(|t| t.bounds.contains(x, z))
    }

    pub fn is_tower_shell(&self, x: i32, z: i32) -> bool {
        self.tower_at(x, z).is_some_and(|t| t.is_shell(x, z))
    }

    pub fn is_keep_shell(&self, x: i32, z: i32) -> bool {
        let k = self.central_keep;
        k.contains(x, z) && (x == k.min_x || x == k.max_x || z == k.min_z || z == k.max_z)
    }

    pub fn is_keep_corner(&self, x: i32, z: i32) -> bool {
        let k = self.central_keep;
        (x == k.min_x || x == k.max_x) && (z == k.min_z || z == k.max_z)
    }

    /// The gate corridor: the columns between the two side rows.
    pub fn gate_corridor(&self) -> Vec<(i32, i32)> {
        let g = self.gatehouse;
        (g.min_x..=g.max_x)
            .flat_map(|x| ((g.min_z + 1)..g.max_z).map(move |z| (x, z)))
            .collect()
    }

    /// The main route's ground cells.
    pub fn main_route(&self) -> &[IVec3] {
        &self.interior_routes[0]
    }

    /// Checks the contract against the pad and the rest of the lower world.
    pub fn validate(
        &self,
        expansion: &WorldExpansionLayout,
        focal: &[IVec3],
        portal_route: &[IVec3],
    ) -> Result<(), String> {
        let pad = expansion.red_black_fortress_pad;
        let inside = |r: &ColumnRect| {
            r.min_x >= pad.min_x
                && r.max_x <= pad.max_x
                && r.min_z >= pad.min_z
                && r.max_z <= pad.max_z
        };
        if !inside(&self.central_keep)
            || !inside(&self.gatehouse)
            || !self.towers().iter().all(|t| inside(&t.bounds))
        {
            return Err("a building leaves the pad".into());
        }
        let towers = self.towers();
        for (i, a) in towers.iter().enumerate() {
            for b in &towers[i + 1..] {
                let ra = a.bounds;
                let rb = b.bounds;
                if ra.max_x >= rb.min_x
                    && rb.max_x >= ra.min_x
                    && ra.max_z >= rb.min_z
                    && rb.max_z >= ra.min_z
                {
                    return Err("towers overlap".into());
                }
                if a.family == b.family {
                    return Err("two towers share a family".into());
                }
            }
            if !self.courtyard.contains(&{
                let e = a.entrance.base;
                let x = if a.entrance.facing == Orientation::East {
                    e.x + 1
                } else {
                    e.x - 1
                };
                (x, e.z)
            }) {
                return Err("a tower entrance does not open on the courtyard".into());
            }
        }
        if self.gate.base.x != pad.min_x
            || !(self.gate.base.z..self.gate.base.z + self.gate.width)
                .contains(&expansion.red_black_path_target.z)
            || self.gate.base.y != self.levels.base_y
        {
            return Err("the gate does not face the approach".into());
        }
        if self.hall.width() < 3 || self.hall.depth() < 3 {
            return Err("the hall is too small".into());
        }
        if self.courtyard.len() < 8 {
            return Err("the courtyard is too small".into());
        }
        if self.levels.max_y <= -22 {
            return Err("the fortress reaches past the lower tip".into());
        }
        if focal.iter().any(|c| pad.contains(c.x, c.z)) {
            return Err("a family focal cell lies under the fortress".into());
        }
        if portal_route.iter().any(|c| pad.contains(c.x, c.z)) {
            return Err("the portal route crosses the fortress".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Dark foundation, courtyard and curtain (C196)
// ---------------------------------------------------------------------

const DARK_MASONRY_SALT: u32 = 0x17C0_0001;
/// Share of structural cells laid in smooth basalt instead of polished
/// blackstone bricks.
pub const BASALT_SHARE: f32 = 0.35;

/// A `Down` block of the inverted world.
pub fn down(block_type: BlockType, material: MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Down)
}

/// Polished blackstone bricks or smooth basalt for a structural cell, by
/// hash, so the dark body reads as masonry rather than a flat colour.
pub fn dark_masonry(seed: u32, cell: IVec3) -> BlockInstance {
    if expansion_hash(
        seed,
        cell.x * 3 + cell.y,
        cell.z * 5 - cell.y,
        DARK_MASONRY_SALT,
    ) < BASALT_SHARE
    {
        down(BlockType::SmoothBasalt, smooth_basalt_material_id())
    } else {
        down(
            BlockType::PolishedBlackstoneBricks,
            polished_blackstone_bricks_material_id(),
        )
    }
}

/// Whether a block is part of the fortress's dark structural body.
pub fn is_dark_structure(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::PolishedBlackstoneBricks | BlockType::SmoothBasalt | BlockType::Deepslate
    )
}

/// The fortress base, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FortressFoundation {
    /// Pad cells under every outline (polished blackstone bricks).
    pub footings: Vec<IVec3>,
    /// Pad cells of the courtyard and corridor (smooth basalt) and of the
    /// hall and tower cores (deepslate).
    pub floors: Vec<IVec3>,
    /// Curtain-wall cells, hollow behind, counted functionally upward.
    pub walls: Vec<IVec3>,
    /// The gate opening, left as air.
    pub gate_opening: Vec<IVec3>,
}

impl FortressFoundation {
    pub fn cells(&self) -> Vec<IVec3> {
        self.footings
            .iter()
            .chain(self.floors.iter())
            .chain(self.walls.iter())
            .copied()
            .collect()
    }
}

/// Lays the dark foundation and raises (functionally) the curtain wall.
///
/// The pad row becomes the floor plan: polished blackstone footings under
/// every outline, smooth basalt across the courtyard and gate corridor,
/// deepslate inside the keep and the tower cores. The curtain wall runs
/// `FORTRESS_WALL_COURSES` courses toward `-Y` on the footprint's outline,
/// one cell thick with nothing behind it; the gate opening stays open.
/// Every block is `Down`.
pub fn build_fortress_foundation(
    terrain: &TerrainConfig,
    layout: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> FortressFoundation {
    let mut built = FortressFoundation::default();
    let f = layout.footprint;
    let g = layout.levels.ground_y;
    let corridor = layout.gate_corridor();
    for z in f.min_z..=f.max_z {
        for x in f.min_x..=f.max_x {
            let cell = IVec3::new(x, g, z);
            let outline = !corridor.contains(&(x, z))
                && (layout.is_curtain_column(x, z)
                    || layout.is_tower_shell(x, z)
                    || layout.is_keep_shell(x, z)
                    || layout.gatehouse.contains(x, z));
            if outline {
                world.insert(
                    cell,
                    down(
                        BlockType::PolishedBlackstoneBricks,
                        polished_blackstone_bricks_material_id(),
                    ),
                );
                built.footings.push(cell);
            } else if layout.hall.contains(x, z) || layout.tower_at(x, z).is_some() {
                world.insert(cell, down(BlockType::Deepslate, deepslate_material_id()));
                built.floors.push(cell);
            } else {
                world.insert(
                    cell,
                    down(BlockType::SmoothBasalt, smooth_basalt_material_id()),
                );
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
            for y in (layout.levels.wall_top_y..=layout.levels.base_y).rev() {
                let cell = IVec3::new(x, y, z);
                if gate_cells.contains(&cell) {
                    world.remove(cell);
                    built.gate_opening.push(cell);
                    continue;
                }
                world.insert(cell, dark_masonry(terrain.seed, cell));
                built.walls.push(cell);
            }
        }
    }
    built
}

// ---------------------------------------------------------------------
// Coloured Red-Black bricks (C197)
// ---------------------------------------------------------------------

/// The family's glowing structural brick.
pub fn family_brick(family: Family) -> BlockInstance {
    let (block_type, material) = family.bricks();
    down(block_type, material)
}

/// Whether a block is one of the three coloured structural bricks.
pub fn is_family_brick(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::RedBlackDeepslateBricksCrimson
            | BlockType::RedBlackDeepslateBricksOrange
            | BlockType::RedBlackDeepslateBricksViolet
    )
}

/// The coloured bricks woven into the dark body, as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FortressColoredBricks {
    /// Corner columns of the curtain in the family of the tower they carry.
    pub edges: Vec<IVec3>,
    /// Alternating cells of the curtain's top course between the towers.
    pub bands: Vec<IVec3>,
    /// Jambs and lintel of the gate: Crimson left, Violet right, Orange above.
    pub gate_accents: Vec<IVec3>,
    /// The curtain cells of each tower's outer corners at the top course.
    pub tower_markers: Vec<IVec3>,
}

impl FortressColoredBricks {
    pub fn cells(&self) -> Vec<IVec3> {
        self.edges
            .iter()
            .chain(self.bands.iter())
            .chain(self.gate_accents.iter())
            .chain(self.tower_markers.iter())
            .copied()
            .collect()
    }
}

/// The family whose colour a curtain cell takes: the nearest tower's.
pub fn curtain_family(layout: &RedBlackFortressLayout, x: i32, z: i32) -> Family {
    let f = layout.footprint;
    let mid_x = (f.min_x + f.max_x) / 2;
    let mid_z = (f.min_z + f.max_z) / 2;
    if z > mid_z {
        Family::Violet
    } else if x > mid_x {
        Family::Orange
    } else {
        Family::Crimson
    }
}

/// Weaves the three coloured bricks into the curtain: the footprint's
/// corner columns under the towers, every other cell of the top course
/// between the towers, the gate's jambs and lintel, and the tower corners
/// on the curtain's top course. Only existing dark cells are recoloured,
/// so the body stays the majority and nothing is added.
pub fn weave_colored_bricks(
    layout: &RedBlackFortressLayout,
    world: &mut VoxelWorld,
) -> FortressColoredBricks {
    let mut built = FortressColoredBricks::default();
    let lv = layout.levels;
    let f = layout.footprint;
    let recolour = |world: &mut VoxelWorld, cell: IVec3, family: Family, list: &mut Vec<IVec3>| {
        let dark = world
            .get(cell)
            .is_some_and(|b| is_dark_structure(b.block_type()));
        if dark {
            world.insert(cell, family_brick(family));
            list.push(cell);
        }
    };
    // Edges: the corner columns that carry a tower.
    for t in layout.towers() {
        let r = t.bounds;
        for (x, z) in [
            (r.min_x, r.min_z),
            (r.max_x, r.min_z),
            (r.min_x, r.max_z),
            (r.max_x, r.max_z),
        ] {
            if !layout.is_curtain_column(x, z)
                || !((x == f.min_x || x == f.max_x) && (z == f.min_z || z == f.max_z))
            {
                continue;
            }
            for y in (lv.wall_top_y..=lv.base_y).rev() {
                recolour(world, IVec3::new(x, y, z), t.family, &mut built.edges);
            }
        }
        // Tower markers: the tower's other corners on the curtain, top course.
        for (x, z) in [
            (r.min_x, r.min_z),
            (r.max_x, r.min_z),
            (r.min_x, r.max_z),
            (r.max_x, r.max_z),
        ] {
            let footprint_corner = (x == f.min_x || x == f.max_x) && (z == f.min_z || z == f.max_z);
            if layout.is_curtain_column(x, z) && !footprint_corner {
                recolour(
                    world,
                    IVec3::new(x, lv.wall_top_y, z),
                    t.family,
                    &mut built.tower_markers,
                );
            }
        }
    }
    // Bands: the top course between towers, every other cell.
    for z in f.min_z..=f.max_z {
        for x in f.min_x..=f.max_x {
            if !layout.is_curtain_column(x, z)
                || layout.tower_at(x, z).is_some()
                || layout.gatehouse.contains(x, z)
            {
                continue;
            }
            let along = if z == f.min_z || z == f.max_z { x } else { z };
            if along % 2 != f.min_x % 2 {
                continue;
            }
            let family = curtain_family(layout, x, z);
            recolour(
                world,
                IVec3::new(x, lv.wall_top_y, z),
                family,
                &mut built.bands,
            );
        }
    }
    // Gate accents: left jamb Crimson, right jamb Violet, lintel Orange.
    let g = layout.gate;
    for dy in 0..2 {
        recolour(
            world,
            IVec3::new(g.base.x, g.base.y - dy, g.base.z - 1),
            Family::Crimson,
            &mut built.gate_accents,
        );
        recolour(
            world,
            IVec3::new(g.base.x, g.base.y - dy, g.base.z + g.width),
            Family::Violet,
            &mut built.gate_accents,
        );
    }
    for dz in 0..g.width {
        recolour(
            world,
            IVec3::new(g.base.x, g.base.y - 2, g.base.z + dz),
            Family::Orange,
            &mut built.gate_accents,
        );
    }
    built
}
