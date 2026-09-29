// The inverted Red-Black half of the diamond: the structural mass under the
// portal waist (this file's first part) and, stage by stage, its -Y-facing
// surface and the Crimson, Orange and Violet families laid over it. It is
// the same `VoxelWorld` as the Overworld; the inversion is geometric and
// visual (Down-oriented blocks), never a second scene.
#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::core::math::IVec3;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::orientation::Orientation;
use crate::scene::overworld_blocks::{
    deepslate_material_id, polished_blackstone_bricks_material_id, smooth_basalt_material_id,
};
use crate::scene::rhombus::RhombusConfig;
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::hash::hash01;
use crate::scene::voxel_world::VoxelWorld;

/// Seed salt of the structural material mix.
const LOWER_MASS_SALT: u32 = 0x10E7_0001;

/// The lower mass as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LowerMass {
    pub cells: Vec<IVec3>,
    pub top_y: i32,
    pub bottom_y: i32,
}

/// Structural block of a lower-mass cell: deepslate right under the waist
/// (the geology continues through the portal), smooth basalt through the
/// widest band, polished blackstone bricks as a growing accent toward the
/// tip, mixed deterministically so the exterior reads as dark stone rather
/// than flat bands.
pub fn lower_mass_block(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    x: i32,
    y: i32,
    z: i32,
) -> (BlockType, MaterialId) {
    let depth = rhombus.shelf_y - y; // 1 at the first row under the shelf
    let span = (rhombus.shelf_y - rhombus.lower_tip_y).max(1) as f32;
    let t = depth as f32 / span; // 0 near the waist, 1 at the tip
    let roll = hash01(terrain.seed ^ LOWER_MASS_SALT, x * 7 + y, z * 11 - y);
    let deepslate_share = (1.0 - t * 2.2).clamp(0.05, 1.0);
    let bricks_share = (t - 0.35).clamp(0.0, 0.45);
    if roll < deepslate_share {
        (BlockType::Deepslate, deepslate_material_id())
    } else if roll > 1.0 - bricks_share {
        (
            BlockType::PolishedBlackstoneBricks,
            polished_blackstone_bricks_material_id(),
        )
    } else {
        (BlockType::SmoothBasalt, smooth_basalt_material_id())
    }
}

/// Fills every row from one under the shelf down to the tip with the
/// diamond's lower section (widening below the waist, then tapering to the
/// tip), skipping the cutaway quadrant, in the same world as the Overworld.
/// Every block is placed `Down`: the lower world's functional top faces -Y.
pub fn build_lower_mass(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    world: &mut VoxelWorld,
) -> LowerMass {
    let top_y = rhombus.shelf_y - 1;
    let bottom_y = rhombus.lower_tip_y;
    let mut mass = LowerMass {
        cells: Vec::new(),
        top_y,
        bottom_y,
    };
    for y in (bottom_y..=top_y).rev() {
        for z in -4..terrain.depth + 4 {
            for x in -4..terrain.width + 4 {
                if !rhombus.contains(terrain, x, y, z) || rhombus.is_cut(x, y, z) {
                    continue;
                }
                let cell = IVec3::new(x, y, z);
                if world.contains(cell) {
                    continue;
                }
                let (block_type, material) = lower_mass_block(terrain, rhombus, x, y, z);
                world.insert(
                    cell,
                    BlockInstance::new(block_type, material, Orientation::Down),
                );
                mass.cells.push(cell);
            }
        }
    }
    mass
}

// ---------------------------------------------------------------------
// The -Y-facing surface
// ---------------------------------------------------------------------

use crate::scene::overworld_blocks::{
    mycelium_material_id, red_black_deepslate_bricks_crimson_material_id,
    red_black_deepslate_bricks_orange_material_id, red_black_deepslate_bricks_violet_material_id,
};

/// The three visual families of the inverted world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    Crimson,
    Orange,
    Violet,
}

impl Family {
    pub const ALL: [Family; 3] = [Family::Crimson, Family::Orange, Family::Violet];

    /// The family's accented deepslate bricks.
    pub fn bricks(self) -> (BlockType, MaterialId) {
        match self {
            Family::Crimson => (
                BlockType::RedBlackDeepslateBricksCrimson,
                red_black_deepslate_bricks_crimson_material_id(),
            ),
            Family::Orange => (
                BlockType::RedBlackDeepslateBricksOrange,
                red_black_deepslate_bricks_orange_material_id(),
            ),
            Family::Violet => (
                BlockType::RedBlackDeepslateBricksViolet,
                red_black_deepslate_bricks_violet_material_id(),
            ),
        }
    }
}

/// Seed salt of the family-boundary jitter.
const FAMILY_SALT: u32 = 0xFA31_0001;
/// Seed salt of the surface material mix.
const SURFACE_SALT: u32 = 0x5F4C_0001;
/// Seed salt of the surface relief.
const RELIEF_SALT: u32 = 0x5F4C_0002;

/// Angular jitter of the family boundaries, in radians, so the three zones
/// interleave instead of meeting on straight lines.
pub const FAMILY_JITTER: f32 = 0.55;

/// Share of the lower columns whose surface is raised by one block.
pub const SURFACE_RELIEF_SHARE: f32 = 0.22;

/// The family a lower column belongs to: one of three angular sectors
/// around the footprint center (Crimson to the north and north-west, Orange
/// to the east, Violet to the south and south-west), with a deterministic
/// jitter of the boundary angle per column.
pub fn family_zone(terrain: &TerrainConfig, rhombus: &RhombusConfig, x: i32, z: i32) -> Family {
    let dx = (x as f32 + 0.5) - rhombus.center_x as f32;
    let dz = (z as f32 + 0.5) - rhombus.center_z as f32;
    let jitter = (hash01(terrain.seed ^ FAMILY_SALT, x, z) - 0.5) * 2.0 * FAMILY_JITTER;
    let angle = dz.atan2(dx) + jitter; // 0 = east, pi/2 = south, -pi/2 = north
    let third = std::f32::consts::PI * 2.0 / 3.0;
    let a = (angle + std::f32::consts::PI / 3.0).rem_euclid(std::f32::consts::PI * 2.0);
    if a < third {
        Family::Orange
    } else if a < 2.0 * third {
        Family::Violet
    } else {
        Family::Crimson
    }
}

/// The inverted surface as built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedBlackSurface {
    /// The lowest cell of every lower column (the ground of the inverted
    /// world), all `Down`.
    pub cells: Vec<IVec3>,
    /// Cells removed to raise part of the surface by one block.
    pub raised: Vec<IVec3>,
}

impl RedBlackSurface {
    /// The surface cell of column `(x, z)`, if the column has one.
    pub fn cell_at(&self, x: i32, z: i32) -> Option<IVec3> {
        self.cells.iter().copied().find(|c| c.x == x && c.z == z)
    }
}

/// Lowest existing cell of column `(x, z)` between `bottom` and `top`.
fn column_bottom(world: &VoxelWorld, x: i32, z: i32, bottom: i32, top: i32) -> Option<i32> {
    (bottom..=top).find(|&y| world.contains(IVec3::new(x, y, z)))
}

/// `true` for the structural blocks of the lower mass (the ones the surface
/// may replace).
fn is_structural(block_type: BlockType) -> bool {
    matches!(
        block_type,
        BlockType::Deepslate | BlockType::SmoothBasalt | BlockType::PolishedBlackstoneBricks
    )
}

/// Base ground block of a surface column: mycelium, smooth basalt, polished
/// blackstone bricks or the column's family bricks, mixed deterministically.
pub fn surface_block(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    x: i32,
    z: i32,
) -> (BlockType, MaterialId) {
    let roll = hash01(terrain.seed ^ SURFACE_SALT, x, z);
    if roll < 0.34 {
        (BlockType::Mycelium, mycelium_material_id())
    } else if roll < 0.62 {
        (BlockType::SmoothBasalt, smooth_basalt_material_id())
    } else if roll < 0.80 {
        (
            BlockType::PolishedBlackstoneBricks,
            polished_blackstone_bricks_material_id(),
        )
    } else {
        family_zone(terrain, rhombus, x, z).bricks()
    }
}

/// Turns the underside of the lower mass into the inverted world's ground:
/// for every lower column outside the cutaway, part of the columns lose
/// their lowest cell (one block of extra relief on top of the stepped
/// section), and the lowest remaining structural cell becomes a `Down`
/// surface block. Columns whose lowest cell belongs to the inverted
/// descent keep their stair.
pub fn build_red_black_surface(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    lower: &LowerMass,
    world: &mut VoxelWorld,
) -> RedBlackSurface {
    let mut surface = RedBlackSurface::default();
    for z in -4..terrain.depth + 4 {
        for x in -4..terrain.width + 4 {
            if rhombus.cutaway.contains_column(x, z) {
                continue;
            }
            let Some(mut y) = column_bottom(world, x, z, lower.bottom_y, lower.top_y) else {
                continue;
            };
            let is_mass = |world: &VoxelWorld, y: i32| {
                world
                    .get(IVec3::new(x, y, z))
                    .map(|b| is_structural(b.block_type()))
                    .unwrap_or(false)
            };
            if !is_mass(world, y) {
                continue;
            }
            // Relief: raise this column by one where the hash says so and
            // the column keeps at least two cells of mass above.
            let tall = is_mass(world, y + 1) && is_mass(world, y + 2);
            if tall && hash01(terrain.seed ^ RELIEF_SALT, x, z) < SURFACE_RELIEF_SHARE {
                let cell = IVec3::new(x, y, z);
                world.remove(cell);
                surface.raised.push(cell);
                y += 1;
            }
            let cell = IVec3::new(x, y, z);
            let (block_type, material) = surface_block(terrain, rhombus, x, z);
            world.insert(
                cell,
                BlockInstance::new(block_type, material, Orientation::Down),
            );
            surface.cells.push(cell);
        }
    }
    surface
}

// ---------------------------------------------------------------------
// Families
// ---------------------------------------------------------------------

use crate::scene::overworld_blocks::{
    crimson_diamond_material_id, crimson_heart_material_id, crying_obsidian_crimson_material_id,
    nether_wart_block_material_id,
};

/// Seed salt of the family composition rolls.
const FAMILY_ROLL_SALT: u32 = 0xFA31_0002;
/// Seed salt of the violet suit choice.
const SUIT_SALT: u32 = 0xFA31_0003;

/// One composed family: what it placed on and under the shared surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyLayout {
    pub family: Family,
    /// Suit-symbol blocks set into the surface.
    pub symbols: Vec<IVec3>,
    /// Crying-obsidian accents set into the surface.
    pub accents: Vec<IVec3>,
    /// Family ground blocks set into the surface (wart, blackstone, mycelium).
    pub ground: Vec<IVec3>,
    /// Family bricks hung one block under the surface (terrace steps).
    pub terraces: Vec<IVec3>,
    /// Structural extras hung under the surface (basalt columns, crystals).
    pub structures: Vec<IVec3>,
}

impl FamilyLayout {
    fn new(family: Family) -> Self {
        Self {
            family,
            symbols: Vec::new(),
            accents: Vec::new(),
            ground: Vec::new(),
            terraces: Vec::new(),
            structures: Vec::new(),
        }
    }

    /// Every cell the family placed.
    pub fn cells(&self) -> Vec<IVec3> {
        self.symbols
            .iter()
            .chain(self.accents.iter())
            .chain(self.ground.iter())
            .chain(self.terraces.iter())
            .chain(self.structures.iter())
            .copied()
            .collect()
    }
}

fn down(block_type: BlockType, material: MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, material, Orientation::Down)
}

/// Lays the Crimson family over its sector of the shared surface: hearts
/// and diamonds as emblems set into the ground, nether wart as organic
/// ground, a few crimson crying-obsidian accents, and crimson-accented
/// brick steps hung one block under the surface as small terraces. Every
/// block replaces or hangs from an existing surface cell, so the family
/// stays part of the one connected lower world.
pub fn compose_crimson(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    surface: &RedBlackSurface,
    world: &mut VoxelWorld,
) -> FamilyLayout {
    let mut layout = FamilyLayout::new(Family::Crimson);
    for cell in &surface.cells {
        if family_zone(terrain, rhombus, cell.x, cell.z) != Family::Crimson {
            continue;
        }
        let roll = hash01(terrain.seed ^ FAMILY_ROLL_SALT, cell.x, cell.z);
        let under = IVec3::new(cell.x, cell.y - 1, cell.z);
        if roll < 0.12 {
            world.insert(
                *cell,
                down(BlockType::CrimsonHeart, crimson_heart_material_id()),
            );
            layout.symbols.push(*cell);
        } else if roll < 0.22 {
            world.insert(
                *cell,
                down(BlockType::CrimsonDiamond, crimson_diamond_material_id()),
            );
            layout.symbols.push(*cell);
        } else if roll < 0.40 {
            world.insert(
                *cell,
                down(BlockType::NetherWartBlock, nether_wart_block_material_id()),
            );
            layout.ground.push(*cell);
        } else if roll < 0.46 {
            world.insert(
                *cell,
                down(
                    BlockType::CryingObsidianCrimson,
                    crying_obsidian_crimson_material_id(),
                ),
            );
            layout.accents.push(*cell);
        } else if roll < 0.60 && !world.contains(under) && under.y >= rhombus.lower_tip_y {
            let (block_type, material) = Family::Crimson.bricks();
            world.insert(under, down(block_type, material));
            layout.terraces.push(under);
        }
    }
    layout
}

use crate::scene::overworld_blocks::{
    crying_obsidian_orange_material_id, orange_club_material_id, orange_spade_material_id,
};

/// Lays the Orange / Black Club family over its sector: clubs and spades
/// as emblems, polished blackstone as paved ground, short smooth-basalt
/// columns hanging under the surface, a few orange crying-obsidian
/// accents, and orange-accented brick steps as terraces. The darker,
/// stonier counterpart of Crimson, blended into the same ground.
pub fn compose_orange(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    surface: &RedBlackSurface,
    world: &mut VoxelWorld,
) -> FamilyLayout {
    let mut layout = FamilyLayout::new(Family::Orange);
    for cell in &surface.cells {
        if family_zone(terrain, rhombus, cell.x, cell.z) != Family::Orange {
            continue;
        }
        let roll = hash01(terrain.seed ^ FAMILY_ROLL_SALT, cell.x, cell.z);
        let under = IVec3::new(cell.x, cell.y - 1, cell.z);
        let under2 = IVec3::new(cell.x, cell.y - 2, cell.z);
        if roll < 0.10 {
            world.insert(
                *cell,
                down(BlockType::OrangeClub, orange_club_material_id()),
            );
            layout.symbols.push(*cell);
        } else if roll < 0.19 {
            world.insert(
                *cell,
                down(BlockType::OrangeSpade, orange_spade_material_id()),
            );
            layout.symbols.push(*cell);
        } else if roll < 0.34 {
            world.insert(
                *cell,
                down(
                    BlockType::PolishedBlackstoneBricks,
                    polished_blackstone_bricks_material_id(),
                ),
            );
            layout.ground.push(*cell);
        } else if roll < 0.39 {
            world.insert(
                *cell,
                down(
                    BlockType::CryingObsidianOrange,
                    crying_obsidian_orange_material_id(),
                ),
            );
            layout.accents.push(*cell);
        } else if roll < 0.47
            && !world.contains(under)
            && !world.contains(under2)
            && under2.y >= rhombus.lower_tip_y
        {
            for c in [under, under2] {
                world.insert(
                    c,
                    down(BlockType::SmoothBasalt, smooth_basalt_material_id()),
                );
                layout.structures.push(c);
            }
        } else if roll < 0.59 && !world.contains(under) && under.y >= rhombus.lower_tip_y {
            let (block_type, material) = Family::Orange.bricks();
            world.insert(under, down(block_type, material));
            layout.terraces.push(under);
        }
    }
    layout
}

use crate::scene::overworld_blocks::{
    amethyst_cluster_material_id, budding_amethyst_material_id, crying_obsidian_violet_material_id,
    purple_club_material_id, purple_diamond_material_id, purple_heart_material_id,
    purple_spade_material_id,
};

/// Lays the Violet / House of Cards family over its sector: the four
/// violet suits as emblems, mycelium as the dominant organic ground,
/// budding amethyst set into the ground with amethyst clusters growing
/// down from it (`Down` growth: base on the block above, tips toward -Y),
/// a few violet crying-obsidian accents, and violet-accented brick steps.
pub fn compose_violet(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    surface: &RedBlackSurface,
    world: &mut VoxelWorld,
) -> FamilyLayout {
    let mut layout = FamilyLayout::new(Family::Violet);
    for cell in &surface.cells {
        if family_zone(terrain, rhombus, cell.x, cell.z) != Family::Violet {
            continue;
        }
        let roll = hash01(terrain.seed ^ FAMILY_ROLL_SALT, cell.x, cell.z);
        let under = IVec3::new(cell.x, cell.y - 1, cell.z);
        let suit = [
            (BlockType::PurpleHeart, purple_heart_material_id()),
            (BlockType::PurpleDiamond, purple_diamond_material_id()),
            (BlockType::PurpleClub, purple_club_material_id()),
            (BlockType::PurpleSpade, purple_spade_material_id()),
        ];
        if roll < 0.20 {
            let pick = (hash01(terrain.seed ^ SUIT_SALT, cell.x, cell.z) * 4.0) as usize % 4;
            let (block_type, material) = suit[pick];
            world.insert(*cell, down(block_type, material));
            layout.symbols.push(*cell);
        } else if roll < 0.36 {
            world.insert(*cell, down(BlockType::Mycelium, mycelium_material_id()));
            layout.ground.push(*cell);
        } else if roll < 0.44 {
            world.insert(
                *cell,
                down(BlockType::BuddingAmethyst, budding_amethyst_material_id()),
            );
            layout.ground.push(*cell);
            if roll < 0.41 && !world.contains(under) && under.y >= rhombus.lower_tip_y {
                world.insert(
                    under,
                    down(BlockType::AmethystCluster, amethyst_cluster_material_id()),
                );
                layout.structures.push(under);
            }
        } else if roll < 0.49 {
            world.insert(
                *cell,
                down(
                    BlockType::CryingObsidianViolet,
                    crying_obsidian_violet_material_id(),
                ),
            );
            layout.accents.push(*cell);
        } else if roll < 0.60 && !world.contains(under) && under.y >= rhombus.lower_tip_y {
            let (block_type, material) = Family::Violet.bricks();
            world.insert(under, down(block_type, material));
            layout.terraces.push(under);
        }
    }
    layout
}

// ---------------------------------------------------------------------
// Family transitions
// ---------------------------------------------------------------------

/// Seed salt of the transition mix.
const TRANSITION_SALT: u32 = 0xFA31_0004;

/// The blended boundaries between the families.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FamilyTransitions {
    /// Surface columns that touch a column of another family.
    pub boundary: Vec<(i32, i32)>,
    /// Boundary surface cells rewritten with bridge materials.
    pub bridged: Vec<IVec3>,
}

/// `true` when column `(x, z)` lies on a family boundary: one of its four
/// side neighbors belongs to a different family.
pub fn is_boundary_column(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    x: i32,
    z: i32,
) -> bool {
    let own = family_zone(terrain, rhombus, x, z);
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(dx, dz)| family_zone(terrain, rhombus, x + dx, z + dz) != own)
}

/// The family of the first side neighbor that differs from the column's own.
fn neighbor_family(terrain: &TerrainConfig, rhombus: &RhombusConfig, x: i32, z: i32) -> Family {
    let own = family_zone(terrain, rhombus, x, z);
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|(dx, dz)| family_zone(terrain, rhombus, x + dx, z + dz))
        .find(|f| *f != own)
        .unwrap_or(own)
}

/// The crying-obsidian accent of a family.
fn crying_accent(family: Family) -> (BlockType, MaterialId) {
    match family {
        Family::Crimson => (
            BlockType::CryingObsidianCrimson,
            crying_obsidian_crimson_material_id(),
        ),
        Family::Orange => (
            BlockType::CryingObsidianOrange,
            crying_obsidian_orange_material_id(),
        ),
        Family::Violet => (
            BlockType::CryingObsidianViolet,
            crying_obsidian_violet_material_id(),
        ),
    }
}

/// Softens the family boundaries: every boundary surface cell is rewritten
/// with a bridge material (smooth basalt, polished blackstone, mycelium,
/// the neighboring family's bricks, or rarely its crying-obsidian accent),
/// so no suit symbol sits on a border and the three regions interleave
/// through shared dark stone instead of meeting on hard lines. Hanging
/// extras (terraces, columns, crystals) are left in place.
pub fn blend_family_transitions(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    surface: &RedBlackSurface,
    world: &mut VoxelWorld,
) -> FamilyTransitions {
    let mut transitions = FamilyTransitions::default();
    for cell in &surface.cells {
        if !is_boundary_column(terrain, rhombus, cell.x, cell.z) {
            continue;
        }
        // A budding block hosting a hanging crystal keeps its crystal's root.
        let hosts_crystal = world
            .get(IVec3::new(cell.x, cell.y - 1, cell.z))
            .map(|b| b.block_type() == BlockType::AmethystCluster)
            .unwrap_or(false);
        if hosts_crystal {
            continue;
        }
        transitions.boundary.push((cell.x, cell.z));
        let roll = hash01(terrain.seed ^ TRANSITION_SALT, cell.x, cell.z);
        let neighbor = neighbor_family(terrain, rhombus, cell.x, cell.z);
        let (block_type, material) = if roll < 0.36 {
            (BlockType::SmoothBasalt, smooth_basalt_material_id())
        } else if roll < 0.62 {
            (
                BlockType::PolishedBlackstoneBricks,
                polished_blackstone_bricks_material_id(),
            )
        } else if roll < 0.78 {
            (BlockType::Mycelium, mycelium_material_id())
        } else if roll < 0.95 {
            neighbor.bricks()
        } else {
            crying_accent(neighbor)
        };
        world.insert(*cell, down(block_type, material));
        transitions.bridged.push(*cell);
    }
    transitions
}

// ---------------------------------------------------------------------
// Accent lights
// ---------------------------------------------------------------------

use crate::core::color::Color;
use crate::core::math::Vec3;
use crate::scene::light::{Light, PointLight};

/// Intensity of each family's accent light: a fifth of the sun, enough to
/// lift the underside out of pure ambient without competing with the
/// emissive symbols.
pub const FAMILY_LIGHT_INTENSITY: f32 = 0.22;

/// How far under the diamond's tip the accent lights hang, so every
/// -Y-facing surface cell sees them.
pub const FAMILY_LIGHT_DROP: i32 = 3;

impl Family {
    /// The family's light color: its palette's bright tone.
    pub fn light_color(self) -> Color {
        match self {
            Family::Crimson => Color::new(0.82, 0.12, 0.17, 1.0),
            Family::Orange => Color::new(1.00, 0.48, 0.00, 1.0),
            Family::Violet => Color::new(0.76, 0.24, 1.00, 1.0),
        }
    }
}

/// World-space position of a family's accent light: under the centroid of
/// the family's columns at the lower half's widest row, `FAMILY_LIGHT_DROP`
/// blocks below the tip. Derived from the zone geometry alone, so it needs
/// no built world.
pub fn family_light_position(
    terrain: &TerrainConfig,
    rhombus: &RhombusConfig,
    family: Family,
) -> Vec3 {
    let mut sum_x = 0.0;
    let mut sum_z = 0.0;
    let mut count = 0.0;
    for z in -4..terrain.depth + 4 {
        for x in -4..terrain.width + 4 {
            if rhombus.contains(terrain, x, rhombus.lower_widest_y, z)
                && !rhombus.cutaway.contains_column(x, z)
                && family_zone(terrain, rhombus, x, z) == family
            {
                sum_x += x as f32 + 0.5;
                sum_z += z as f32 + 0.5;
                count += 1.0;
            }
        }
    }
    let (cx, cz) = if count > 0.0 {
        (sum_x / count, sum_z / count)
    } else {
        (rhombus.center_x as f32, rhombus.center_z as f32)
    };
    Vec3::new(cx, (rhombus.lower_tip_y - FAMILY_LIGHT_DROP) as f32, cz)
}

/// One representative point light per family.
pub fn family_lights(terrain: &TerrainConfig, rhombus: &RhombusConfig) -> Vec<Light> {
    Family::ALL
        .iter()
        .map(|&family| {
            Light::Point(PointLight::new(
                family_light_position(terrain, rhombus, family),
                family.light_color(),
                FAMILY_LIGHT_INTENSITY,
            ))
        })
        .collect()
}
