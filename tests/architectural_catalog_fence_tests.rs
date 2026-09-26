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
use scene::catalog::{CatalogScene, CatalogTextures, SampleKind, catalog_materials};
use scene::orientation::Orientation;
use scene::overworld_blocks::{fence_material_id, wood_planks_material_id};
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

fn fence_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Fence, fence_material_id(), Orientation::South),
    );
    // A block behind the fence so a ray through a gap has something to reach.
    w.insert(
        IVec3::new(0, 0, -3),
        BlockInstance::new(
            BlockType::Stone,
            core::material::MaterialId::new(2),
            Orientation::Up,
        ),
    );
    w
}

fn ray(o: (f32, f32, f32), d: (f32, f32, f32)) -> Ray {
    Ray::new(Vec3::new(o.0, o.1, o.2), Vec3::new(d.0, d.1, d.2))
}

#[test]
fn the_fence_is_a_composite_of_post_and_rails_not_a_full_cube() {
    let g = block_geometry(BlockType::Fence, Orientation::South);
    assert!(!matches!(g, BlockGeometry::FullCube));
    assert_eq!(g.kind(), GeometryKind::Composite);
    assert_eq!(g.part_count(), 3, "one post and two rails");
}

#[test]
fn texture_and_material_resolve_and_share_the_planks_family() {
    let (manager, library) = env();
    let m = library.get(fence_material_id()).unwrap();
    let f = m.face_textures.unwrap();
    let planks = library
        .get(wood_planks_material_id())
        .unwrap()
        .face_textures
        .unwrap();

    for face in [
        Face::PositiveX,
        Face::PositiveY,
        Face::NegativeY,
        Face::PositiveZ,
        Face::NegativeZ,
    ] {
        assert!(manager.get(f.texture_for_face(face)).is_some());
        assert_eq!(f.texture_for_face(face), planks.texture_for_face(face));
    }
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
}

#[test]
fn a_ray_hits_the_post() {
    let world = fence_world();
    // Below the rails, dead center: only the post is there.
    let h = nearest_voxel_hit(&world, &ray((0.5, 0.15, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(h.cell, IVec3::new(0, 0, 0));
    assert!(
        (h.hit.point.z - 0.625).abs() < 1e-4,
        "front of the 0.25-wide post"
    );
}

#[test]
fn a_ray_hits_a_rail() {
    let world = fence_world();
    // Off-center along x, at rail height: the rail (thin in z) is struck.
    let h = nearest_voxel_hit(&world, &ray((0.1, 0.47, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(h.cell, IVec3::new(0, 0, 0));
    assert!((h.hit.point.z - 0.5625).abs() < 1e-4, "front of the rail");
}

#[test]
fn a_ray_through_the_gap_passes_the_fence_cell_and_continues() {
    let world = fence_world();
    // Between the two rails (y ~ 0.65) and beside the post.
    let h = nearest_voxel_hit(&world, &ray((0.15, 0.65, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(h.cell, IVec3::new(0, 0, -3), "the gap is real air");

    // Above the upper rail too.
    let h = nearest_voxel_hit(&world, &ray((0.15, 0.97, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(h.cell, IVec3::new(0, 0, -3));
}

#[test]
fn the_post_uses_a_cropped_slice_of_the_planks_not_a_squashed_copy() {
    let world = fence_world();
    let h = nearest_voxel_hit(&world, &ray((0.5, 0.15, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    // Cell-local UV: the post's front face at x = 0.5, y = 0.15.
    assert!(
        (h.hit.uv.x - 0.5).abs() < 1e-3 && (h.hit.uv.y - 0.85).abs() < 1e-3,
        "{:?}",
        h.hit.uv
    );
}

#[test]
fn the_fence_is_one_official_catalog_entry() {
    let scene = CatalogScene::new();
    let fences: Vec<_> = scene
        .entries()
        .iter()
        .filter(|e| e.block_type == BlockType::Fence)
        .collect();
    assert_eq!(fences.len(), 1);
    let e = fences[0];
    assert_eq!(e.display_name, "Fence");
    assert_eq!(e.kind, SampleKind::PartialGeometry);
    assert_eq!(e.material_id, fence_material_id());
    assert_eq!(scene.world().get(e.position), Some(&e.block()));
    let p = e.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}
