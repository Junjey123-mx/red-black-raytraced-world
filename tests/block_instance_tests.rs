#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

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
}

use core::material::{Material, MaterialId};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::orientation::Orientation;

#[test]
fn material_id_preserves_its_value_and_compares_by_value() {
    let a = MaterialId::new(7);
    let b = MaterialId::new(7);
    let c = MaterialId::new(8);

    assert_eq!(a.value(), 7);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn construction_preserves_every_field() {
    let block = BlockInstance::new(BlockType::Grass, MaterialId::new(1), Orientation::Up);

    assert_eq!(block.block_type(), BlockType::Grass);
    assert_eq!(block.material_id(), MaterialId::new(1));
    assert_eq!(block.orientation(), Orientation::Up);
}

#[test]
fn block_type_is_preserved() {
    let block = BlockInstance::new(BlockType::WoodStairs, MaterialId::new(3), Orientation::East);
    assert_eq!(block.block_type(), BlockType::WoodStairs);
}

#[test]
fn material_id_is_preserved() {
    let block = BlockInstance::new(BlockType::Stone, MaterialId::new(42), Orientation::Up);
    assert_eq!(block.material_id().value(), 42);
}

#[test]
fn orientation_is_preserved() {
    let block = BlockInstance::new(
        BlockType::CrimsonHeart,
        MaterialId::new(9),
        Orientation::Down,
    );
    assert_eq!(block.orientation(), Orientation::Down);
}

#[test]
fn two_instances_can_share_the_same_material_id() {
    let shared = MaterialId::new(5);
    let a = BlockInstance::new(BlockType::Dirt, shared, Orientation::Up);
    let b = BlockInstance::new(BlockType::Grass, shared, Orientation::Up);

    assert_eq!(a.material_id(), b.material_id());
    assert_ne!(a.block_type(), b.block_type());
    assert_ne!(a, b);
}

#[test]
fn different_blocks_can_have_different_orientations() {
    let up = BlockInstance::new(BlockType::WoodStairs, MaterialId::new(2), Orientation::Up);
    let down = BlockInstance::new(BlockType::WoodStairs, MaterialId::new(2), Orientation::Down);
    let north = BlockInstance::new(
        BlockType::WoodStairs,
        MaterialId::new(2),
        Orientation::North,
    );

    assert_ne!(up.orientation(), down.orientation());
    assert_ne!(up.orientation(), north.orientation());
    assert_ne!(up, down);
}

#[test]
fn identical_fields_produce_equal_instances() {
    let a = BlockInstance::new(BlockType::Sand, MaterialId::new(4), Orientation::Up);
    let b = BlockInstance::new(BlockType::Sand, MaterialId::new(4), Orientation::Up);
    assert_eq!(a, b);
}

#[test]
fn block_instance_does_not_embed_a_material() {
    // A full Material carries an albedo Color, specular, shininess and an
    // optional FaceTextures; the instance must be far smaller than that,
    // proving it stores only the MaterialId reference.
    assert!(std::mem::size_of::<BlockInstance>() <= 8);
    assert!(std::mem::size_of::<BlockInstance>() < std::mem::size_of::<Material>());
}

#[test]
fn block_instance_is_copy() {
    let a = BlockInstance::new(BlockType::Log, MaterialId::new(6), Orientation::North);
    let b = a;
    assert_eq!(a, b);
}
