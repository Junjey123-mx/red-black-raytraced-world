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
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
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
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{MAX_RAY_DEPTH, MISSING_MATERIAL_COLOR, VoxelScene, trace_ray};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::material_gallery::{
    GalleryTextures, advanced_materials_library, glass_material_id, water_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-3;
const RANGE: f32 = 50.0;

const FLOOR: u32 = 1;
const RED: u32 = 2;
const CLEAR: u32 = 3;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

fn approx_color(a: Color, b: Color) -> bool {
    approx(a.r, b.r) && approx(a.g, b.g) && approx(a.b, b.b)
}

fn grey() -> Color {
    Color::new(0.5, 0.5, 0.5, 1.0)
}

fn red() -> Color {
    Color::new(1.0, 0.0, 0.0, 1.0)
}

fn background() -> Color {
    Color::new(0.0, 0.0, 1.0, 1.0)
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn block(block_type: BlockType, id: u32) -> BlockInstance {
    BlockInstance::new(block_type, MaterialId::new(id), Orientation::Up)
}

/// `CLEAR` is the transparent specimen under test; `RED` is a plain red
/// matte; `FLOOR` is a plain grey matte.
fn library(clear: Material) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    library.insert(MaterialId::new(FLOOR), Material::matte(grey()));
    library.insert(MaterialId::new(RED), Material::matte(red()));
    library.insert(MaterialId::new(CLEAR), clear);
    library
}

fn clear_material(transparency: f32, ior: f32) -> Material {
    Material::matte(grey())
        .with_transparency(transparency)
        .with_refractive_index(ior)
}

/// With no lights and `ambient_factor = 1`, local shading is exactly the
/// albedo, so every expected color is a simple blend.
fn trace_at(world: &VoxelWorld, materials: &MaterialLibrary, r: &Ray, depth: u32) -> Color {
    let manager = TextureManager::new();
    let scene = VoxelScene {
        world,
        materials,
        camera_position: r.origin,
        lights: &[],
        ambient_factor: 1.0,
        background: background(),
        texture_manager: &manager,
        max_distance: RANGE,
    };
    trace_ray(&scene, r, depth)
}

fn trace(world: &VoxelWorld, materials: &MaterialLibrary, r: &Ray) -> Color {
    trace_at(world, materials, r, 0)
}

fn finite(c: Color) -> bool {
    c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && (0.0..=1.0).contains(&c.r)
}

/// A transparent cube at (0,1,0) with a red block straight behind it.
fn cube_in_front_of_red() -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 1, 0), block(BlockType::Glass, CLEAR));
    world.insert(cell(0, 1, -3), block(BlockType::Stone, RED));
    world
}

fn head_on() -> Ray {
    ray((0.5, 1.5, 5.0), (0.0, 0.0, -1.0))
}

#[test]
fn zero_transparency_stays_fully_opaque() {
    let world = cube_in_front_of_red();
    let color = trace(&world, &library(clear_material(0.0, 1.5)), &head_on());

    assert!(approx_color(color, grey()), "color = {color:?}");
}

#[test]
fn full_transparency_shows_what_lies_behind() {
    let world = cube_in_front_of_red();
    // IOR 1: index-matched, so the ray goes straight through.
    let color = trace(&world, &library(clear_material(1.0, 1.0)), &head_on());

    assert!(approx_color(color, red()), "color = {color:?}");
}

#[test]
fn an_object_behind_a_transparent_material_contributes_to_its_color() {
    let world = cube_in_front_of_red();
    let color = trace(&world, &library(clear_material(0.8, 1.5)), &head_on());

    // Entry: 0.2 * grey + 0.8 * exit; exit: 0.2 * grey + 0.8 * red.
    let exit = Color::new(0.9, 0.1, 0.1, 1.0);
    let expected = Color::new(
        0.5 * 0.2 + 0.8 * exit.r,
        0.5 * 0.2 + 0.8 * exit.g,
        0.5 * 0.2 + 0.8 * exit.b,
        1.0,
    );
    assert!(approx_color(color, expected), "color = {color:?}");
    assert!(color.r > color.g + 0.5, "red must show through");
}

#[test]
fn glass_and_water_use_their_expected_indices_of_refraction() {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
    )
    .unwrap();
    let library = advanced_materials_library(&textures);

    let glass = library.get(glass_material_id()).unwrap();
    let water = library.get(water_material_id()).unwrap();

    assert!(approx(glass.refractive_index, 1.5));
    assert!(approx(water.refractive_index, 1.33));
    assert_ne!(glass.refractive_index, water.refractive_index);
    assert!(glass.transparency > water.transparency);
    assert!(glass.specular > 0.5 && water.specular > 0.4);
    assert!(glass.reflectivity > 0.0 && water.reflectivity > glass.reflectivity);
}

#[test]
fn different_indices_bend_an_oblique_ray_to_different_colors() {
    // Red/grey floor stripes under a transparent cube; sweep rays across it.
    let mut world = VoxelWorld::new();
    for x in -6..8 {
        let id = if x % 2 == 0 { RED } else { FLOOR };
        world.insert(cell(x, 0, 0), block(BlockType::Stone, id));
    }
    world.insert(cell(0, 1, 0), block(BlockType::Glass, CLEAR));

    let sweep = |ior: f32| -> Vec<Color> {
        let materials = library(clear_material(1.0, ior));
        (0..24)
            .map(|i| {
                let x = 0.05 + i as f32 * 0.04;
                trace(
                    &world,
                    &materials,
                    &ray((x - 3.0, 4.0, 0.5), (3.0, -3.0, 0.0)),
                )
            })
            .collect()
    };

    let straight = sweep(1.0);
    let glass = sweep(1.5);
    let water = sweep(1.33);

    assert!(glass.iter().all(|c| finite(*c)));
    assert_ne!(glass, straight, "glass must bend rays");
    assert_ne!(water, straight, "water must bend rays");
    assert_ne!(glass, water, "glass and water must bend differently");
}

#[test]
fn total_internal_reflection_is_stable_and_traps_the_ray() {
    // Grazing entry through the top face: inside the cube the ray meets the
    // +x face beyond the critical angle (asin(1/1.5) ~ 41.8 deg).
    let world = {
        let mut world = VoxelWorld::new();
        world.insert(cell(0, 1, 0), block(BlockType::Glass, CLEAR));
        world.insert(cell(2, 1, 0), block(BlockType::Stone, RED));
        world
    };
    let r = ray((-1.6, 2.624, 0.5), (0.95, -0.312, 0.0));

    let bent = trace(&world, &library(clear_material(1.0, 1.5)), &r);
    let unbent = trace(&world, &library(clear_material(1.0, 1.0)), &r);

    assert!(finite(bent), "bent = {bent:?}");
    assert_ne!(bent, MISSING_MATERIAL_COLOR);
    // Without bending the ray leaves through +x and reaches the red block;
    // with TIR the red block is never seen.
    assert!(unbent.r > 0.8, "unbent = {unbent:?}");
    assert!(bent.r < 0.5, "bent = {bent:?}");
}

#[test]
fn reflection_and_refraction_coexist_on_one_surface() {
    // Floor red under the cube, blue background above it: a 45-degree ray
    // reflects up into the blue background and refracts down to the red floor.
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Stone, RED));
    world.insert(cell(0, 1, 0), block(BlockType::Glass, CLEAR));
    let material = clear_material(0.6, 1.5).with_reflectivity(0.3);

    let color = trace(
        &world,
        &library(material),
        &ray((-1.5, 3.0, 0.5), (1.0, -1.0, 0.0)),
    );

    assert!(finite(color));
    assert!(color.b > 0.25, "reflected blue background: {color:?}");
    assert!(color.r > 0.15, "refracted red floor: {color:?}");
}

#[test]
fn recursion_stays_bounded_between_transmissive_and_reflective_surfaces() {
    // A gallery of parallel glass slabs would recurse forever without a cap.
    let mut world = VoxelWorld::new();
    for y in 0..6 {
        world.insert(cell(0, y, 0), block(BlockType::Glass, CLEAR));
    }
    let material = clear_material(0.5, 1.5).with_reflectivity(0.5);
    let materials = library(material);

    for dir in [(0.2, -1.0, 0.0), (1.0, -0.3, 0.1), (0.0, -1.0, 0.0)] {
        let color = trace(&world, &materials, &ray((0.4, 9.0, 0.5), dir));
        assert!(finite(color), "color = {color:?}");
    }

    // At the depth limit only local shading remains (no secondary rays).
    let at_limit = trace_at(&world, &materials, &head_on_column(), MAX_RAY_DEPTH);
    assert!(approx_color(at_limit, grey()), "at_limit = {at_limit:?}");
}

fn head_on_column() -> Ray {
    ray((0.5, 2.5, 5.0), (0.0, 0.0, -1.0))
}

#[test]
fn a_thin_partial_geometry_slab_can_be_transparent() {
    // A door leaf (thin prism) in front of a red block.
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        BlockInstance::new(
            BlockType::WoodDoor,
            MaterialId::new(CLEAR),
            Orientation::South,
        ),
    );
    world.insert(cell(0, 0, -3), block(BlockType::Stone, RED));

    let color = trace(
        &world,
        &library(clear_material(0.9, 1.5)),
        &ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0)),
    );

    assert!(color.r > 0.85 && color.g < 0.2, "color = {color:?}");
}

#[test]
fn texture_alpha_only_matters_in_blend_mode() {
    let base = clear_material(0.92, 1.5);
    let blended = base.clone().with_alpha_mode(AlphaMode::Blend);

    // Ignore: the scalar transparency, whatever the texel alpha.
    assert!(approx(base.effective_transparency(0.0), 0.92));
    assert!(approx(base.effective_transparency(1.0), 0.92));
    // Blend: an opaque texel (a frame) cancels it; a clear texel keeps it.
    assert!(approx(blended.effective_transparency(1.0), 0.0));
    assert!(approx(blended.effective_transparency(0.0), 0.92));
    assert!(approx(blended.effective_transparency(0.5), 0.46));
}

#[test]
fn a_glass_frame_texel_is_opaque_while_a_clear_texel_is_transmissive() {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
    )
    .unwrap();
    let library = advanced_materials_library(&textures);
    let glass = library.get(glass_material_id()).unwrap();
    assert_eq!(glass.alpha_mode, AlphaMode::Blend);

    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        block(BlockType::Glass, glass_material_id().value()),
    );
    world.insert(
        cell(0, 0, -3),
        block(BlockType::Stone, glass_material_id().value() + 100),
    );
    let mut lib = MaterialLibrary::new();
    lib.insert(glass_material_id(), glass.clone());
    lib.insert(
        MaterialId::new(glass_material_id().value() + 100),
        Material::matte(red()),
    );

    let scene = VoxelScene {
        world: &world,
        materials: &lib,
        camera_position: Vec3::new(0.5, 0.5, 5.0),
        lights: &[],
        ambient_factor: 1.0,
        background: background(),
        texture_manager: &manager,
        max_distance: RANGE,
    };

    // Center of the +z face: clear texel -> the red block shows through.
    let center = trace_ray(&scene, &ray((0.53, 0.53, 5.0), (0.0, 0.0, -1.0)), 0);
    assert!(center.r > 0.5 && center.r > center.b, "center = {center:?}");

    // Top-left corner texel of the +z face is the opaque frame: red hidden.
    let frame = trace_ray(&scene, &ray((0.02, 0.98, 5.0), (0.0, 0.0, -1.0)), 0);
    assert!(frame.r < center.r, "frame = {frame:?}, center = {center:?}");
}
