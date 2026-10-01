//! Gate 17.5: the Red-Black timber and vegetation family.
//!
//! Five blocks give the lower realm a non-stone material language: dark
//! corinto wood for paths, floors, rails and doors, and crimson foliage for
//! its trees. Every one of them is a recoloured twin of a certified
//! Overworld block and reuses that block's geometry, UV contract, alpha
//! mode and collision semantics; none of them is emissive. This module is
//! the single contract the materials, textures, catalog and world builders
//! derive from.

#![allow(dead_code)]

use crate::core::material::MaterialId;
use crate::scene::block_type::BlockType;

/// The five Red-Black timber blocks, in catalog order.
pub const RED_BLACK_TIMBER: [BlockType; 5] = [
    BlockType::RedBlackLeaves,
    BlockType::RedBlackLog,
    BlockType::RedBlackWoodPlanks,
    BlockType::RedBlackFence,
    BlockType::RedBlackWoodDoor,
];

/// Official blocks before Gate 17.5 (the Gate 17 catalog).
pub const PRE_TIMBER_OFFICIAL_COUNT: usize = 39;

/// Official blocks once the timber family is registered (Gate 17.5).
pub const TIMBER_OFFICIAL_COUNT: usize = PRE_TIMBER_OFFICIAL_COUNT + RED_BLACK_TIMBER.len();

/// Directory (relative to the repository root) of the timber textures.
pub const RED_BLACK_TIMBER_DIR: &str = "assets/textures/red_black_maze/timber";

pub fn red_black_leaves_material_id() -> MaterialId {
    MaterialId::new(64)
}

pub fn red_black_log_material_id() -> MaterialId {
    MaterialId::new(65)
}

pub fn red_black_wood_planks_material_id() -> MaterialId {
    MaterialId::new(66)
}

pub fn red_black_fence_material_id() -> MaterialId {
    MaterialId::new(67)
}

pub fn red_black_wood_door_bottom_material_id() -> MaterialId {
    MaterialId::new(68)
}

pub fn red_black_wood_door_top_material_id() -> MaterialId {
    MaterialId::new(69)
}

/// What a timber block reuses and how it must behave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimberContract {
    pub block_type: BlockType,
    /// The certified Overworld block whose geometry, UV contract, alpha
    /// mode and collision this block reuses.
    pub counterpart: BlockType,
    /// Material ids (the door has a lower and an upper half).
    pub material_ids: Vec<MaterialId>,
    /// Texture files, relative to the repository root.
    pub texture_paths: Vec<String>,
    /// Never: the family takes its light from the world, not from itself.
    pub emissive: bool,
    /// The camera passes through it (only the door).
    pub camera_passable: bool,
    /// Alpha-cutout texels are holes (leaves, door windows).
    pub cutout: bool,
    /// Its shape is the counterpart's partial geometry (fence, door).
    pub partial: bool,
}

/// The certified Overworld block a timber block is a twin of.
pub fn overworld_counterpart(block_type: BlockType) -> Option<BlockType> {
    match block_type {
        BlockType::RedBlackLeaves => Some(BlockType::Leaves),
        BlockType::RedBlackLog => Some(BlockType::Log),
        BlockType::RedBlackWoodPlanks => Some(BlockType::WoodPlanks),
        BlockType::RedBlackFence => Some(BlockType::Fence),
        BlockType::RedBlackWoodDoor => Some(BlockType::WoodDoor),
        _ => None,
    }
}

/// Whether a block belongs to the Red-Black timber family.
pub fn is_red_black_timber(block_type: BlockType) -> bool {
    overworld_counterpart(block_type).is_some()
}

/// The contract of one timber block, or `None` for any other block.
pub fn timber_contract(block_type: BlockType) -> Option<TimberContract> {
    let counterpart = overworld_counterpart(block_type)?;
    let path = |file: &str| format!("{RED_BLACK_TIMBER_DIR}/{file}");
    let (material_ids, texture_paths, camera_passable, cutout, partial) = match block_type {
        BlockType::RedBlackLeaves => (
            vec![red_black_leaves_material_id()],
            vec![path("leaves.png")],
            false,
            true,
            false,
        ),
        BlockType::RedBlackLog => (
            vec![red_black_log_material_id()],
            vec![path("log/end.png"), path("log/side.png")],
            false,
            false,
            false,
        ),
        BlockType::RedBlackWoodPlanks => (
            vec![red_black_wood_planks_material_id()],
            vec![path("wood_planks.png")],
            false,
            false,
            false,
        ),
        // Like the Overworld fence, it wears the planks texture.
        BlockType::RedBlackFence => (
            vec![red_black_fence_material_id()],
            vec![path("wood_planks.png")],
            false,
            false,
            true,
        ),
        BlockType::RedBlackWoodDoor => (
            vec![
                red_black_wood_door_bottom_material_id(),
                red_black_wood_door_top_material_id(),
            ],
            vec![
                path("wood_door/bottom.png"),
                path("wood_door/top.png"),
                path("wood_door/edge_bottom.png"),
                path("wood_door/edge_top.png"),
            ],
            true,
            true,
            true,
        ),
        _ => unreachable!("overworld_counterpart covers exactly the timber family"),
    };
    Some(TimberContract {
        block_type,
        counterpart,
        material_ids,
        texture_paths,
        emissive: false,
        camera_passable,
        cutout,
        partial,
    })
}
