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
use core::material::{AlphaMode, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{
    VoxelScene, light_visibility, nearest_visible_hit, nearest_voxel_hit, trace_ray,
};
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_gallery::leaves_material_id;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const RANGE: f32 = 60.0;
const N: usize = 16;
const RED: u32 = 210;

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
    solid: Vec<Vec<bool>>,
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
        MaterialId::new(RED),
        core::material::Material::matte(Color::new(1.0, 0.0, 0.0, 1.0)),
    );
    let m = library.get(leaves_material_id()).unwrap();
    let t = manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    let solid = (0..N)
        .map(|y| (0..N).map(|x| t.texel(x, y).unwrap().a >= 0.5).collect())
        .collect();
    Env {
        manager,
        library,
        solid,
    }
}

impl Env {
    fn scene<'a>(&'a self, world: &'a VoxelWorld, ambient: f32) -> VoxelScene<'a> {
        VoxelScene {
            world,
            materials: &self.library,
            camera_position: Vec3::zero(),
            lights: &[],
            ambient_factor: ambient,
            background: Color::new(0.0, 0.0, 1.0, 1.0),
            texture_manager: &self.manager,
            max_distance: RANGE,
        }
    }
}

fn leaves_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Leaves, leaves_material_id(), Orientation::Up),
    );
    w
}

fn front(i: usize, j: usize) -> Ray {
    Ray::new(
        Vec3::new((i as f32 + 0.5) / 16.0, 1.0 - (j as f32 + 0.5) / 16.0, 9.0),
        Vec3::new(0.0, 0.0, -1.0),
    )
}

fn find(e: &Env, pred: impl Fn(&Env, usize, usize) -> bool) -> (usize, usize) {
    (0..N * N)
        .map(|n| (n % N, n / N))
        .find(|&(i, j)| pred(e, i, j))
        .expect("texel exists")
}

#[test]
fn leaves_is_an_official_full_cube_block_type() {
    assert!(matches!(
        block_geometry(BlockType::Leaves, Orientation::Up),
        BlockGeometry::FullCube
    ));
    let s = CatalogScene::new();
    let e = s
        .entries()
        .iter()
        .find(|e| e.block_type == BlockType::Leaves)
        .unwrap();
    // The geometric envelope is the whole cell: a ray at the corner is hit
    // by the raw traversal even though the texel may be empty.
    let w = leaves_world();
    let r = Ray::new(Vec3::new(0.99, 0.99, 9.0), Vec3::new(0.0, 0.0, -1.0));
    assert!(nearest_voxel_hit(&w, &r, RANGE).is_some());
    assert_eq!(e.display_name, "Leaves");
}

#[test]
fn the_material_is_cutout_not_blend_and_never_refracts() {
    let e = env();
    let m = e.library.get(leaves_material_id()).unwrap();

    assert_eq!(m.alpha_mode, AlphaMode::Cutout);
    assert_ne!(m.alpha_mode, AlphaMode::Blend);
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.refractive_index, 1.0);
    assert_eq!(m.reflectivity, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none() && m.normal_texture.is_none());
    assert!((m.specular - 0.03).abs() < 1e-6 && (m.shininess - 4.0).abs() < 1e-6);
    assert_eq!(
        m.effective_transparency(0.0),
        0.0,
        "cutout is never continuous transmission"
    );
}

#[test]
fn the_texture_has_solid_and_empty_texels_and_matches_the_reference_mask() {
    let e = env();
    let empty = e.solid.iter().flatten().filter(|s| !**s).count();
    // 84 of 256 texels are air in the supplied foliage reference.
    assert_eq!(empty, 84);
    let m = e.library.get(leaves_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    assert_eq!((t.width(), t.height()), (16, 16));
    assert!(
        t.pixels().iter().all(|c| c.a == 0.0 || c.a == 1.0),
        "binary alpha: opaque or empty"
    );
}

#[test]
fn a_solid_texel_hits_and_shows_its_leaf_color() {
    let e = env();
    let world = leaves_world();
    let (i, j) = find(&e, |e, i, j| e.solid[j][i]);
    let s = e.scene(&world, 1.0);

    assert!(nearest_visible_hit(&s, &front(i, j), RANGE).is_some());
    let c = trace_ray(&s, &front(i, j), 0);
    assert!(
        c.g > c.b + 0.05,
        "opaque green leaf, not the blue background: {c:?}"
    );
}

#[test]
fn an_empty_texel_lets_the_ray_continue_to_the_block_behind() {
    let e = env();
    let mut world = leaves_world();
    world.insert(
        IVec3::new(0, 0, -3),
        BlockInstance::new(BlockType::Stone, MaterialId::new(RED), Orientation::Up),
    );
    let (i, j) = find(&e, |e, i, j| !e.solid[j][i] && !e.solid[j][N - 1 - i]);
    let s = e.scene(&world, 1.0);

    let hit = nearest_visible_hit(&s, &front(i, j), RANGE).unwrap();
    assert_eq!(hit.cell, IVec3::new(0, 0, -3));
    // Not glass: no blending with anything, the red block shows at full color.
    let c = trace_ray(&s, &front(i, j), 0);
    assert!(c.r > 0.99 && c.g < 0.01 && c.b < 0.01, "{c:?}");
}

fn shadow_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Stone, MaterialId::new(RED), Orientation::Up),
    );
    w.insert(
        IVec3::new(0, 2, 0),
        BlockInstance::new(BlockType::Leaves, leaves_material_id(), Orientation::Up),
    );
    w
}

fn shadow_ray(i: usize, j_top: usize) -> Ray {
    Ray::new(
        Vec3::new((i as f32 + 0.5) / 16.0, 1.0001, (j_top as f32 + 0.5) / 16.0),
        Vec3::new(0.0, 1.0, 0.0),
    )
}

#[test]
fn shadows_pass_through_holes_and_are_blocked_by_solid_texels() {
    let e = env();
    let world = shadow_world();
    let s = e.scene(&world, 1.0);

    // Top texel (i, j) and bottom texel (i, N-1-j) are both empty: light passes.
    let (i, j) = find(&e, |e, i, j| !e.solid[j][i] && !e.solid[N - 1 - j][i]);
    assert!((light_visibility(&s, &shadow_ray(i, j), 10.0) - 1.0).abs() < 1e-4);

    // A solid bottom texel blocks it completely.
    let (i, j) = find(&e, |e, i, j| e.solid[N - 1 - j][i]);
    assert!(light_visibility(&s, &shadow_ray(i, j), 10.0) < 1e-4);
}

#[test]
fn leaves_is_a_single_official_entry_and_no_diagnostic_duplicate() {
    let entries: Vec<_> = catalog_entries()
        .into_iter()
        .filter(|e| e.block_type == BlockType::Leaves || e.material_id == leaves_material_id())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, SampleKind::Cutout);
    assert!(
        !catalog_entries()
            .iter()
            .any(|e| e.display_name.to_lowercase().contains("diagnostic"))
    );

    let scene = CatalogScene::new();
    assert_eq!(
        scene.world().get(entries[0].position),
        Some(&entries[0].block())
    );
    let p = entries[0].focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn the_reused_leaves_asset_is_the_gate_07_file_and_is_pixel_art() {
    // The asset already matches the reference exactly, so it was reused:
    // one leaves.png, no regenerated copy.
    let dir = format!("{}/assets/textures/overworld", env!("CARGO_MANIFEST_DIR"));
    let leaves: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.contains("leaves"))
        .collect();
    assert_eq!(leaves, ["leaves.png"]);

    // Nearest-neighbor: one constant color inside a texel.
    let e = env();
    let m = e.library.get(leaves_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    let a = renderer::texture_sampling::sample_nearest(
        t,
        core::math::Vec2::new(4.05 / 16.0, 6.05 / 16.0),
    );
    let b = renderer::texture_sampling::sample_nearest(
        t,
        core::math::Vec2::new(4.95 / 16.0, 6.95 / 16.0),
    );
    assert_eq!(a, b);
}
