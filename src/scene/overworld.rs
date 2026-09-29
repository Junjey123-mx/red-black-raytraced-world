// The Overworld micro-scene: the deliberate features built on top of the
// generated terrain: pond and sandy shore, house, trees and the cobblestone
// path toward the reserved descent point. Each feature is a deterministic edit of the `VoxelWorld` driven
// by the terrain seed, never by a random-number crate.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::material_gallery::{
    glass_material_id, leaves_material_id, redstone_lamp_material_id, water_material_id,
};
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::{
    cobblestone_material_id, dirt_material_id, double_wood_slab_material_id, fence_material_id,
    log_material_id, sand_material_id, stone_block_material_id, wood_door_bottom_material_id,
    wood_door_top_material_id, wood_planks_material_id, wood_stairs_material_id,
};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::generator::{footprint_contains, footprint_radius};
use crate::scene::terrain::hash::hash01;
use crate::scene::voxel_world::VoxelWorld;

/// Highest occupied cell of column `(x, z)` between `top` and `bottom`.
pub fn column_top(world: &VoxelWorld, x: i32, z: i32, top: i32, bottom: i32) -> Option<i32> {
    (bottom..=top)
        .rev()
        .find(|&y| world.contains(IVec3::new(x, y, z)))
}

fn full(block_type: BlockType, material: crate::core::material::MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Up)
}

// ---------------------------------------------------------------------
// Pond
// ---------------------------------------------------------------------

/// `y` of the water surface: one block under the lowest bank around it.
pub const POND_WATER_LEVEL: i32 = 1;

/// Deepest the basin goes below the water surface (interior cells).
pub const POND_MAX_DEPTH: i32 = 2;

/// Seed salt of the shoreline choice.
const SHORE_SALT: u32 = 0x5A4D_0001;

/// Share of the bank cells that turn to sand (the rest stay grass).
const SHORE_SAND_SHARE: f32 = 0.7;

/// Columns of the pond basin: a 5 x 3 core in the low south-western hollow
/// of the official terrain, plus a few outlying cells so the outline reads
/// as an irregular pool rather than a rectangle.
pub fn pond_basin_columns() -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    for z in 17..=19 {
        for x in 5..=9 {
            cells.push((x, z));
        }
    }
    cells.extend([(10, 17), (10, 18), (10, 19), (4, 18), (8, 20)]);
    cells
}

/// Where the pond ended up: its water cells, its sand cells and the columns
/// that were dug.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PondLayout {
    pub water: Vec<IVec3>,
    pub sand: Vec<IVec3>,
    pub basin: Vec<(i32, i32)>,
}

/// Interior basin columns (all four side neighbors also in the basin) are
/// two blocks deep; the rest one.
fn basin_depth(basin: &[(i32, i32)], x: i32, z: i32) -> i32 {
    let interior = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .all(|(dx, dz)| basin.contains(&(x + dx, z + dz)));
    if interior { POND_MAX_DEPTH } else { 1 }
}

/// Digs the pond into the terrain and dresses part of its bank with sand.
///
/// Every basin column is excavated from its surface down to a floor
/// (`POND_WATER_LEVEL - depth`), the floor becomes sand (shallow) or stone
/// (deep), and water fills the column up to `POND_WATER_LEVEL`. Bank
/// columns (eight-neighbors of the basin that are not basin themselves)
/// deterministically swap their grass for sand, most but not all of them;
/// a bank that does not rise above the water gets a sand block so the pool
/// stays contained. The mass under the floor is untouched.
pub fn carve_pond(config: &TerrainConfig, world: &mut VoxelWorld) -> PondLayout {
    let basin = pond_basin_columns();
    let mut layout = PondLayout {
        basin: basin.clone(),
        ..Default::default()
    };
    let top = config.max_surface_height();
    let bottom = config.deepslate_level - 8;

    for &(x, z) in &basin {
        debug_assert!(footprint_contains(config, x, z));
        let Some(surface) = column_top(world, x, z, top, bottom) else {
            continue;
        };
        let floor = POND_WATER_LEVEL - basin_depth(&basin, x, z);
        for y in (floor + 1)..=surface {
            world.remove(IVec3::new(x, y, z));
        }
        let floor_block = if floor >= POND_WATER_LEVEL - 1 {
            full(BlockType::Sand, sand_material_id())
        } else {
            full(BlockType::Stone, stone_block_material_id())
        };
        world.insert(IVec3::new(x, floor, z), floor_block);
        for y in (floor + 1)..=POND_WATER_LEVEL {
            let cell = IVec3::new(x, y, z);
            world.insert(cell, full(BlockType::Water, water_material_id()));
            layout.water.push(cell);
        }
    }

    // Sandy bank.
    let mut bank: Vec<(i32, i32)> = Vec::new();
    for &(x, z) in &basin {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let cell = (x + dx, z + dz);
                if (dx != 0 || dz != 0) && !basin.contains(&cell) && !bank.contains(&cell) {
                    bank.push(cell);
                }
            }
        }
    }
    for (x, z) in bank {
        if !footprint_contains(config, x, z) {
            continue;
        }
        let Some(y) = column_top(world, x, z, top, bottom) else {
            continue;
        };
        // A bank at or under the water surface would let the water spill:
        // it gets a sand block on top so the pool is always contained.
        if y <= POND_WATER_LEVEL {
            let raised = IVec3::new(x, POND_WATER_LEVEL + 1, z);
            world.insert(raised, full(BlockType::Sand, sand_material_id()));
            layout.sand.push(raised);
            continue;
        }
        if hash01(config.seed ^ SHORE_SALT, x, z) >= SHORE_SAND_SHARE {
            continue;
        }
        let cell = IVec3::new(x, y, z);
        if world.get(cell).map(|b| b.block_type()) == Some(BlockType::Grass) {
            world.insert(cell, full(BlockType::Sand, sand_material_id()));
            layout.sand.push(cell);
        }
    }

    layout
}

// ---------------------------------------------------------------------
// Reservations shared by the features
// ---------------------------------------------------------------------

/// An inclusive rectangle of columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRect {
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
}

impl ColumnRect {
    pub const fn new(min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Self {
        Self {
            min_x,
            min_z,
            max_x,
            max_z,
        }
    }

    pub fn contains(&self, x: i32, z: i32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }

    /// The same rectangle grown by `margin` on every side.
    pub fn grown(&self, margin: i32) -> Self {
        Self::new(
            self.min_x - margin,
            self.min_z - margin,
            self.max_x + margin,
            self.max_z + margin,
        )
    }

    pub fn width(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn depth(&self) -> i32 {
        self.max_z - self.min_z + 1
    }
}

/// Columns of the house (7 x 5) on the central plateau of the official
/// terrain; its door opens to the south (+Z), toward the camera.
pub const HOUSE_FOOTPRINT: ColumnRect = ColumnRect::new(5, 8, 11, 12);

/// Corridor south of the house reserved for the cobblestone path that will
/// lead from the door toward the descent point.
pub const PATH_RESERVE: ColumnRect = ColumnRect::new(7, 13, 15, 17);

/// Column where the Overworld path ends: the reserved entrance of the
/// Gate 13 descent toward the rhombus waist.
pub const DESCENT_ENDPOINT: (i32, i32) = (14, 16);

/// Clearance a tree trunk keeps from the house footprint, the path corridor
/// and the pond basin: at least the canopy radius plus one, so no canopy
/// overhangs the roof, the path or the water.
pub const TREE_CLEARANCE: i32 = 3;

/// `true` when column `(x, z)` is claimed by the house, the path corridor or
/// the pond (each with `TREE_CLEARANCE`), so no trunk may stand there.
pub fn is_reserved_column(pond: &PondLayout, x: i32, z: i32) -> bool {
    HOUSE_FOOTPRINT.grown(TREE_CLEARANCE).contains(x, z)
        || PATH_RESERVE.grown(TREE_CLEARANCE - 1).contains(x, z)
        || pond
            .basin
            .iter()
            .any(|&(bx, bz)| (bx - x).abs() <= TREE_CLEARANCE && (bz - z).abs() <= TREE_CLEARANCE)
}

// ---------------------------------------------------------------------
// Trees
// ---------------------------------------------------------------------

/// Number of trees planted on the official terrain.
pub const TREE_COUNT: usize = 4;

/// Minimum column distance (Chebyshev) between two trunks.
pub const MIN_TREE_SPACING: i32 = 4;

/// Trunks stay inside this normalized footprint radius so no canopy hangs
/// over the void beyond the island's rim.
pub const MAX_TREE_RADIUS: f32 = 0.68;

/// Shortest and tallest trunks.
pub const MIN_TRUNK_HEIGHT: i32 = 4;
pub const MAX_TRUNK_HEIGHT: i32 = 5;

const TREE_SITE_SALT: u32 = 0x7EE5_0001;
const TRUNK_SALT: u32 = 0x7EE5_0002;
const CANOPY_SALT: u32 = 0x7EE5_0003;

/// One planted tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    /// The grass cell the trunk stands on.
    pub base: IVec3,
    pub trunk_height: i32,
    pub trunk: Vec<IVec3>,
    pub leaves: Vec<IVec3>,
}

/// Canopy cells of a tree whose trunk top is `top`, before removing the
/// trunk cells and any occupied cell. Two wide layers around the top of
/// the trunk, a 3 x 3 cap above and a small cross on top; a few corner and
/// arm cells are dropped per tree so no two canopies are identical.
fn canopy_cells(seed: u32, base: IVec3, top: i32) -> Vec<IVec3> {
    let mut cells = Vec::new();
    let keep = |salt: u32, dx: i32, dz: i32, dy: i32| {
        hash01(
            seed ^ CANOPY_SALT ^ salt,
            base.x * 7 + dx * 3 + dy,
            base.z * 11 + dz * 5,
        ) < 0.55
    };
    // Wide layers: the lower keeps most corners, the upper drops them all.
    for (dy, layer) in [(-1, 0u32), (0, 1u32)] {
        for dz in -2i32..=2 {
            for dx in -2i32..=2 {
                let corner = dx.abs() == 2 && dz.abs() == 2;
                if corner && (dy == 0 || !keep(layer, dx, dz, dy)) {
                    continue;
                }
                cells.push(IVec3::new(base.x + dx, top + dy, base.z + dz));
            }
        }
    }
    // Cap.
    for dz in -1..=1 {
        for dx in -1..=1 {
            cells.push(IVec3::new(base.x + dx, top + 1, base.z + dz));
        }
    }
    // Top cross: the center always, each arm only sometimes.
    cells.push(IVec3::new(base.x, top + 2, base.z));
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if keep(2, dx, dz, 2) {
            cells.push(IVec3::new(base.x + dx, top + 2, base.z + dz));
        }
    }
    cells
}

/// Plants `TREE_COUNT` trees on grass columns of the terrain.
///
/// Columns are visited in a deterministic seed-driven order and a column is
/// accepted when its top is grass, it is not reserved for the house, path or
/// pond, it lies well inside the island, and it keeps `MIN_TREE_SPACING`
/// from every tree already planted. Trunk height and canopy shape vary per
/// tree through the same hash, never through a random-number crate.
pub fn plant_trees(config: &TerrainConfig, world: &mut VoxelWorld, pond: &PondLayout) -> Vec<Tree> {
    let top = config.max_surface_height() + 2;
    let bottom = config.deepslate_level - 8;

    let mut sites: Vec<(i32, i32)> = (0..config.depth)
        .flat_map(|z| (0..config.width).map(move |x| (x, z)))
        .collect();
    sites.sort_by(|a, b| {
        let ka = hash01(config.seed ^ TREE_SITE_SALT, a.0, a.1);
        let kb = hash01(config.seed ^ TREE_SITE_SALT, b.0, b.1);
        kb.partial_cmp(&ka).unwrap().then(a.cmp(b))
    });

    let mut trees: Vec<Tree> = Vec::new();
    for (x, z) in sites {
        if trees.len() >= TREE_COUNT {
            break;
        }
        if !footprint_contains(config, x, z)
            || footprint_radius(config, x, z) > MAX_TREE_RADIUS
            || is_reserved_column(pond, x, z)
        {
            continue;
        }
        let Some(y) = column_top(world, x, z, top, bottom) else {
            continue;
        };
        let base = IVec3::new(x, y, z);
        if world.get(base).map(|b| b.block_type()) != Some(BlockType::Grass) {
            continue;
        }
        if trees.iter().any(|t| {
            (t.base.x - x).abs() < MIN_TREE_SPACING && (t.base.z - z).abs() < MIN_TREE_SPACING
        }) {
            continue;
        }

        // Height: the column hash offset by the planting order, so a small
        // stand of trees never ends up all the same height.
        let span = MAX_TRUNK_HEIGHT - MIN_TRUNK_HEIGHT + 1;
        let pick = (hash01(config.seed ^ TRUNK_SALT, x, z) * span as f32) as i32;
        let trunk_height = MIN_TRUNK_HEIGHT + (pick + trees.len() as i32) % span;
        let trunk_top = y + trunk_height;
        let trunk: Vec<IVec3> = (y + 1..=trunk_top).map(|ty| IVec3::new(x, ty, z)).collect();
        for cell in &trunk {
            world.insert(*cell, full(BlockType::Log, log_material_id()));
        }
        let mut leaves = Vec::new();
        for cell in canopy_cells(config.seed, base, trunk_top) {
            if world.contains(cell) {
                continue;
            }
            world.insert(cell, full(BlockType::Leaves, leaves_material_id()));
            leaves.push(cell);
        }
        trees.push(Tree {
            base,
            trunk_height,
            trunk,
            leaves,
        });
    }
    trees
}

// ---------------------------------------------------------------------
// House
// ---------------------------------------------------------------------

/// `y` of the house floor: the plateau under `HOUSE_FOOTPRINT` is leveled
/// to this height (its most common surface height), never the whole island.
pub const HOUSE_FLOOR_LEVEL: i32 = 6;

/// Courses of wall above the floor.
pub const HOUSE_WALL_HEIGHT: i32 = 3;

/// Column of the door in the south wall (`z = HOUSE_FOOTPRINT.max_z`).
pub const HOUSE_DOOR_X: i32 = 8;

/// Window openings `(x, z)` in the walls, one course above the floor.
pub const HOUSE_WINDOWS: [(i32, i32); 6] = [(6, 12), (10, 12), (6, 8), (10, 8), (5, 10), (11, 10)];

/// Where the house structure ended up.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HouseLayout {
    pub floor: Vec<IVec3>,
    /// Plank and double-slab wall cells (corners excluded).
    pub walls: Vec<IVec3>,
    /// Log corner posts.
    pub corners: Vec<IVec3>,
    /// The two empty cells of the entrance.
    pub door_opening: Vec<IVec3>,
    /// The empty window cells.
    pub window_openings: Vec<IVec3>,
}

impl HouseLayout {
    pub fn footprint(&self) -> ColumnRect {
        HOUSE_FOOTPRINT
    }

    pub fn floor_level(&self) -> i32 {
        HOUSE_FLOOR_LEVEL
    }
}

fn is_house_perimeter(x: i32, z: i32) -> bool {
    let r = HOUSE_FOOTPRINT;
    x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z
}

fn is_house_corner(x: i32, z: i32) -> bool {
    let r = HOUSE_FOOTPRINT;
    (x == r.min_x || x == r.max_x) && (z == r.min_z || z == r.max_z)
}

/// Levels the house footprint to `HOUSE_FLOOR_LEVEL` and raises the house
/// structure on it: a plank floor at floor level, three courses of wall on
/// the perimeter (a double-slab sill course under two plank courses) with
/// log posts at the four corners, a two-cell door opening in the south
/// wall and one-cell window openings. The roof and the fittings (door,
/// glass, fence, lamps) come separately.
pub fn build_house(config: &TerrainConfig, world: &mut VoxelWorld) -> HouseLayout {
    let r = HOUSE_FOOTPRINT;
    let top = config.max_surface_height() + 2;
    let bottom = config.deepslate_level - 8;
    let mut layout = HouseLayout::default();

    for z in r.min_z..=r.max_z {
        for x in r.min_x..=r.max_x {
            debug_assert!(footprint_contains(config, x, z));
            // Level: cut anything above the floor level, fill up to it.
            let Some(surface) = column_top(world, x, z, top, bottom) else {
                continue;
            };
            for y in (HOUSE_FLOOR_LEVEL + 1)..=surface {
                world.remove(IVec3::new(x, y, z));
            }
            for y in (surface + 1)..HOUSE_FLOOR_LEVEL {
                world.insert(
                    IVec3::new(x, y, z),
                    full(BlockType::Dirt, dirt_material_id()),
                );
            }
            // Buried grass under the floor reads as soil.
            let under = IVec3::new(x, HOUSE_FLOOR_LEVEL - 1, z);
            if world.get(under).map(|b| b.block_type()) == Some(BlockType::Grass) {
                world.insert(under, full(BlockType::Dirt, dirt_material_id()));
            }

            let floor = IVec3::new(x, HOUSE_FLOOR_LEVEL, z);
            world.insert(
                floor,
                full(BlockType::WoodPlanks, wood_planks_material_id()),
            );
            layout.floor.push(floor);

            if !is_house_perimeter(x, z) {
                continue;
            }
            for course in 1..=HOUSE_WALL_HEIGHT {
                let cell = IVec3::new(x, HOUSE_FLOOR_LEVEL + course, z);
                let is_door = z == r.max_z && x == HOUSE_DOOR_X && course <= 2;
                let is_window = course == 2 && HOUSE_WINDOWS.contains(&(x, z));
                if is_door {
                    world.remove(cell);
                    layout.door_opening.push(cell);
                } else if is_window {
                    world.remove(cell);
                    layout.window_openings.push(cell);
                } else if is_house_corner(x, z) {
                    world.insert(cell, full(BlockType::Log, log_material_id()));
                    layout.corners.push(cell);
                } else if course == 1 {
                    world.insert(
                        cell,
                        full(BlockType::DoubleWoodSlab, double_wood_slab_material_id()),
                    );
                    layout.walls.push(cell);
                } else {
                    world.insert(cell, full(BlockType::WoodPlanks, wood_planks_material_id()));
                    layout.walls.push(cell);
                }
            }
        }
    }
    layout
}

// ---------------------------------------------------------------------
// House exterior: roof, door, glass, fence, lamps
// ---------------------------------------------------------------------

/// Row of the roof ridge (the middle row of the footprint).
pub const HOUSE_RIDGE_Z: i32 = 10;

/// The roof overhangs the east and west walls by this many blocks.
pub const HOUSE_ROOF_OVERHANG: i32 = 1;

/// Row of the porch in front of the door (just south of the house).
pub const HOUSE_PORCH_Z: i32 = 13;

/// Lamp cells: in the south wall, one course up, flanking the door.
pub const HOUSE_LAMPS: [(i32, i32); 2] = [(HOUSE_DOOR_X - 1, 12), (HOUSE_DOOR_X + 1, 12)];

/// One roof block: its cell, block type and orientation.
pub type RoofBlock = (IVec3, BlockType, Orientation);

/// The fittings of the finished house exterior.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HouseExterior {
    pub roof: Vec<RoofBlock>,
    /// Lower and upper door cells.
    pub door: Vec<IVec3>,
    pub glass: Vec<IVec3>,
    pub fence: Vec<IVec3>,
    pub lamps: Vec<IVec3>,
}

/// The stair orientation of an eave row: a stair's raised half faces away
/// from its orientation (canonical `South` stairs are high on the north),
/// so rows south of the ridge face `South` and rows north of it `North`.
pub fn roof_stair_orientation(z: i32) -> Orientation {
    if z > HOUSE_RIDGE_Z {
        Orientation::South
    } else {
        Orientation::North
    }
}

/// Finishes the house exterior on top of `build_house`'s structure:
///
/// - a gabled stair roof rising row by row toward the ridge (planks fill
///   the body under the slope, a double slab caps the ridge), one block of
///   overhang east and west and a small stair awning over the door;
/// - the two-cell `WoodDoor` in the entrance, closed, facing south;
/// - `Glass` in every window opening;
/// - a short `Fence` line along the porch, open in front of the door;
/// - two lit `RedstoneLamp` blocks in the wall flanking the door.
pub fn furnish_house(config: &TerrainConfig, world: &mut VoxelWorld) -> HouseExterior {
    let r = HOUSE_FOOTPRINT;
    let top = config.max_surface_height() + 2;
    let bottom = config.deepslate_level - 8;
    let wall_top = HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT;
    let mut ext = HouseExterior::default();

    // Roof: layer k (k = 0, 1, 2) covers rows |z - ridge| <= 2 - k, with
    // stairs on its two outermost rows and planks inside; the ridge is a
    // double slab.
    let x_range = (r.min_x - HOUSE_ROOF_OVERHANG)..=(r.max_x + HOUSE_ROOF_OVERHANG);
    for layer in 0..=2 {
        let half = 2 - layer;
        let y = wall_top + 1 + layer;
        for z in (HOUSE_RIDGE_Z - half)..=(HOUSE_RIDGE_Z + half) {
            for x in x_range.clone() {
                let cell = IVec3::new(x, y, z);
                let (block_type, material, orientation) = if half == 0 {
                    (
                        BlockType::DoubleWoodSlab,
                        double_wood_slab_material_id(),
                        Orientation::Up,
                    )
                } else if (z - HOUSE_RIDGE_Z).abs() == half {
                    (
                        BlockType::WoodStairs,
                        wood_stairs_material_id(),
                        roof_stair_orientation(z),
                    )
                } else {
                    (
                        BlockType::WoodPlanks,
                        wood_planks_material_id(),
                        Orientation::Up,
                    )
                };
                world.insert(cell, BlockInstance::new(block_type, material, orientation));
                ext.roof.push((cell, block_type, orientation));
            }
        }
    }
    // Awning over the door.
    for x in (HOUSE_DOOR_X - 1)..=(HOUSE_DOOR_X + 1) {
        let cell = IVec3::new(x, wall_top + 1, HOUSE_PORCH_Z);
        world.insert(
            cell,
            BlockInstance::new(
                BlockType::WoodStairs,
                wood_stairs_material_id(),
                Orientation::South,
            ),
        );
        ext.roof
            .push((cell, BlockType::WoodStairs, Orientation::South));
    }

    // Door.
    for (dy, material) in [
        (1, wood_door_bottom_material_id()),
        (2, wood_door_top_material_id()),
    ] {
        let cell = IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + dy, r.max_z);
        world.insert(
            cell,
            BlockInstance::new(BlockType::WoodDoor, material, Orientation::South),
        );
        ext.door.push(cell);
    }

    // Glass.
    for (x, z) in HOUSE_WINDOWS {
        let cell = IVec3::new(x, HOUSE_FLOOR_LEVEL + 2, z);
        world.insert(cell, full(BlockType::Glass, glass_material_id()));
        ext.glass.push(cell);
    }

    // Porch fence, standing on the ground, open in front of the door.
    for x in r.min_x..=r.max_x {
        if x == HOUSE_DOOR_X {
            continue;
        }
        let Some(ground) = column_top(world, x, HOUSE_PORCH_Z, top, bottom) else {
            continue;
        };
        let cell = IVec3::new(x, ground + 1, HOUSE_PORCH_Z);
        world.insert(
            cell,
            BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::South),
        );
        ext.fence.push(cell);
    }

    // Lamps.
    for (x, z) in HOUSE_LAMPS {
        let cell = IVec3::new(x, HOUSE_FLOOR_LEVEL + 2, z);
        world.insert(
            cell,
            full(BlockType::RedstoneLampLit, redstone_lamp_material_id()),
        );
        ext.lamps.push(cell);
    }

    ext
}

// ---------------------------------------------------------------------
// Cobblestone path
// ---------------------------------------------------------------------

/// Where the path starts: the ground in front of the door.
pub const PATH_START: (i32, i32) = (HOUSE_DOOR_X, HOUSE_PORCH_Z);

/// Largest rise or fall between two consecutive path cells.
pub const PATH_MAX_STEP: i32 = 1;

/// Upper bound on path cells (the corridor is small; this only guards the
/// walk against a malformed configuration).
const PATH_MAX_LENGTH: usize = 64;

/// `true` for the blocks that make up the ground surface.
fn is_ground_block(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::Grass
            | BlockType::Dirt
            | BlockType::Sand
            | BlockType::Stone
            | BlockType::Cobblestone
            | BlockType::Deepslate
    )
}

/// Highest ground cell of column `(x, z)`, ignoring anything built above
/// it (fences, awnings, canopies).
pub fn ground_top(config: &TerrainConfig, world: &VoxelWorld, x: i32, z: i32) -> Option<i32> {
    let top = config.max_surface_height() + 16;
    let bottom = config.deepslate_level - 8;
    (bottom..=top).rev().find(|&y| {
        world
            .get(IVec3::new(x, y, z))
            .map(|b| is_ground_block(b.block_type()))
            .unwrap_or(false)
    })
}

/// The laid path: its surface cells in walking order from the door to the
/// descent endpoint, plus the landing around the endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathLayout {
    pub cells: Vec<IVec3>,
    pub landing: Vec<IVec3>,
}

impl PathLayout {
    pub fn start(&self) -> Option<IVec3> {
        self.cells.first().copied()
    }

    pub fn end(&self) -> Option<IVec3> {
        self.cells.last().copied()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// Lays the cobblestone path from `PATH_START` to `DESCENT_ENDPOINT`.
///
/// A greedy walk inside `PATH_RESERVE`: at each cell the two moves that
/// close in on the endpoint are considered (a sideways move only when
/// neither is possible), a move is valid when its column is not water, not
/// a trunk, has nothing built on its ground and is at most `PATH_MAX_STEP`
/// higher or lower, and the gentler move wins (ties go to the axis with
/// more distance left).
/// Each visited column's ground block becomes `Cobblestone`; the endpoint
/// gets a 2 x 2 cobblestone landing that marks the Gate 13 descent entrance.
pub fn lay_path(
    config: &TerrainConfig,
    world: &mut VoxelWorld,
    pond: &PondLayout,
    trees: &[Tree],
) -> PathLayout {
    // A column is closed to the path when it is water, a trunk, or has
    // something built right on its ground (a fence post).
    let blocked = |world: &VoxelWorld, x: i32, z: i32| {
        if pond.basin.contains(&(x, z)) || trees.iter().any(|t| t.base.x == x && t.base.z == z) {
            return true;
        }
        match ground_top(config, world, x, z) {
            Some(y) => world.contains(IVec3::new(x, y + 1, z)),
            None => true,
        }
    };
    let height = |world: &VoxelWorld, x: i32, z: i32| ground_top(config, world, x, z);

    let mut layout = PathLayout::default();
    let (mut x, mut z) = PATH_START;
    let (tx, tz) = DESCENT_ENDPOINT;
    let Some(mut y) = height(world, x, z) else {
        return layout;
    };
    let pave = |world: &mut VoxelWorld, x: i32, y: i32, z: i32, into: &mut Vec<IVec3>| {
        let cell = IVec3::new(x, y, z);
        world.insert(
            cell,
            full(BlockType::Cobblestone, cobblestone_material_id()),
        );
        into.push(cell);
    };
    pave(world, x, y, z, &mut layout.cells);

    while (x, z) != (tx, tz) && layout.cells.len() < PATH_MAX_LENGTH {
        let dx = (tx - x).signum();
        let dz = (tz - z).signum();
        // Closing moves first; sideways moves only when neither closes in.
        let closing = [(dx, 0), (0, dz)];
        let sideways = [(-dz, 0), (dz, 0), (0, -dx), (0, dx)];
        let valid = |world: &VoxelWorld, layout: &PathLayout, moves: &[(i32, i32)]| {
            let mut out: Vec<(i32, i32, i32)> = Vec::new();
            for &(mx, mz) in moves {
                if (mx, mz) == (0, 0) {
                    continue;
                }
                let (nx, nz) = (x + mx, z + mz);
                if !PATH_RESERVE.contains(nx, nz)
                    || blocked(world, nx, nz)
                    || layout.cells.iter().any(|c| c.x == nx && c.z == nz)
                {
                    continue;
                }
                let Some(ny) = height(world, nx, nz) else {
                    continue;
                };
                if (ny - y).abs() > PATH_MAX_STEP {
                    continue;
                }
                out.push((nx, nz, ny));
            }
            out
        };
        let mut candidates = valid(world, &layout, &closing);
        if candidates.is_empty() {
            candidates = valid(world, &layout, &sideways);
        }
        let Some(&(nx, nz, ny)) = candidates.iter().min_by_key(|&&(nx, nz, ny)| {
            let along_longer = if (tx - x).abs() >= (tz - z).abs() {
                nx != x
            } else {
                nz != z
            };
            ((ny - y).abs(), !along_longer)
        }) else {
            break;
        };
        (x, z, y) = (nx, nz, ny);
        pave(world, x, y, z, &mut layout.cells);
    }

    // Landing around the endpoint.
    for (lx, lz) in [(tx + 1, tz), (tx, tz + 1), (tx + 1, tz + 1)] {
        if !PATH_RESERVE.contains(lx, lz) || blocked(world, lx, lz) {
            continue;
        }
        if let Some(ly) = height(world, lx, lz) {
            pave(world, lx, ly, lz, &mut layout.landing);
        }
    }

    layout
}
