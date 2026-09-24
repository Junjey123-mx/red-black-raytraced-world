#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

use core::material::MaterialId;
use core::math::IVec3;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::voxel_world::VoxelWorld;

/// Non-comment source lines only, so structural audits look at real code
/// rather than at prose that merely names a concept it explicitly excludes.
fn code_of(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_token(code: &str, token: &str) -> bool {
    code.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == token)
}

// ---------------------------------------------------------------------
// Case A - heterogeneous world preserves type, material and orientation.
// ---------------------------------------------------------------------

#[test]
fn heterogeneous_world_preserves_block_type_material_and_orientation() {
    let mut world = VoxelWorld::new();
    world.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Grass, MaterialId::new(1), Orientation::Up),
    );
    world.insert(
        IVec3::new(1, 0, 0),
        BlockInstance::new(BlockType::Dirt, MaterialId::new(2), Orientation::Up),
    );
    world.insert(
        IVec3::new(2, 0, 0),
        BlockInstance::new(BlockType::Stone, MaterialId::new(2), Orientation::North),
    );

    let grass = world.get(IVec3::new(0, 0, 0)).unwrap();
    let dirt = world.get(IVec3::new(1, 0, 0)).unwrap();
    let stone = world.get(IVec3::new(2, 0, 0)).unwrap();

    assert_eq!(grass.block_type(), BlockType::Grass);
    assert_eq!(grass.material_id(), MaterialId::new(1));
    assert_eq!(grass.orientation(), Orientation::Up);

    assert_eq!(dirt.block_type(), BlockType::Dirt);
    assert_eq!(dirt.material_id(), MaterialId::new(2));
    assert_eq!(dirt.orientation(), Orientation::Up);

    // Dirt and Stone deliberately share MaterialId(2): sharing is allowed.
    assert_eq!(stone.block_type(), BlockType::Stone);
    assert_eq!(stone.material_id(), dirt.material_id());
    assert_eq!(stone.orientation(), Orientation::North);
}

// ---------------------------------------------------------------------
// Case B - a future inverted (Down) block is stored without reinterpretation.
// ---------------------------------------------------------------------

#[test]
fn down_orientation_is_preserved_without_reinterpretation() {
    let mut world = VoxelWorld::new();
    let inverted = BlockInstance::new(
        BlockType::CrimsonHeart,
        MaterialId::new(30),
        Orientation::Down,
    );
    world.insert(IVec3::new(0, -3, 0), inverted);

    let stored = world.get(IVec3::new(0, -3, 0)).unwrap();
    assert_eq!(stored.orientation(), Orientation::Down);
    assert_ne!(stored.orientation(), Orientation::Up);
    assert_eq!(*stored, inverted);
}

// ---------------------------------------------------------------------
// Case C - negative coordinates recover exactly.
// ---------------------------------------------------------------------

#[test]
fn negative_coordinate_block_is_recovered_exactly() {
    let mut world = VoxelWorld::new();
    let block = BlockInstance::new(BlockType::Sand, MaterialId::new(7), Orientation::East);
    world.insert(IVec3::new(-2, 3, -5), block);

    assert_eq!(world.get(IVec3::new(-2, 3, -5)), Some(&block));
    // Sign-mirrored neighbors are different cells.
    assert_eq!(world.get(IVec3::new(2, 3, -5)), None);
    assert_eq!(world.get(IVec3::new(-2, 3, 5)), None);
    assert_eq!(world.get(IVec3::new(-2, -3, -5)), None);
}

// ---------------------------------------------------------------------
// Case D - replacement (Dirt -> Stone).
// ---------------------------------------------------------------------

#[test]
fn replacing_dirt_with_stone_keeps_len_and_returns_the_old_block() {
    let mut world = VoxelWorld::new();
    let dirt = BlockInstance::new(BlockType::Dirt, MaterialId::new(2), Orientation::Up);
    let stone = BlockInstance::new(BlockType::Stone, MaterialId::new(3), Orientation::Up);
    let position = IVec3::new(5, 1, 5);

    world.insert(position, dirt);
    let before = world.len();
    let previous = world.insert(position, stone);

    assert_eq!(world.len(), before);
    assert_eq!(previous, Some(dirt));
    assert_eq!(world.get(position), Some(&stone));
}

// ---------------------------------------------------------------------
// Case E - sparse storage: len equals the number of real insertions.
// ---------------------------------------------------------------------

#[test]
fn len_equals_the_number_of_real_cells_for_widely_separated_positions() {
    let mut world = VoxelWorld::new();
    let positions = [
        IVec3::new(0, 0, 0),
        IVec3::new(1000, -500, 200),
        IVec3::new(-100_000, 40, 7),
        IVec3::new(3, 90_000, -90_000),
    ];
    for (i, position) in positions.iter().enumerate() {
        world.insert(
            *position,
            BlockInstance::new(BlockType::Stone, MaterialId::new(i as u32), Orientation::Up),
        );
    }

    assert_eq!(world.len(), positions.len());
    // Nothing exists between the extremes.
    assert!(!world.contains(IVec3::new(500, -250, 100)));
    assert!(!world.contains(IVec3::new(1, 0, 0)));
}

#[test]
fn a_full_lifecycle_across_all_layers_stays_consistent() {
    let mut world = VoxelWorld::new();
    let a = BlockInstance::new(BlockType::Grass, MaterialId::new(1), Orientation::Up);
    let b = BlockInstance::new(BlockType::WoodStairs, MaterialId::new(9), Orientation::Down);

    world.insert(IVec3::new(-1, -1, -1), a);
    world.insert(IVec3::new(1, 1, 1), b);
    assert_eq!(world.len(), 2);

    assert_eq!(world.remove(IVec3::new(-1, -1, -1)), Some(a));
    assert_eq!(world.len(), 1);
    assert_eq!(world.get(IVec3::new(1, 1, 1)), Some(&b));
    assert!(!world.contains(IVec3::new(-1, -1, -1)));
}

// ---------------------------------------------------------------------
// Structural audit: the storage layer stays free of rendering concerns.
// ---------------------------------------------------------------------

#[test]
fn block_type_carries_no_material_texture_or_geometry() {
    let code = code_of(include_str!("../src/scene/block_type.rs"));
    for forbidden in [
        "Material",
        "MaterialId",
        "CpuTexture",
        "Cube",
        "Aabb",
        "Vec3",
    ] {
        assert!(
            !has_token(&code, forbidden),
            "BlockType mentions {forbidden}"
        );
    }
}

#[test]
fn block_instance_has_no_position_and_no_heavy_members() {
    let code = code_of(include_str!("../src/scene/block.rs"));
    for forbidden in [
        "IVec3",
        "Vec3",
        "position",
        "Material",
        "CpuTexture",
        "Cube",
        "Aabb",
        "Light",
    ] {
        assert!(
            !has_token(&code, forbidden),
            "BlockInstance mentions {forbidden}"
        );
    }
}

#[test]
fn voxel_world_has_no_camera_framebuffer_raylib_or_dda() {
    let code = code_of(include_str!("../src/scene/voxel_world.rs")).to_lowercase();
    for forbidden in [
        "camera",
        "framebuffer",
        "raylib",
        "dda",
        "ray",
        "traversal",
        "cube",
    ] {
        assert!(
            !has_token(&code, forbidden),
            "VoxelWorld mentions {forbidden}"
        );
    }
}
