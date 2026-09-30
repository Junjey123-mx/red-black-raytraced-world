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
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
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
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
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
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::projection::primary_ray;
use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm};
use core::color::Color;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{
    VoxelScene, light_visibility, nearest_visible_hit, nearest_voxel_hit, trace_ray,
};
use renderer::shadows::shadow_ray_to_point_light;
use scene::block::BlockInstance;
use scene::catalog::{CatalogScene, CatalogTextures, catalog_materials};
use scene::light::Light;
use scene::material_gallery::{
    GALLERY_MAX_DISTANCE, gallery_background, gallery_camera, gallery_lights,
};
use scene::material_library::MaterialLibrary;
use scene::texture_manager::TextureManager;
use scene::voxel_world::{BOUNDS_EXIT_SLACK, VoxelBounds, VoxelWorld};
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn any_block(world: &VoxelWorld) -> BlockInstance {
    *world.iter().next().unwrap().1
}

/// The same cells plus two far sentinel cells that stretch the box so wide
/// that clipping can no longer cut any traversal short: the reference
/// "unclipped" world. The sentinels are thousands of cells away, far beyond
/// every `max_distance` used, so they are never hit.
fn unclipped_copy(world: &VoxelWorld) -> VoxelWorld {
    let mut copy = VoxelWorld::new();
    for (cell, block) in world.iter() {
        copy.insert(*cell, *block);
    }
    let sentinel = any_block(world);
    copy.insert(IVec3::new(-4000, -4000, -4000), sentinel);
    copy.insert(IVec3::new(4000, 4000, 4000), sentinel);
    assert!(copy.bounds().unwrap().extent().x > 8000);
    copy
}

fn one_cell_world() -> VoxelWorld {
    let scene = WorldScene::new();
    let mut world = VoxelWorld::new();
    world.insert(IVec3::new(0, 0, 0), any_block(scene.world()));
    world
}

struct WorldRig {
    manager: TextureManager,
    scene: WorldScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
    reference: VoxelWorld,
}

impl WorldRig {
    fn new() -> Self {
        let mut manager = TextureManager::new();
        let textures = WorldTextures::load(
            &mut manager,
            "assets/textures/overworld",
            "assets/textures/overworld/grass",
            "assets/textures/portal",
        )
        .unwrap();
        let scene = WorldScene::new();
        let reference = unclipped_copy(scene.world());
        Self {
            manager,
            materials: world_materials(&textures),
            lights: world_lights(),
            scene,
            reference,
        }
    }

    fn scenes(
        &self,
        camera_position: Vec3,
        state: &WorldFreeFlyCameraState,
    ) -> (VoxelScene<'_>, VoxelScene<'_>) {
        let make = |world| VoxelScene {
            world,
            materials: &self.materials,
            camera_position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        (make(self.scene.world()), make(&self.reference))
    }
}

fn poses() -> Vec<WorldFreeFlyCameraState> {
    let home = world_free_fly_camera();
    let mut red_black = |p: Vec3, t: Vec3| {
        let mut s = WorldFreeFlyCameraState::looking_at(p, t);
        s.realm = WorldRealm::RedBlack;
        s.local_up = WorldRealm::RedBlack.up();
        s.pitch = -s.pitch;
        s
    };
    vec![
        home,
        WorldFreeFlyCameraState::looking_at(v(15.0, 11.0, 21.0), v(8.0, 6.0, 10.0)),
        WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
        red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        WorldFreeFlyCameraState::looking_at(v(7.5, 10.0, 25.0), v(7.5, 5.0, 18.0)),
        red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
    ]
}

fn render(scene: &VoxelScene, camera: &camera::camera::Camera, w: usize, h: usize) -> Vec<Color> {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            out.push(trace_ray(scene, &primary_ray(camera, x, y, w, h), 0));
        }
    }
    out
}

#[test]
fn a_ray_that_never_reaches_the_box_is_an_immediate_miss() {
    let world = one_cell_world();
    let bounds = world.bounds().unwrap();
    assert_eq!(bounds, VoxelBounds::of_cell(IVec3::new(0, 0, 0)));
    // Pointing away, passing beside, and passing above the cell.
    for (o, d) in [
        (v(3.0, 0.5, 0.5), v(1.0, 0.0, 0.0)),
        (v(-3.0, 0.5, 0.5), v(-1.0, 0.0, 0.0)),
        (v(-3.0, 2.5, 0.5), v(1.0, 0.0, 0.0)),
        (v(-3.0, 0.5, 0.5), v(1.0, 1.0, 0.0)),
    ] {
        let ray = Ray::new(o, d);
        assert_eq!(
            bounds.exit_distance(ray.origin, ray.direction),
            None,
            "{o:?} {d:?}"
        );
        assert_eq!(bounds.clip_distance(ray.origin, ray.direction, 96.0), None);
        assert!(nearest_voxel_hit(&world, &ray, 96.0).is_none());
    }
    // An empty world has no box at all.
    assert_eq!(VoxelWorld::new().bounds(), None);
}

#[test]
fn a_ray_entering_the_box_is_clipped_at_its_exit() {
    let world = one_cell_world();
    let bounds = world.bounds().unwrap();
    let ray = Ray::new(v(-2.0, 0.5, 0.5), v(1.0, 0.0, 0.0));
    let exit = bounds.exit_distance(ray.origin, ray.direction).unwrap();
    assert!((exit - 3.0).abs() < 1e-5, "exit {exit}");
    assert!(
        (bounds
            .clip_distance(ray.origin, ray.direction, 96.0)
            .unwrap()
            - (3.0 + BOUNDS_EXIT_SLACK))
            .abs()
            < 1e-5
    );
    let hit = nearest_voxel_hit(&world, &ray, 96.0).unwrap();
    assert!((hit.hit.distance - 2.0).abs() < 1e-5);
}

#[test]
fn a_ray_starting_inside_the_box_traverses_to_the_far_face() {
    let scene = WorldScene::new();
    let bounds = scene.world().bounds().unwrap();
    let inside = v(12.0, -5.0, 12.0);
    assert!(bounds.contains_cell(IVec3::new(12, -5, 12)));
    // Toward +Z the far face is z = 24: 12 units away.
    let exit = bounds.exit_distance(inside, v(0.0, 0.0, 1.0)).unwrap();
    assert!((exit - 12.0).abs() < 1e-5, "{exit}");
    let exit = bounds.exit_distance(inside, v(0.0, 1.0, 0.0)).unwrap();
    assert!(
        (exit - (bounds.max_corner().y + 5.0)).abs() < 1e-5,
        "{exit}"
    );
    // Inside a solid cell the hit is that cell's exit face, still found.
    let ray = Ray::new(v(16.5, -7.0, 13.8), v(0.0, 0.0, -1.0));
    let clipped = nearest_voxel_hit(scene.world(), &ray, 96.0);
    let reference = nearest_voxel_hit(&unclipped_copy(scene.world()), &ray, 96.0);
    assert_eq!(clipped, reference);
    assert!(clipped.is_some());
}

#[test]
fn negative_directions_clip_symmetrically() {
    let world = one_cell_world();
    let bounds = world.bounds().unwrap();
    let forward = Ray::new(v(-2.0, 0.5, 0.5), v(1.0, 0.0, 0.0));
    let backward = Ray::new(v(3.0, 0.5, 0.5), v(-1.0, 0.0, 0.0));
    let a = bounds
        .exit_distance(forward.origin, forward.direction)
        .unwrap();
    let b = bounds
        .exit_distance(backward.origin, backward.direction)
        .unwrap();
    assert!((a - 3.0).abs() < 1e-5 && (b - 3.0).abs() < 1e-5);
    let ha = nearest_voxel_hit(&world, &forward, 96.0).unwrap();
    let hb = nearest_voxel_hit(&world, &backward, 96.0).unwrap();
    assert!((ha.hit.distance - 2.0).abs() < 1e-5 && (hb.hit.distance - 2.0).abs() < 1e-5);
    // Diagonal negative direction through the box.
    let diag = Ray::new(v(4.0, 4.0, 4.0), v(-1.0, -1.0, -1.0));
    assert!(bounds.exit_distance(diag.origin, diag.direction).is_some());
    assert!(nearest_voxel_hit(&world, &diag, 96.0).is_some());
}

#[test]
fn axis_parallel_rays_respect_their_slab() {
    let world = one_cell_world();
    let bounds = world.bounds().unwrap();
    // Exactly parallel to two slabs, inside them: the box is reached.
    let inside = v(0.5, 0.5, 5.0);
    assert!(bounds.exit_distance(inside, v(0.0, 0.0, -1.0)).is_some());
    assert!(nearest_voxel_hit(&world, &Ray::new(inside, v(0.0, 0.0, -1.0)), 96.0).is_some());
    // Parallel but outside one slab: never reached.
    let outside = v(1.5, 0.5, 5.0);
    assert_eq!(bounds.exit_distance(outside, v(0.0, 0.0, -1.0)), None);
    assert!(nearest_voxel_hit(&world, &Ray::new(outside, v(0.0, 0.0, -1.0)), 96.0).is_none());
    // A nearly-parallel component (below the DDA's axis epsilon) behaves
    // like the reference world.
    let almost = Ray::new(v(0.5, 0.5, 5.0), v(1e-8, 0.0, -1.0));
    assert_eq!(
        nearest_voxel_hit(&world, &almost, 96.0),
        nearest_voxel_hit(&unclipped_copy(&world), &almost, 96.0)
    );
}

#[test]
fn corner_and_edge_grazing_rays_match_the_unclipped_reference() {
    let world = one_cell_world();
    let reference = unclipped_copy(&world);
    let mut checked = 0;
    // Rays aimed at, and just around, every corner and several edge points.
    for cx in [0.0f32, 1.0] {
        for cy in [0.0f32, 1.0] {
            for cz in [0.0f32, 1.0] {
                for (ox, oy, oz) in [
                    (-3.0, 2.0, 2.5),
                    (3.5, -2.0, 2.0),
                    (2.0, 3.0, -3.5),
                    (-2.5, -2.5, -2.5),
                ] {
                    for jitter in [-1e-3f32, 0.0, 1e-3] {
                        let origin = v(ox, oy, oz);
                        let target = v(cx + jitter, cy - jitter, cz + jitter);
                        let ray = Ray::new(origin, target - origin);
                        assert_eq!(
                            nearest_voxel_hit(&world, &ray, 96.0),
                            nearest_voxel_hit(&reference, &ray, 96.0),
                            "{origin:?} -> {target:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked >= 96);
}

#[test]
fn the_traversal_limit_is_the_smaller_of_max_distance_and_the_box_exit() {
    let bounds = VoxelBounds {
        min: IVec3::new(0, 0, 0),
        max_exclusive: IVec3::new(10, 10, 10),
    };
    let origin = v(-5.0, 5.0, 5.0);
    let dir = v(1.0, 0.0, 0.0);
    assert!((bounds.exit_distance(origin, dir).unwrap() - 15.0).abs() < 1e-5);
    assert!(
        (bounds.clip_distance(origin, dir, 96.0).unwrap() - (15.0 + BOUNDS_EXIT_SLACK)).abs()
            < 1e-5
    );
    assert!((bounds.clip_distance(origin, dir, 7.0).unwrap() - 7.0).abs() < 1e-6);
    // Behind the box: the exit is negative, so the ray misses.
    assert_eq!(bounds.exit_distance(v(20.0, 5.0, 5.0), dir), None);
    assert_eq!(bounds.extent(), IVec3::new(10, 10, 10));
}

#[test]
fn primary_hits_are_unchanged_by_clipping() {
    let rig = WorldRig::new();
    let mut hits = 0;
    for state in poses() {
        let camera = state.camera(4.0 / 3.0);
        let (clipped, reference) = rig.scenes(camera.position, &state);
        for y in 0..24 {
            for x in 0..32 {
                let ray = primary_ray(&camera, x, y, 32, 24);
                let a = nearest_visible_hit(&clipped, &ray, WORLD_MAX_DISTANCE);
                let b = nearest_visible_hit(&reference, &ray, WORLD_MAX_DISTANCE);
                assert_eq!(a, b, "pixel ({x}, {y})");
                hits += a.is_some() as usize;
            }
        }
    }
    assert!(hits > 500);
}

#[test]
fn shadow_visibility_is_unchanged_by_clipping() {
    let rig = WorldRig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let (clipped, reference) = rig.scenes(camera.position, &state);
    let mut compared = 0;
    for y in (0..24).step_by(2) {
        for x in (0..32).step_by(2) {
            let ray = primary_ray(&camera, x, y, 32, 24);
            let Some(hit) = nearest_visible_hit(&clipped, &ray, WORLD_MAX_DISTANCE) else {
                continue;
            };
            for light in &rig.lights {
                let (shadow_ray, distance) = match light {
                    Light::Directional(d) => (
                        Ray::new(hit.hit.point + hit.hit.normal * 1e-4, d.direction),
                        WORLD_MAX_DISTANCE,
                    ),
                    Light::Point(p) => shadow_ray_to_point_light(hit.hit.point, hit.hit.normal, p),
                };
                let a = light_visibility(&clipped, &shadow_ray, distance);
                let b = light_visibility(&reference, &shadow_ray, distance);
                assert_eq!(a.to_bits(), b.to_bits(), "pixel ({x}, {y})");
                compared += 1;
            }
        }
    }
    assert!(compared > 100);
}

#[test]
fn reflection_and_refraction_paths_are_unchanged_by_clipping() {
    let rig = WorldRig::new();
    // The pond view: water reflects and refracts; the portal view: the
    // transparent membrane.
    for state in [
        WorldFreeFlyCameraState::looking_at(v(7.5, 10.0, 25.0), v(7.5, 5.0, 18.0)),
        WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
    ] {
        let camera = state.camera(4.0 / 3.0);
        let (clipped, reference) = rig.scenes(camera.position, &state);
        for y in 0..30 {
            for x in 0..40 {
                let ray = primary_ray(&camera, x, y, 40, 30);
                let a = trace_ray(&clipped, &ray, 0);
                let b = trace_ray(&reference, &ray, 0);
                assert_eq!(a, b, "pixel ({x}, {y})");
                // Secondary depth explicitly.
                assert_eq!(trace_ray(&clipped, &ray, 1), trace_ray(&reference, &ray, 1));
            }
        }
    }
}

#[test]
fn the_catalog_renders_identically_with_clipping() {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let catalog = CatalogScene::new();
    let materials = catalog_materials(&textures);
    let lights = gallery_lights();
    let reference = unclipped_copy(catalog.world());
    let camera = gallery_camera(4.0 / 3.0);
    let make = |world| VoxelScene {
        world,
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: renderer::shading::DEFAULT_AMBIENT_FACTOR,
        background: renderer::skybox::Background::Solid(gallery_background()),
        texture_manager: &manager,
        max_distance: GALLERY_MAX_DISTANCE,
    };
    let a = render(&make(catalog.world()), &camera, 64, 48);
    let b = render(&make(&reference), &camera, 64, 48);
    assert_eq!(a, b);
    assert!(a.iter().any(|c| *c != gallery_background()));
}

#[test]
fn world_frames_are_bit_identical_at_reduced_resolution() {
    let rig = WorldRig::new();
    for (i, state) in poses().into_iter().enumerate() {
        let camera = state.camera(4.0 / 3.0);
        let (clipped, reference) = rig.scenes(camera.position, &state);
        let a = render(&clipped, &camera, 80, 60);
        let b = render(&reference, &camera, 80, 60);
        assert_eq!(a, b, "pose {i}");
    }
}

#[test]
fn the_world_box_is_exact_and_maintained_through_edits() {
    let scene = WorldScene::new();
    let world = scene.world();
    let bounds = world.bounds().unwrap();
    let brute = world.iter().fold(None, |b: Option<VoxelBounds>, (c, _)| {
        Some(b.map_or(VoxelBounds::of_cell(*c), |b| b.including(*c)))
    });
    assert_eq!(Some(bounds), brute);
    assert_eq!(bounds.min, IVec3::new(0, -22, 0));
    // Gate 15 grew the lobe east: the box ends at the lobe's last column.
    assert_eq!(
        bounds.max_exclusive.x,
        scene.expansion_layout().overworld_extension.max_x + 1
    );
    assert_eq!((bounds.max_exclusive.y, bounds.max_exclusive.z), (13, 24));
    // Shrinks when the only cell on a face goes away, stays otherwise.
    let block = any_block(world);
    let mut w = VoxelWorld::new();
    w.insert(IVec3::new(0, 0, 0), block);
    w.insert(IVec3::new(5, 2, 1), block);
    w.insert(IVec3::new(2, 9, 1), block);
    assert_eq!(w.bounds().unwrap().max_exclusive, IVec3::new(6, 10, 2));
    w.remove(IVec3::new(2, 9, 1));
    assert_eq!(w.bounds().unwrap().max_exclusive, IVec3::new(6, 3, 2));
    w.remove(IVec3::new(9, 9, 9));
    assert_eq!(w.bounds().unwrap().max_exclusive, IVec3::new(6, 3, 2));
    w.remove(IVec3::new(5, 2, 1));
    w.remove(IVec3::new(0, 0, 0));
    assert_eq!(w.bounds(), None);
    assert_eq!(scene.world().len(), WorldScene::new().world().len()); // deterministic
    assert!(scene.world().len() > 6000);
}
