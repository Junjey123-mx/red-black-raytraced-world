//! Gate 17: the inverted Red-Black fortress on the Gate 15 fortress pad.
//!
//! Functional up is `-Y`: the pad row is the ground, courses are counted
//! downward (more negative `y` is higher for the Red-Black viewer), floors
//! are the lowest cell of a column and every block is `Orientation::Down`.
//! The layout (C195) is a contract derived from `WorldExpansionLayout`;
//! the builders that follow place blocks stage by stage.

#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::expansion::WorldExpansionLayout;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::ColumnRect;
use crate::scene::red_black_maze::Family;

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
        let mut main = Vec::new();
        main.push(IVec3::new(target.x - 2, ground_y, target.z));
        main.push(IVec3::new(target.x - 1, ground_y, target.z));
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
