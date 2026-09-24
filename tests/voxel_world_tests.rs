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

fn grass() -> BlockInstance {
    BlockInstance::new(BlockType::Grass, MaterialId::new(1), Orientation::Up)
}

fn dirt() -> BlockInstance {
    BlockInstance::new(BlockType::Dirt, MaterialId::new(2), Orientation::Up)
}

fn stone() -> BlockInstance {
    BlockInstance::new(BlockType::Stone, MaterialId::new(3), Orientation::Down)
}

#[test]
fn new_world_is_empty() {
    let world = VoxelWorld::new();
    assert!(world.is_empty());
    assert_eq!(world.len(), 0);
}

#[test]
fn insert_occupies_an_empty_cell() {
    let mut world = VoxelWorld::new();
    let previous = world.insert(IVec3::new(0, 0, 0), grass());

    assert!(previous.is_none());
    assert_eq!(world.len(), 1);
    assert!(!world.is_empty());
}

#[test]
fn get_returns_the_stored_instance() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(1, 2, 3), grass());

    assert_eq!(world.get(IVec3::new(1, 2, 3)), Some(&grass()));
    assert!(world.contains(IVec3::new(1, 2, 3)));
}

#[test]
fn get_on_an_empty_cell_is_none() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), grass());

    assert_eq!(world.get(IVec3::new(5, 5, 5)), None);
    assert!(!world.contains(IVec3::new(5, 5, 5)));
}

#[test]
fn insert_into_an_occupied_cell_replaces_and_returns_the_previous_block() {
    let mut world = VoxelWorld::new();
    let position = IVec3::new(4, 0, 4);
    world.insert(position, dirt());

    let previous = world.insert(position, stone());

    assert_eq!(previous, Some(dirt()));
    assert_eq!(world.get(position), Some(&stone()));
    assert_eq!(world.len(), 1);
}

#[test]
fn remove_empties_the_cell_and_returns_the_block() {
    let mut world = VoxelWorld::new();
    let position = IVec3::new(2, 2, 2);
    world.insert(position, grass());

    let removed = world.remove(position);

    assert_eq!(removed, Some(grass()));
    assert!(!world.contains(position));
    assert_eq!(world.len(), 0);
}

#[test]
fn remove_on_a_missing_cell_is_none_and_changes_nothing() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), grass());

    assert_eq!(world.remove(IVec3::new(9, 9, 9)), None);
    assert_eq!(world.len(), 1);
}

#[test]
fn multiple_coordinates_are_stored_independently() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), grass());
    world.insert(IVec3::new(1, 0, 0), dirt());
    world.insert(IVec3::new(0, 1, 0), stone());

    assert_eq!(world.len(), 3);
    assert_eq!(world.get(IVec3::new(0, 0, 0)), Some(&grass()));
    assert_eq!(world.get(IVec3::new(1, 0, 0)), Some(&dirt()));
    assert_eq!(world.get(IVec3::new(0, 1, 0)), Some(&stone()));
}

#[test]
fn negative_coordinates_are_valid_cells() {
    let mut world = VoxelWorld::new();
    let positions = [
        IVec3::new(-1, 0, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, -1),
        IVec3::new(-4, 2, 7),
    ];

    for (i, position) in positions.iter().enumerate() {
        let block =
            BlockInstance::new(BlockType::Stone, MaterialId::new(i as u32), Orientation::Up);
        world.insert(*position, block);
    }

    assert_eq!(world.len(), 4);
    for (i, position) in positions.iter().enumerate() {
        let stored = world.get(*position).expect("negative cell must exist");
        assert_eq!(stored.material_id(), MaterialId::new(i as u32));
    }
    // Neighboring positive cells were never touched.
    assert!(!world.contains(IVec3::new(1, 0, 0)));
    assert!(!world.contains(IVec3::new(0, 1, 0)));
    assert!(!world.contains(IVec3::new(0, 0, 1)));
}

#[test]
fn blocks_with_different_materials_and_orientations_coexist() {
    let mut world = VoxelWorld::new();
    let a = BlockInstance::new(BlockType::WoodStairs, MaterialId::new(10), Orientation::Up);
    let b = BlockInstance::new(
        BlockType::WoodStairs,
        MaterialId::new(11),
        Orientation::Down,
    );
    world.insert(IVec3::new(0, 0, 0), a);
    world.insert(IVec3::new(1, 0, 0), b);

    let stored_a = world.get(IVec3::new(0, 0, 0)).unwrap();
    let stored_b = world.get(IVec3::new(1, 0, 0)).unwrap();
    assert_eq!(stored_a.material_id(), MaterialId::new(10));
    assert_eq!(stored_b.material_id(), MaterialId::new(11));
    assert_eq!(stored_a.orientation(), Orientation::Up);
    assert_eq!(stored_b.orientation(), Orientation::Down);
}

#[test]
fn len_is_correct_after_inserts_replacements_and_removals() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), grass());
    world.insert(IVec3::new(1, 0, 0), dirt());
    assert_eq!(world.len(), 2);

    world.insert(IVec3::new(0, 0, 0), stone()); // replacement
    assert_eq!(world.len(), 2);

    world.remove(IVec3::new(1, 0, 0));
    assert_eq!(world.len(), 1);

    world.remove(IVec3::new(1, 0, 0)); // already gone
    assert_eq!(world.len(), 1);
}

#[test]
fn storage_is_sparse_and_far_cells_create_no_intermediate_cells() {
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), grass());
    world.insert(IVec3::new(1000, -500, 200), dirt());

    assert_eq!(world.len(), 2);
    assert!(!world.contains(IVec3::new(500, -250, 100)));
    assert!(!world.contains(IVec3::new(1, 0, 0)));
    assert_eq!(world.get(IVec3::new(1000, -500, 200)), Some(&dirt()));
}
