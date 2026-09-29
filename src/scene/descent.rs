// The routes that join the two worlds through the waist: the upper descent
// from the Gate 12 path endpoint down to the portal approach (this file's
// first half) and, later, the inverted descent hanging under the shelf.
// Both are ordinary blocks in the one `VoxelWorld`: wooden stairs resting on
// natural rock, a deepslate shelf, and a brick landing in front of the
// portal opening.
#![allow(dead_code)]

use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::material_gallery::deepslate_bricks_material_id;
use crate::scene::orientation::Orientation;
use crate::scene::overworld::{DESCENT_ENDPOINT, ground_top};
use crate::scene::overworld_blocks::{deepslate_material_id, wood_stairs_material_id};
use crate::scene::rhombus::{LOWER_WIDEST_RADIUS, RhombusConfig};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::generator::footprint_radius;
use crate::scene::voxel_world::VoxelWorld;

/// The stepped pit dug beside the cutaway: the two columns west of the
/// endpoint's column plus the endpoint's own column south of its landing,
/// four rows deep. A serpentine of stairs visits every column once, each
/// one block lower than the last, and leaves the pit eastward onto the
/// shelf. Columns are `(x, z)` offsets from `DESCENT_ENDPOINT`; the first
/// one is right beside the landing.
pub const PIT_SERPENTINE: [(i32, i32); 11] = [
    (-1, 1),
    (-2, 1),
    (-2, 2),
    (-2, 3),
    (-2, 4),
    (-1, 4),
    (0, 4),
    (0, 3),
    (-1, 3),
    (-1, 2),
    (0, 2),
];

/// The upper descent as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpperDescent {
    /// Every stair tread in walking order (cell, orientation), from the
    /// first step beside the landing to the last one on the shelf.
    pub treads: Vec<(IVec3, Orientation)>,
    /// Terrain cells removed to open the pit.
    pub dug: Vec<IVec3>,
    /// Shelf cells added under the cutaway (the cut's floor).
    pub shelf: Vec<IVec3>,
    /// Brick landing in front of the portal opening.
    pub approach: Vec<IVec3>,
    /// Deepslate cells added under treads the taper left unsupported.
    pub footings: Vec<IVec3>,
}

impl UpperDescent {
    pub fn first(&self) -> Option<IVec3> {
        self.treads.first().map(|(c, _)| *c)
    }

    pub fn last(&self) -> Option<IVec3> {
        self.treads.last().map(|(c, _)| *c)
    }
}

/// Orientation of a stair whose raised half must face the cell the walker
/// comes from (`from`), one step away from `at` on the ground plane.
pub fn stair_facing_uphill(at: IVec3, from: IVec3) -> Orientation {
    // Canonical stairs (South) are raised on their north half; East stairs
    // on the west, West on the east, North on the south.
    match (from.x - at.x, from.z - at.z) {
        (0, -1) => Orientation::South,
        (0, 1) => Orientation::North,
        (-1, 0) => Orientation::East,
        (1, 0) => Orientation::West,
        _ => Orientation::South,
    }
}

fn stair(orientation: Orientation) -> BlockInstance {
    BlockInstance::new(
        BlockType::WoodStairs,
        wood_stairs_material_id(),
        orientation,
    )
}

/// Builds the upper descent:
///
/// 1. the shelf plate: row `shelf_y` across the whole cutaway quadrant
///    (out to the lower half's widest radius), so the cut has a floor;
/// 2. the stepped pit: for each serpentine column, every terrain cell above
///    the tread is removed and a wooden stair is placed on the natural rock;
/// 3. the exit stair onto the shelf and the brick landing in front of the
///    portal opening.
pub fn build_upper_descent(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    world: &mut VoxelWorld,
) -> UpperDescent {
    let mut descent = UpperDescent::default();
    let deepslate = || {
        BlockInstance::new(
            BlockType::Deepslate,
            deepslate_material_id(),
            Orientation::Up,
        )
    };

    // 1. Shelf plate.
    for z in rhombus.cutaway.min_z..terrain.depth + 4 {
        for x in rhombus.cutaway.min_x..terrain.width + 4 {
            if footprint_radius(terrain, x, z) > LOWER_WIDEST_RADIUS {
                continue;
            }
            let cell = IVec3::new(x, rhombus.shelf_y, z);
            if !world.contains(cell) {
                world.insert(cell, deepslate());
                descent.shelf.push(cell);
            }
        }
    }

    // 2. Stepped pit.
    let (ex, ez) = DESCENT_ENDPOINT;
    let landing = IVec3::new(
        ex,
        ground_top(terrain, world, ex, ez + 1).unwrap_or(0),
        ez + 1,
    );
    let mut previous = landing;
    let mut y = landing.y - 1;
    for (dx, dz) in PIT_SERPENTINE {
        let (x, z) = (ex + dx, ez + dz);
        let top = terrain.max_surface_height() + 2;
        for dy in (y + 1)..=top {
            let cell = IVec3::new(x, dy, z);
            if world.remove(cell).is_some() {
                descent.dug.push(cell);
            }
        }
        let at = IVec3::new(x, y, z);
        let orientation = stair_facing_uphill(at, previous);
        if world.remove(at).is_some() {
            descent.dug.push(at);
        }
        world.insert(at, stair(orientation));
        descent.treads.push((at, orientation));
        previous = at;
        y -= 1;
    }

    // 3. Exit onto the shelf: one more stair inside the cutaway, standing on
    //    a deepslate step over the shelf plate, then the brick landing in
    //    front of the portal.
    let exit = IVec3::new(previous.x + 1, y, previous.z);
    debug_assert_eq!(y, rhombus.shelf_y + 2);
    let orientation = stair_facing_uphill(exit, previous);
    world.insert(exit, stair(orientation));
    descent.treads.push((exit, orientation));

    // Every tread rests on rock: where the diamond's taper leaves a gap
    // under a tread, a short deepslate footing fills it down to the mass.
    let treads: Vec<IVec3> = descent.treads.iter().map(|(c, _)| *c).collect();
    for tread in treads {
        let mut y = tread.y - 1;
        while y >= rhombus.shelf_y && !world.contains(IVec3::new(tread.x, y, tread.z)) {
            let cell = IVec3::new(tread.x, y, tread.z);
            world.insert(cell, deepslate());
            descent.footings.push(cell);
            y -= 1;
        }
    }

    let p = rhombus.portal;
    for z in (p.wall_z + 1)..=(p.wall_z + 2) {
        for x in (p.min_x + 1)..p.max_x {
            let cell = IVec3::new(x, rhombus.shelf_y, z);
            world.insert(
                cell,
                BlockInstance::new(
                    BlockType::DeepslateBricks,
                    deepslate_bricks_material_id(),
                    Orientation::Up,
                ),
            );
            descent.approach.push(cell);
        }
    }

    descent
}

// ---------------------------------------------------------------------
// Inverted descent: the hanging flight under the shelf
// ---------------------------------------------------------------------

use crate::scene::red_black_maze::LowerMass;

/// The inverted descent as built: a straight flight of `Down`-oriented
/// stairs hanging from the lower mass, one row under the shelf, running
/// south from the portal's side along the cutaway's west wall until it
/// emerges through the lower surface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InvertedDescent {
    /// Every tread in walking order (all `Orientation::Down`).
    pub treads: Vec<(IVec3, Orientation)>,
    /// Lower-mass cells removed under the treads (the trench that opens the
    /// flight to the inverted world's sky).
    pub dug: Vec<IVec3>,
}

impl InvertedDescent {
    pub fn first(&self) -> Option<IVec3> {
        self.treads.first().map(|(c, _)| *c)
    }

    pub fn last(&self) -> Option<IVec3> {
        self.treads.last().map(|(c, _)| *c)
    }
}

/// Builds the inverted descent. Each tread `(x, y, z)` needs the cell above
/// it (its ceiling, the inverted world's ground) to be lower mass; every
/// mass cell under the tread is removed so the tread itself becomes part of
/// the -Y-facing surface. The flight descends one row per step southward
/// (`Down` stairs are raised on their north half, i.e. toward the previous
/// step) and stops once nothing of the mass lies under the last tread.
pub fn build_inverted_descent(
    rhombus: &RhombusConfig,
    lower: &LowerMass,
    world: &mut VoxelWorld,
) -> InvertedDescent {
    let mut descent = InvertedDescent::default();
    let x = rhombus.cutaway.min_x - 1;
    let z0 = rhombus.cutaway.min_z;
    let mut y = rhombus.shelf_y - 1;
    let mut z = z0;
    loop {
        let tread = IVec3::new(x, y, z);
        let ceiling = IVec3::new(x, y + 1, z);
        if !world.contains(ceiling) {
            break;
        }
        if world.remove(tread).is_some() {
            descent.dug.push(tread);
        }
        world.insert(tread, stair(Orientation::Down));
        descent.treads.push((tread, Orientation::Down));
        // Open the trench under the tread down to the underside.
        let mut below = y - 1;
        let mut emerged = true;
        while below >= lower.bottom_y {
            let cell = IVec3::new(x, below, z);
            if world.remove(cell).is_some() {
                descent.dug.push(cell);
                emerged = false;
            }
            below -= 1;
        }
        if emerged {
            break;
        }
        y -= 1;
        z += 1;
    }
    descent
}
