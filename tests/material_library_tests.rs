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
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
}

use core::color::Color;
use core::material::{Material, MaterialId};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;

fn grey(v: f32) -> Color {
    Color::new(v, v, v, 1.0)
}

#[test]
fn a_material_can_be_inserted_and_queried_by_id() {
    let mut library = MaterialLibrary::new();
    assert!(library.is_empty());

    let id = MaterialId::new(1);
    assert!(library.insert(id, Material::matte(grey(0.5))).is_none());

    assert_eq!(library.len(), 1);
    assert!(library.contains(id));
    assert_eq!(library.get(id).unwrap().albedo, grey(0.5));
    assert!(library.get(MaterialId::new(2)).is_none());
}

#[test]
fn distinct_ids_resolve_to_distinct_materials() {
    let mut library = MaterialLibrary::new();
    library.insert(MaterialId::new(1), Material::matte(grey(0.2)));
    library.insert(MaterialId::new(2), Material::matte(grey(0.8)));

    assert_eq!(library.get(MaterialId::new(1)).unwrap().albedo, grey(0.2));
    assert_eq!(library.get(MaterialId::new(2)).unwrap().albedo, grey(0.8));
}

#[test]
fn reinserting_an_id_replaces_and_returns_the_previous_material() {
    let mut library = MaterialLibrary::new();
    let id = MaterialId::new(7);
    library.insert(id, Material::matte(grey(0.1)));
    let previous = library.insert(id, Material::matte(grey(0.9)));

    assert_eq!(previous.unwrap().albedo, grey(0.1));
    assert_eq!(library.len(), 1);
    assert_eq!(library.get(id).unwrap().albedo, grey(0.9));
}

#[test]
fn two_blocks_can_share_one_material_id() {
    let id = MaterialId::new(3);
    let a = BlockInstance::new(BlockType::Stone, id, Orientation::Up);
    let b = BlockInstance::new(BlockType::Cobblestone, id, Orientation::Down);
    assert_eq!(a.material_id(), b.material_id());

    let mut library = MaterialLibrary::new();
    library.insert(id, Material::matte(grey(0.4)));
    assert_eq!(
        library.get(a.material_id()).unwrap(),
        library.get(b.material_id()).unwrap()
    );
    assert_eq!(library.len(), 1);
}

#[test]
fn reflectivity_is_never_negative_and_never_exceeds_one() {
    let m = Material::matte(grey(0.5));
    assert_eq!(m.clone().with_reflectivity(-3.0).reflectivity, 0.0);
    assert_eq!(m.clone().with_reflectivity(0.25).reflectivity, 0.25);
    assert_eq!(m.clone().with_reflectivity(9.0).reflectivity, 1.0);
    assert_eq!(m.with_reflectivity(f32::NAN).reflectivity, 0.0);
}

#[test]
fn transparency_stays_within_zero_and_one() {
    let m = Material::matte(grey(0.5));
    assert_eq!(m.clone().with_transparency(-0.5).transparency, 0.0);
    assert_eq!(m.clone().with_transparency(0.92).transparency, 0.92);
    assert_eq!(m.clone().with_transparency(2.0).transparency, 1.0);
    assert_eq!(m.with_transparency(f32::INFINITY).transparency, 0.0);
}

#[test]
fn refractive_index_is_always_positive_and_finite() {
    let m = Material::matte(grey(0.5));
    assert_eq!(m.clone().with_refractive_index(1.5).refractive_index, 1.5);
    assert_eq!(m.clone().with_refractive_index(0.0).refractive_index, 1.0);
    assert_eq!(m.clone().with_refractive_index(-1.3).refractive_index, 1.0);
    assert_eq!(m.with_refractive_index(f32::NAN).refractive_index, 1.0);
}

#[test]
fn emission_strength_is_never_negative() {
    let m = Material::matte(grey(0.5));
    assert_eq!(
        m.clone().with_emission_strength(-2.0).emission_strength,
        0.0
    );
    assert_eq!(m.clone().with_emission_strength(2.5).emission_strength, 2.5);
    assert_eq!(m.with_emission_strength(f32::NAN).emission_strength, 0.0);
}

#[test]
fn a_material_without_emissive_or_normal_texture_is_valid() {
    let m = Material::new(grey(0.5), 0.1, 10.0);
    assert!(m.emissive_texture.is_none());
    assert!(m.normal_texture.is_none());
    assert!(m.face_textures.is_none());
}

#[test]
fn legacy_materials_remain_opaque_and_non_reflective() {
    for m in [
        Material::matte(grey(0.5)),
        Material::glossy(grey(0.5)),
        Material::new(grey(0.5), 0.2, 30.0),
    ] {
        assert_eq!(m.reflectivity, 0.0);
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.refractive_index, 1.0);
        assert_eq!(m.emission_strength, 0.0);
    }
}
