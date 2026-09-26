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

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_gallery.rs"]
    pub mod material_gallery;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::hit::Face;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::nearest_voxel_hit;
use scene::block::BlockInstance;
use scene::block_geometry::{BlockGeometry, GeometryKind};
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::orientation::Orientation;
use scene::overworld_blocks::{wood_planks_material_id, wood_stairs_material_id};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const RANGE: f32 = 50.0;

fn env() -> (TextureManager, scene::material_library::MaterialLibrary) {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let library = catalog_materials(&textures);
    (manager, library)
}

fn stairs_world(orientation: Orientation) -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::WoodStairs,
            wood_stairs_material_id(),
            orientation,
        ),
    );
    w
}

fn ray(o: (f32, f32, f32), d: (f32, f32, f32)) -> Ray {
    Ray::new(Vec3::new(o.0, o.1, o.2), Vec3::new(d.0, d.1, d.2))
}

#[test]
fn stairs_are_a_composite_of_two_prisms_never_a_full_cube() {
    let g = block_geometry(BlockType::WoodStairs, Orientation::South);
    assert!(!matches!(g, BlockGeometry::FullCube));
    assert_eq!(g.kind(), GeometryKind::Composite);
    assert_eq!(g.part_count(), 2, "tread block + riser block");
}

#[test]
fn the_material_uses_the_final_planks_texture_on_every_face() {
    let (manager, library) = env();
    let m = library.get(wood_stairs_material_id()).unwrap();
    let planks = library.get(wood_planks_material_id()).unwrap();
    let f = m.face_textures.unwrap();

    for face in [
        Face::PositiveX,
        Face::NegativeX,
        Face::PositiveY,
        Face::NegativeY,
        Face::PositiveZ,
        Face::NegativeZ,
    ] {
        assert!(manager.get(f.texture_for_face(face)).is_some());
        assert_eq!(
            f.texture_for_face(face),
            planks.face_textures.unwrap().texture_for_face(face)
        );
    }
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
}

#[test]
fn both_steps_are_hit_and_the_notch_is_air() {
    let world = stairs_world(Orientation::South);
    // South: the tall half is at the back (z < 0.5), the low step in front.
    let low = ray((0.5, 5.0, 0.75), (0.0, -1.0, 0.0));
    let h = nearest_voxel_hit(&world, &low, RANGE).unwrap();
    assert_eq!(h.hit.face, Face::PositiveY);
    assert!(
        (h.hit.point.y - 0.5).abs() < 1e-4,
        "low step top at half height"
    );

    let high = ray((0.5, 5.0, 0.25), (0.0, -1.0, 0.0));
    let h = nearest_voxel_hit(&world, &high, RANGE).unwrap();
    assert!(
        (h.hit.point.y - 1.0).abs() < 1e-4,
        "high step top at full height"
    );

    // Above the low step, in front of the riser: nothing there.
    let notch = ray((0.5, 0.75, 5.0), (0.0, 0.0, -1.0));
    let h = nearest_voxel_hit(&world, &notch, RANGE).unwrap();
    assert_eq!(h.hit.face, Face::PositiveZ);
    assert!(
        (h.hit.point.z - 0.5).abs() < 1e-4,
        "hits the riser, not a full-cube front"
    );
}

#[test]
fn the_orientation_rotates_the_steps() {
    let hit_z = |o: Orientation| {
        let r = ray((0.5, 0.75, 5.0), (0.0, 0.0, -1.0));
        nearest_voxel_hit(&stairs_world(o), &r, RANGE)
            .unwrap()
            .hit
            .point
            .z
    };
    let south = hit_z(Orientation::South);
    let north = hit_z(Orientation::North);
    assert!(
        (south - 0.5).abs() < 1e-4 && (north - 1.0).abs() < 1e-4,
        "{south} vs {north}"
    );
}

#[test]
fn the_texture_continues_across_tread_and_riser_using_cell_local_uv() {
    let world = stairs_world(Orientation::South);
    // A hit on the low riser (front face, y in 0..0.5) reads the lower part of
    // the cell's texture; the upper riser continues from the upper part.
    let low = nearest_voxel_hit(&world, &ray((0.5, 0.25, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    let high = nearest_voxel_hit(&world, &ray((0.5, 0.75, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();

    assert!(
        (low.hit.uv.y - 0.75).abs() < 1e-3,
        "low riser uv.y = {}",
        low.hit.uv.y
    );
    assert!(
        (high.hit.uv.y - 0.25).abs() < 1e-3,
        "high riser uv.y = {}",
        high.hit.uv.y
    );
    assert!((low.hit.uv.x - 0.5).abs() < 1e-3);
}

#[test]
fn stairs_are_one_official_catalog_entry_with_a_finite_focus_point() {
    let scene = CatalogScene::new();
    let entries: Vec<_> = scene
        .entries()
        .iter()
        .filter(|e| e.block_type == BlockType::WoodStairs)
        .collect();
    assert_eq!(entries.len(), 1, "no second stairs sample");
    let e = entries[0];
    assert_eq!(e.display_name, "Wood stairs");
    assert_eq!(e.kind, SampleKind::PartialGeometry);
    assert_eq!(e.material_id, wood_stairs_material_id());
    assert_eq!(scene.world().get(e.position), Some(&e.block()));
    let p = e.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    assert!(
        catalog_entries()
            .iter()
            .filter(|x| x.display_name.contains("stairs"))
            .count()
            == 1
    );
}
