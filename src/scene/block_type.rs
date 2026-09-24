// Nominal catalog stage: lands ahead of the BlockInstance/VoxelWorld work
// that will store and consume these identifiers.
#![allow(dead_code)]

/// Lightweight, deterministic identifier for the approved block catalog.
/// A fieldless enum on purpose: it carries no material, texture, or
/// geometry, so a block type only names a role and never activates
/// rendering behavior by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    // Overworld
    Grass,
    Dirt,
    Stone,
    Cobblestone,
    Sand,
    Water,
    Deepslate,
    DeepslateBricks,
    Log,
    Leaves,
    WoodPlanks,
    DoubleWoodSlab,
    WoodStairs,
    Fence,
    Glass,
    RedstoneLampLit,
    WoodDoor,

    // Portal
    PortalFrameRedObsidian,
    PortalCoreDarkCrimson,

    // Red-Black
    CrimsonHeart,
    CrimsonDiamond,
    OrangeClub,
    OrangeSpade,
    PurpleHeart,
    PurpleDiamond,
    PurpleClub,
    PurpleSpade,
    Mycelium,
    SmoothBasalt,
    BuddingAmethyst,
    AmethystCluster,
    CryingObsidianCrimson,
    CryingObsidianOrange,
    CryingObsidianViolet,
    RedBlackDeepslateBricks,
    PolishedBlackstoneBricks,
    NetherWartBlock,
}
