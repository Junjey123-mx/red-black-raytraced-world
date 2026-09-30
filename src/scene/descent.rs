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
// Inverted route: from the portal exit to the Red-Black underside
// ---------------------------------------------------------------------

use crate::scene::red_black_maze::LowerMass;

/// Air cells kept clear under each tread of the inverted flight (the
/// inverted viewer stands "below" its floor in world coordinates): eye,
/// body and the camera's collision radius.
pub const ROUTE_CLEARANCE_HEIGHT: i32 = 3;

/// Where the post-portal route goes, derived from the real portal anchor
/// and the real lower mass: nothing here is a free-standing coordinate.
///
/// Crossing the membrane (facing South, at `wall_z`) leaves the camera on
/// the north side, inside the mass. A `Down` stair's walkable faces point
/// -Y and its raised half stays north, so an inverted flight climbs toward
/// -Z: the route therefore runs straight north from the aperture, one cell
/// lower (toward the inverted viewer's up, -Y) per cell, until the tunnel
/// under the treads breaks out of the underside at the north rim of the
/// diamond, where the -Y-facing Red-Black surface is the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvertedRouteLayout {
    /// Column of the route: the aperture's center column.
    pub x: i32,
    /// First cell behind the membrane, at the aperture's middle row.
    pub portal_exit: IVec3,
    /// Rows of the exit clearance (the aperture's rows) and its depth.
    pub exit_min_y: i32,
    pub exit_max_y: i32,
    pub exit_min_x: i32,
    pub exit_max_x: i32,
    /// The first `Down` tread: one cell north of the clearance, at the
    /// aperture's top row (one step above the exit floor, which is the
    /// rock over the aperture).
    pub first_step: IVec3,
    /// Per step: this much in x/z (north) and one cell down in y.
    pub direction: IVec3,
    /// Treads from `first_step`, each `direction` further and one lower.
    pub step_count: i32,
    /// The last tread: where the flight opens onto the underside.
    pub landing: IVec3,
    /// The rim cell of the mass beside the landing whose -Y face is the
    /// Red-Black ground the viewer meets when stepping off.
    pub surface_connection: IVec3,
    /// Air cells kept under every tread.
    pub clearance_height: i32,
}

impl InvertedRouteLayout {
    /// Tread `k` of the flight (`0` is `first_step`).
    pub fn tread(&self, k: i32) -> IVec3 {
        IVec3::new(
            self.first_step.x + self.direction.x * k,
            self.first_step.y - k,
            self.first_step.z + self.direction.z * k,
        )
    }

    /// Every tread cell, first to last.
    pub fn treads(&self) -> Vec<IVec3> {
        (0..self.step_count).map(|k| self.tread(k)).collect()
    }

    /// The air cells kept clear under tread `k` (world -Y side).
    pub fn clearance_under(&self, k: i32) -> Vec<IVec3> {
        let t = self.tread(k);
        (1..=self.clearance_height)
            .map(|d| IVec3::new(t.x, t.y - d, t.z))
            .collect()
    }

    /// The exit clearance: the aperture's rows and columns, one cell
    /// behind the membrane.
    pub fn exit_clearance(&self) -> Vec<IVec3> {
        let mut cells = Vec::new();
        for y in self.exit_min_y..=self.exit_max_y {
            for x in self.exit_min_x..=self.exit_max_x {
                cells.push(IVec3::new(x, y, self.portal_exit.z));
            }
        }
        cells
    }

    /// Derives the route from the portal anchor and the mass as built so
    /// far: the flight runs north from the aperture as long as there is
    /// rock to carry a tread, and stops at the last tread whose column
    /// still has mass above it (the rim).
    pub fn derive(rhombus: &RhombusConfig, lower: &LowerMass, world: &VoxelWorld) -> Self {
        let p = rhombus.portal;
        let x = p.center_x();
        let (exit_min_x, exit_max_x) = (p.min_x + 1, p.max_x - 1);
        let (exit_min_y, exit_max_y) = (p.min_y + 1, p.max_y - 1);
        // The membrane faces South: the far side is north (-Z).
        let direction = IVec3::new(0, 0, -1);
        let exit_z = p.wall_z + direction.z;
        let portal_exit = IVec3::new(x, (exit_min_y + exit_max_y) / 2, exit_z);
        let first_step = IVec3::new(x, exit_max_y, exit_z + direction.z);
        let mut layout = Self {
            x,
            portal_exit,
            exit_min_y,
            exit_max_y,
            exit_min_x,
            exit_max_x,
            first_step,
            direction,
            step_count: 0,
            landing: first_step,
            surface_connection: first_step,
            clearance_height: ROUTE_CLEARANCE_HEIGHT,
        };
        // Count treads: each needs rock to replace and rock above to hang
        // from, and must stay above the mass bottom.
        let mut count = 0;
        loop {
            let t = layout.tread(count);
            let above = IVec3::new(t.x, t.y + 1, t.z);
            if t.y - layout.clearance_height < lower.bottom_y
                || !world.contains(t)
                || !world.contains(above)
            {
                break;
            }
            count += 1;
        }
        layout.step_count = count.max(1);
        layout.landing = layout.tread(layout.step_count - 1);
        // The ground beside the landing: the mass cell just past the
        // clearance, one column east (the rim keeps its cells there).
        let l = layout.landing;
        layout.surface_connection = IVec3::new(l.x + 1, l.y - layout.clearance_height + 1, l.z);
        layout
    }
}

/// The route as built into the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvertedRoute {
    pub layout: InvertedRouteLayout,
    /// Cells carved for the exit clearance behind the membrane.
    pub exit_clearance: Vec<IVec3>,
    /// The `Down` stairs of the flight, first to last.
    pub treads: Vec<(IVec3, Orientation)>,
    /// Every cell removed for the route (clearance, tunnel, landing).
    pub dug: Vec<IVec3>,
    /// Support blocks added for the route.
    pub supports: Vec<IVec3>,
}

impl InvertedRoute {
    pub fn first(&self) -> Option<IVec3> {
        self.treads.first().map(|(c, _)| *c)
    }

    pub fn last(&self) -> Option<IVec3> {
        self.treads.last().map(|(c, _)| *c)
    }
}

/// Builds the post-portal route.
///
/// 1. The exit clearance immediately behind the membrane: the aperture's
///    own rows and columns, one cell deep, so the camera that crosses the
///    core anywhere in the opening is in air with the radius and head room
///    the collision volume needs. The rock over the aperture is the
///    inverted viewer's first floor.
/// 2. The flight: from `first_step`, one `Down` wooden stair per cell
///    north, each one cell lower (toward the inverted up), hanging from
///    the rock above it, with `clearance_height` cells of air carved under
///    it for the viewer. The first tread stands right behind the
///    clearance, so the climb starts the moment the camera has crossed.
///
/// Only structural mass is removed or replaced; the frame, the core and
/// every family block stay.
pub fn build_inverted_route(
    rhombus: &RhombusConfig,
    lower: &LowerMass,
    world: &mut VoxelWorld,
) -> InvertedRoute {
    let layout = InvertedRouteLayout::derive(rhombus, lower, world);
    let mut route = InvertedRoute {
        layout,
        exit_clearance: Vec::new(),
        treads: Vec::new(),
        dug: Vec::new(),
        supports: Vec::new(),
    };
    for cell in layout.exit_clearance() {
        if world.remove(cell).is_some() {
            route.dug.push(cell);
        }
        route.exit_clearance.push(cell);
    }
    for k in 0..layout.step_count {
        let tread = layout.tread(k);
        if world.remove(tread).is_some() {
            route.dug.push(tread);
        }
        world.insert(tread, stair(Orientation::Down));
        route.treads.push((tread, Orientation::Down));
        for cell in layout.clearance_under(k) {
            if world.remove(cell).is_some() {
                route.dug.push(cell);
            }
        }
    }
    route
}
