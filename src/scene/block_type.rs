// Catalog-stage registry: the official block identities, their visual
// families and the one official count every catalog check derives from.
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
    RedBlackDeepslateBricksCrimson,
    RedBlackDeepslateBricksOrange,
    RedBlackDeepslateBricksViolet,
    PolishedBlackstoneBricks,
    NetherWartBlock,
}

/// Number of official catalog blocks: the single source of truth every
/// registry and catalog check derives from.
pub const OFFICIAL_BLOCK_COUNT: usize = 39;

/// Visual family of an official block, in catalog order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlockFamily {
    OverworldTerrain,
    OverworldArchitecture,
    Portal,
    RedBlackBase,
    Crimson,
    Orange,
    Violet,
    StructuralRedBlack,
}

impl BlockFamily {
    /// Human-readable family name for the catalog label.
    pub fn label(self) -> &'static str {
        match self {
            BlockFamily::OverworldTerrain => "Overworld Terrain",
            BlockFamily::OverworldArchitecture => "Overworld Architecture",
            BlockFamily::Portal => "Portal",
            BlockFamily::RedBlackBase => "Red-Black Base",
            BlockFamily::Crimson => "Crimson",
            BlockFamily::Orange => "Orange",
            BlockFamily::Violet => "Purple/Violet",
            BlockFamily::StructuralRedBlack => "Structural Red-Black",
        }
    }
}

impl BlockType {
    /// Every official block exactly once, grouped by family in catalog
    /// order. The array length ties it to `OFFICIAL_BLOCK_COUNT`, and
    /// `family` is an exhaustive match, so a new variant cannot be added
    /// without classifying it and extending this registry.
    pub const ALL: [BlockType; OFFICIAL_BLOCK_COUNT] = [
        // Overworld Terrain
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Cobblestone,
        BlockType::Sand,
        BlockType::Water,
        BlockType::Deepslate,
        BlockType::Log,
        BlockType::Leaves,
        // Overworld Architecture
        BlockType::DeepslateBricks,
        BlockType::WoodPlanks,
        BlockType::DoubleWoodSlab,
        BlockType::WoodStairs,
        BlockType::Fence,
        BlockType::Glass,
        BlockType::RedstoneLampLit,
        BlockType::WoodDoor,
        // Portal
        BlockType::PortalFrameRedObsidian,
        BlockType::PortalCoreDarkCrimson,
        // Red-Black Base
        BlockType::Mycelium,
        BlockType::SmoothBasalt,
        BlockType::BuddingAmethyst,
        BlockType::AmethystCluster,
        // Crimson
        BlockType::CrimsonHeart,
        BlockType::CrimsonDiamond,
        BlockType::CryingObsidianCrimson,
        BlockType::NetherWartBlock,
        // Orange
        BlockType::OrangeClub,
        BlockType::OrangeSpade,
        BlockType::CryingObsidianOrange,
        // Purple/Violet
        BlockType::PurpleHeart,
        BlockType::PurpleDiamond,
        BlockType::PurpleClub,
        BlockType::PurpleSpade,
        BlockType::CryingObsidianViolet,
        // Structural Red-Black
        BlockType::RedBlackDeepslateBricksCrimson,
        BlockType::RedBlackDeepslateBricksOrange,
        BlockType::RedBlackDeepslateBricksViolet,
        BlockType::PolishedBlackstoneBricks,
    ];

    /// The visual family this block belongs to.
    pub fn family(self) -> BlockFamily {
        match self {
            BlockType::Grass
            | BlockType::Dirt
            | BlockType::Stone
            | BlockType::Cobblestone
            | BlockType::Sand
            | BlockType::Water
            | BlockType::Deepslate
            | BlockType::Log
            | BlockType::Leaves => BlockFamily::OverworldTerrain,
            BlockType::DeepslateBricks
            | BlockType::WoodPlanks
            | BlockType::DoubleWoodSlab
            | BlockType::WoodStairs
            | BlockType::Fence
            | BlockType::Glass
            | BlockType::RedstoneLampLit
            | BlockType::WoodDoor => BlockFamily::OverworldArchitecture,
            BlockType::PortalFrameRedObsidian | BlockType::PortalCoreDarkCrimson => {
                BlockFamily::Portal
            }
            BlockType::Mycelium
            | BlockType::SmoothBasalt
            | BlockType::BuddingAmethyst
            | BlockType::AmethystCluster => BlockFamily::RedBlackBase,
            BlockType::CrimsonHeart
            | BlockType::CrimsonDiamond
            | BlockType::CryingObsidianCrimson
            | BlockType::NetherWartBlock => BlockFamily::Crimson,
            BlockType::OrangeClub | BlockType::OrangeSpade | BlockType::CryingObsidianOrange => {
                BlockFamily::Orange
            }
            BlockType::PurpleHeart
            | BlockType::PurpleDiamond
            | BlockType::PurpleClub
            | BlockType::PurpleSpade
            | BlockType::CryingObsidianViolet => BlockFamily::Violet,
            BlockType::RedBlackDeepslateBricksCrimson
            | BlockType::RedBlackDeepslateBricksOrange
            | BlockType::RedBlackDeepslateBricksViolet
            | BlockType::PolishedBlackstoneBricks => BlockFamily::StructuralRedBlack,
        }
    }

    /// Position of this block in the official registry order.
    pub fn catalog_rank(self) -> usize {
        BlockType::ALL
            .iter()
            .position(|t| *t == self)
            .expect("every BlockType is in BlockType::ALL")
    }
}
