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
    #[path = "../src/core/texture.rs"]
    pub mod texture;
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
use core::material::{Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{
    MAX_RAY_DEPTH, MISSING_MATERIAL_COLOR, VoxelScene, cast_ray_voxel_lit, nearest_voxel_hit,
    trace_ray,
};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 50.0;

const MIRROR: u32 = 1;
const RED: u32 = 2;

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
    BlockInstance::new(block_type, MaterialId::new(id), Orientation::South)
}

/// `MIRROR` is grey with the given reflectivity, `RED` is a plain red matte.
fn library(mirror_reflectivity: f32) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    library.insert(
        MaterialId::new(MIRROR),
        Material::matte(grey()).with_reflectivity(mirror_reflectivity),
    );
    library.insert(MaterialId::new(RED), Material::matte(red()));
    library
}

/// With no lights and `ambient_factor = 1`, local shading is exactly the
/// albedo, which makes every expected color a simple blend.
fn trace(world: &VoxelWorld, materials: &MaterialLibrary, r: &Ray, depth: u32) -> Color {
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

/// A mirror floor along +x with a red block up in the air, positioned so the
/// 45-degree bounce from `floor_ray()` lands on it.
fn mirror_floor_and_red_wall() -> VoxelWorld {
    let mut world = VoxelWorld::new();
    for x in 0..8 {
        world.insert(cell(x, 0, 0), block(BlockType::Stone, MIRROR));
    }
    world.insert(cell(6, 3, 0), block(BlockType::Stone, RED));
    world
}

/// Hits the floor top at (3.5, 1, 0.5); the mirror ray then travels
/// (1, 1, 0) and reaches the red block at (6, 3.5, 0.5).
fn floor_ray() -> Ray {
    ray((1.5, 3.0, 0.5), (1.0, -1.0, 0.0))
}

#[test]
fn a_non_reflective_material_keeps_its_local_shading() {
    let world = mirror_floor_and_red_wall();
    let color = trace(&world, &library(0.0), &floor_ray(), 0);

    assert!(approx_color(color, grey()), "color = {color:?}");
}

#[test]
fn full_reflectivity_returns_the_reflected_scene_color() {
    let world = mirror_floor_and_red_wall();
    let color = trace(&world, &library(1.0), &floor_ray(), 0);

    assert!(approx_color(color, red()), "color = {color:?}");
}

#[test]
fn intermediate_reflectivity_blends_local_and_reflected_color() {
    let world = mirror_floor_and_red_wall();
    let color = trace(&world, &library(0.5), &floor_ray(), 0);

    // 0.5 * grey + 0.5 * red.
    assert!(
        approx_color(color, Color::new(0.75, 0.25, 0.25, 1.0)),
        "color = {color:?}"
    );
}

#[test]
fn the_depth_limit_stops_recursion_and_returns_local_shading() {
    let world = mirror_floor_and_red_wall();
    let materials = library(1.0);

    let at_limit = trace(&world, &materials, &floor_ray(), MAX_RAY_DEPTH);
    assert!(approx_color(at_limit, grey()), "at_limit = {at_limit:?}");

    let below_limit = trace(&world, &materials, &floor_ray(), MAX_RAY_DEPTH - 1);
    assert!(approx_color(below_limit, red()), "below = {below_limit:?}");
}

#[test]
fn two_facing_mirrors_terminate_at_the_depth_limit() {
    // Mirror floor and mirror ceiling: the ray would bounce forever.
    let mut world = VoxelWorld::new();
    for x in 0..40 {
        world.insert(cell(x, 0, 0), block(BlockType::Stone, MIRROR));
        world.insert(cell(x, 4, 0), block(BlockType::Stone, MIRROR));
    }

    let color = trace(
        &world,
        &library(1.0),
        &ray((0.5, 2.5, 0.5), (1.0, -0.4, 0.0)),
        0,
    );

    assert!(color.r.is_finite() && color.g.is_finite() && color.b.is_finite());
    // Every bounce ends on a mirror or, at the limit, on local grey.
    assert!(approx_color(color, grey()) || approx_color(color, background()));
}

#[test]
fn reflection_works_off_thin_partial_geometry() {
    // A door leaf (thin prism) in cell (0,0,0), a red block behind the eye.
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::WoodDoor, MIRROR));
    world.insert(cell(0, 0, 6), block(BlockType::Stone, RED));

    let r = ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &r, RANGE).expect("door must be hit");
    assert_eq!(hit.block.block_type(), BlockType::WoodDoor);
    assert_eq!(hit.hit.face, Face::PositiveZ);

    let color = trace(&world, &library(1.0), &r, 0);
    assert!(approx_color(color, red()), "color = {color:?}");
}

#[test]
fn the_reflected_ray_does_not_hit_the_surface_it_leaves() {
    // A lone mirror block, looked at head-on: the bounce has nothing to hit.
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Stone, MIRROR));

    let color = trace(
        &world,
        &library(1.0),
        &ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0)),
        0,
    );

    assert!(approx_color(color, background()), "color = {color:?}");
}

#[test]
fn a_reflected_miss_returns_the_background_blend() {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Stone, MIRROR));

    let color = trace(
        &world,
        &library(0.25),
        &ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0)),
        0,
    );

    // 0.75 * grey + 0.25 * background.
    assert!(
        approx_color(color, Color::new(0.375, 0.375, 0.625, 1.0)),
        "color = {color:?}"
    );
}

#[test]
fn the_public_entry_point_matches_trace_ray_at_depth_zero() {
    let world = mirror_floor_and_red_wall();
    let materials = library(0.5);
    let manager = TextureManager::new();
    let r = floor_ray();

    let direct = cast_ray_voxel_lit(
        &world,
        &materials,
        &r,
        r.origin,
        &[],
        1.0,
        background(),
        &manager,
        RANGE,
    );

    assert!(approx_color(direct, trace(&world, &materials, &r, 0)));
    assert_ne!(direct, MISSING_MATERIAL_COLOR);
}
