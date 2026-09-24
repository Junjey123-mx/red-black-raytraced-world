#[path = "../src/scene/block_type.rs"]
mod block_type;

use block_type::BlockType;
use std::collections::{HashMap, HashSet};

#[test]
fn same_variant_is_equal() {
    assert_eq!(BlockType::Grass, BlockType::Grass);
    assert_eq!(BlockType::CrimsonHeart, BlockType::CrimsonHeart);
}

#[test]
fn different_variants_are_not_equal() {
    assert_ne!(BlockType::Grass, BlockType::Dirt);
    assert_ne!(BlockType::CrimsonHeart, BlockType::PurpleHeart);
    assert_ne!(BlockType::Grass, BlockType::Mycelium);
}

#[test]
fn block_type_is_copy_and_clone() {
    let original = BlockType::Stone;
    let copied = original;
    let cloned = original.clone();

    // `original` is still usable after the copy: BlockType is Copy.
    assert_eq!(original, copied);
    assert_eq!(original, cloned);
}

#[test]
fn block_type_works_as_a_hash_map_key() {
    let mut map: HashMap<BlockType, u32> = HashMap::new();
    map.insert(BlockType::Grass, 1);
    map.insert(BlockType::Dirt, 2);
    map.insert(BlockType::Grass, 3);

    assert_eq!(map.len(), 2);
    assert_eq!(map[&BlockType::Grass], 3);
    assert_eq!(map[&BlockType::Dirt], 2);
    assert!(!map.contains_key(&BlockType::Stone));
}

#[test]
fn block_type_works_in_a_hash_set() {
    let set: HashSet<BlockType> = [
        BlockType::Water,
        BlockType::Sand,
        BlockType::Water,
        BlockType::Sand,
    ]
    .into_iter()
    .collect();

    assert_eq!(set.len(), 2);
}

#[test]
fn catalog_contains_representative_blocks_of_all_three_families() {
    let overworld = [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Cobblestone,
        BlockType::Sand,
        BlockType::Water,
        BlockType::Deepslate,
        BlockType::DeepslateBricks,
        BlockType::Log,
        BlockType::Leaves,
        BlockType::WoodPlanks,
        BlockType::DoubleWoodSlab,
        BlockType::WoodStairs,
        BlockType::Fence,
        BlockType::Glass,
        BlockType::RedstoneLampLit,
        BlockType::WoodDoor,
    ];
    let portal = [
        BlockType::PortalFrameRedObsidian,
        BlockType::PortalCoreDarkCrimson,
    ];
    let red_black = [
        BlockType::CrimsonHeart,
        BlockType::CrimsonDiamond,
        BlockType::OrangeClub,
        BlockType::OrangeSpade,
        BlockType::PurpleHeart,
        BlockType::PurpleDiamond,
        BlockType::PurpleClub,
        BlockType::PurpleSpade,
        BlockType::Mycelium,
        BlockType::SmoothBasalt,
        BlockType::BuddingAmethyst,
        BlockType::AmethystCluster,
        BlockType::CryingObsidianCrimson,
        BlockType::CryingObsidianOrange,
        BlockType::CryingObsidianViolet,
        BlockType::RedBlackDeepslateBricks,
        BlockType::PolishedBlackstoneBricks,
        BlockType::NetherWartBlock,
    ];

    assert_eq!(overworld.len(), 17);
    assert_eq!(portal.len(), 2);
    assert_eq!(red_black.len(), 18);

    // The whole frozen catalog (37 nominal entries) is pairwise distinct.
    let all: HashSet<BlockType> = overworld
        .iter()
        .chain(portal.iter())
        .chain(red_black.iter())
        .copied()
        .collect();
    assert_eq!(all.len(), 37);
}

#[test]
fn block_type_is_only_a_lightweight_identifier() {
    // A fieldless enum occupies a single byte: it cannot embed a Material,
    // a CpuTexture, or any geometry.
    assert_eq!(std::mem::size_of::<BlockType>(), 1);
}

#[test]
fn block_type_is_deterministic_across_copies() {
    let a = BlockType::WoodStairs;
    let b = BlockType::WoodStairs;
    assert_eq!(a, b);
    assert_eq!(format!("{a:?}"), "WoodStairs");
}
