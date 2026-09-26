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

use core::color::Color;
use core::hit::Face;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, nearest_visible_hit, nearest_voxel_hit};
use scene::block::BlockInstance;
use scene::block_geometry::{BlockGeometry, GeometryKind};
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{wood_door_bottom_material_id, wood_door_top_material_id};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const RANGE: f32 = 50.0;
const STONE: u32 = 230;

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn env() -> Env {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let mut library = catalog_materials(&textures);
    library.insert(
        MaterialId::new(STONE),
        Material::matte(Color::new(0.5, 0.5, 0.5, 1.0)),
    );
    Env { manager, library }
}

fn door_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::WoodDoor,
            wood_door_bottom_material_id(),
            Orientation::South,
        ),
    );
    w.insert(
        IVec3::new(0, 1, 0),
        BlockInstance::new(
            BlockType::WoodDoor,
            wood_door_top_material_id(),
            Orientation::South,
        ),
    );
    w.insert(
        IVec3::new(0, 1, -3),
        BlockInstance::new(BlockType::Stone, MaterialId::new(STONE), Orientation::Up),
    );
    w
}

fn ray(o: (f32, f32, f32), d: (f32, f32, f32)) -> Ray {
    Ray::new(Vec3::new(o.0, o.1, o.2), Vec3::new(d.0, d.1, d.2))
}

#[test]
fn the_door_is_a_thin_prism_never_a_full_cube() {
    let g = block_geometry(BlockType::WoodDoor, Orientation::South);
    assert!(!matches!(g, BlockGeometry::FullCube));
    assert_eq!(g.kind(), GeometryKind::Prism);
    let p = g.parts()[0];
    assert!(
        p.size().z < 0.25 && p.size().x > 0.99 && p.size().y > 0.99,
        "{:?}",
        p.size()
    );
}

#[test]
fn both_halves_and_their_edges_resolve() {
    let e = env();
    for id in [wood_door_bottom_material_id(), wood_door_top_material_id()] {
        let m = e.library.get(id).unwrap();
        let f = m.face_textures.unwrap();
        for face in [
            Face::PositiveX,
            Face::NegativeX,
            Face::PositiveY,
            Face::NegativeY,
            Face::PositiveZ,
            Face::NegativeZ,
        ] {
            let t = e.manager.get(f.texture_for_face(face)).expect("resolves");
            assert_eq!((t.width(), t.height()), (16, 16));
        }
        assert_ne!(
            f.texture_for_face(Face::PositiveZ),
            f.texture_for_face(Face::PositiveX),
            "broad != edge"
        );
        assert_eq!(
            f.texture_for_face(Face::PositiveZ),
            f.texture_for_face(Face::NegativeZ)
        );
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.emission_strength, 0.0);
        assert!((m.specular - 0.10).abs() < 1e-6);
    }
    assert_ne!(wood_door_bottom_material_id(), wood_door_top_material_id());
}

#[test]
fn the_reference_windows_are_real_holes_in_the_upper_half_only() {
    let e = env();
    let count_holes = |id| {
        let m = e.library.get(id).unwrap();
        assert_eq!(m.alpha_mode, AlphaMode::Cutout);
        let t = e
            .manager
            .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
            .unwrap();
        t.pixels().iter().filter(|c| c.a < 0.5).count()
    };
    assert_eq!(
        count_holes(wood_door_bottom_material_id()),
        0,
        "solid panels below"
    );
    assert_eq!(
        count_holes(wood_door_top_material_id()),
        48,
        "four window panes above"
    );
}

#[test]
fn a_ray_hits_the_leaf_at_the_cell_edge() {
    let e = env();
    let world = door_world();
    // Lower half, on a solid panel: the leaf face at z = 1.
    let r = ray((0.5, 0.3, 5.0), (0.0, 0.0, -1.0));
    let h = nearest_voxel_hit(&world, &r, RANGE).unwrap();
    assert_eq!(h.cell, IVec3::new(0, 0, 0));
    assert_eq!(h.hit.face, Face::PositiveZ);
    assert!((h.hit.point.z - 1.0).abs() < 1e-4);
    let s = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: r.origin,
        lights: &[],
        ambient_factor: 1.0,
        background: Color::black(),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };
    assert_eq!(
        nearest_visible_hit(&s, &r, RANGE).unwrap().cell,
        IVec3::new(0, 0, 0)
    );
}

#[test]
fn the_rest_of_the_cell_is_air_so_there_is_no_false_full_cube_hit() {
    let world = door_world();
    // Passing through the cell in front of the leaf, sideways.
    for y in [0.3_f32, 1.4] {
        let r = ray((-3.0, y, 0.3), (1.0, 0.0, 0.0));
        assert!(nearest_voxel_hit(&world, &r, RANGE).is_none(), "y = {y}");
    }
    // A ray that skims the leaf's thickness range from the side does hit it.
    let r = ray((-3.0, 0.3, 0.9), (1.0, 0.0, 0.0));
    assert!(nearest_voxel_hit(&world, &r, RANGE).is_some());
}

#[test]
fn a_ray_through_a_window_pane_reaches_the_block_behind() {
    let e = env();
    let world = door_world();
    let s = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Color::black(),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };

    // Find a window texel (alpha 0) of the top half and aim through it.
    let m = e.library.get(wood_door_top_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    let (i, j) = (0..256)
        .map(|n| (n % 16, n / 16))
        .find(|&(i, j)| t.texel(i, j).unwrap().a < 0.5)
        .expect("a window exists");
    let x = (i as f32 + 0.5) / 16.0;
    let y = 1.0 + 1.0 - (j as f32 + 0.5) / 16.0;
    let r = ray((x, y, 5.0), (0.0, 0.0, -1.0));

    let hit = nearest_visible_hit(&s, &r, RANGE).expect("reaches the stone behind");
    assert_eq!(hit.cell, IVec3::new(0, 1, -3));
    // ...while a solid texel of the same half is the door itself.
    let solid = ray((0.5, 1.55, 5.0), (0.0, 0.0, -1.0));
    assert_eq!(
        nearest_visible_hit(&s, &solid, RANGE).unwrap().cell,
        IVec3::new(0, 1, 0)
    );
}

#[test]
fn the_front_and_back_are_both_the_door_and_orientation_moves_the_leaf() {
    let world = door_world();
    let front = nearest_voxel_hit(&world, &ray((0.5, 0.3, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    let back = nearest_voxel_hit(&world, &ray((0.5, 0.3, -1.0), (0.0, 0.0, 1.0)), RANGE).unwrap();
    assert_eq!(front.hit.face, Face::PositiveZ);
    assert_eq!(back.hit.face, Face::NegativeZ);
    assert!((back.hit.point.z - 0.8125).abs() < 1e-4);

    let mut north = VoxelWorld::new();
    north.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(
            BlockType::WoodDoor,
            wood_door_bottom_material_id(),
            Orientation::North,
        ),
    );
    let h = nearest_voxel_hit(&north, &ray((0.5, 0.3, 5.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert!(
        (h.hit.point.z - 0.1875).abs() < 1e-4,
        "North puts the leaf on the -Z edge"
    );
}

#[test]
fn the_door_is_one_official_entry_with_both_cells() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::WoodDoor)
        .collect();
    assert_eq!(entries.len(), 1, "no second door");
    let d = &entries[0];
    assert_eq!(d.display_name, "WoodDoor");
    assert_eq!(d.kind, SampleKind::PartialGeometry);
    assert_eq!(d.material_id, wood_door_bottom_material_id());
    assert_eq!(d.extra_blocks.len(), 1);
    assert_eq!(
        d.extra_blocks[0].1.material_id(),
        wood_door_top_material_id()
    );

    let scene = CatalogScene::new();
    assert_eq!(scene.world().get(d.position), Some(&d.block()));
    assert_eq!(
        scene.world().get(d.extra_blocks[0].0),
        Some(&d.extra_blocks[0].1)
    );
    let p = d.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}
