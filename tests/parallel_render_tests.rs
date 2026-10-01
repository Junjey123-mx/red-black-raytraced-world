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
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
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
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
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

use camera::camera::Camera;
use camera::projection::primary_ray;
use camera::world_free_fly::WorldFreeFlyCameraState;
use core::color::Color;
use core::math::Vec3;
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{
    DEFAULT_BAND_ROWS, ParallelRenderConfig, THREADS_ENV_VAR, assign_bands, band_count,
    render_parallel,
};
use renderer::raytracer::{RenderQuality, VoxelScene, cast_ray_voxel_at};
use renderer::skybox::Background;
use scene::catalog::{CatalogScene, CatalogTextures, catalog_materials};
use scene::light::Light;
use scene::material_gallery::{
    GALLERY_MAX_DISTANCE, gallery_background, gallery_camera, gallery_lights,
};
use scene::material_library::MaterialLibrary;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};

struct Rig {
    manager: TextureManager,
    scene: WorldScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
}

impl Rig {
    fn new() -> Self {
        let mut manager = TextureManager::new();
        let textures = WorldTextures::load(
            &mut manager,
            "assets/textures/overworld",
            "assets/textures/overworld/grass",
            "assets/textures/portal",
        )
        .unwrap();
        Self {
            manager,
            scene: WorldScene::new(),
            materials: world_materials(&textures),
            lights: world_lights(),
        }
    }

    fn scene(&self, state: &WorldFreeFlyCameraState) -> VoxelScene<'_> {
        VoxelScene {
            world: self.scene.world(),
            materials: &self.materials,
            camera_position: state.position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        }
    }
}

/// The reference: the plain serial pixel loop.
fn serial(scene: &VoxelScene, camera: &Camera, w: usize, h: usize, q: RenderQuality) -> Vec<Color> {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            out.push(cast_ray_voxel_at(
                scene,
                &primary_ray(camera, x, y, w, h),
                q,
            ));
        }
    }
    out
}

fn parallel(
    scene: &VoxelScene,
    camera: &Camera,
    w: usize,
    h: usize,
    q: RenderQuality,
    workers: usize,
) -> Vec<Color> {
    let mut fb = Framebuffer::new(w, h);
    render_parallel(
        &mut fb,
        camera,
        scene,
        q,
        &ParallelRenderConfig::with_workers(workers),
    );
    fb.pixels().to_vec()
}

const MARKER: Color = Color {
    r: 0.123,
    g: 0.456,
    b: 0.789,
    a: 0.5,
};

#[test]
fn one_worker_equals_the_serial_loop() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Full, 1),
        reference
    );
    assert_eq!(ParallelRenderConfig::serial().workers, 1);
    assert_eq!(ParallelRenderConfig::serial().effective_workers(48), 1);
}

#[test]
fn two_workers_produce_the_serial_image() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Full, 2),
        reference
    );
}

#[test]
fn four_workers_produce_the_serial_image() {
    let rig = Rig::new();
    let state =
        WorldFreeFlyCameraState::looking_at(Vec3::new(7.5, 10.0, 25.0), Vec3::new(7.5, 5.0, 18.0));
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Full, 4),
        reference
    );
}

#[test]
fn more_workers_than_bands_are_clamped_and_still_correct() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    // 6 rows = 2 bands of 4 rows (one partial): at most 2 workers run.
    let config = ParallelRenderConfig::with_workers(64);
    assert_eq!(band_count(6, DEFAULT_BAND_ROWS), 2);
    assert_eq!(config.effective_workers(6), 2);
    assert_eq!(config.effective_workers(1), 1);
    assert_eq!(config.effective_workers(0), 1);
    let reference = serial(&scene, &camera, 40, 6, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 40, 6, RenderQuality::Full, 64),
        reference
    );
    // Zero workers requested means one.
    assert_eq!(ParallelRenderConfig::with_workers(0).workers, 1);
}

#[test]
fn parallel_output_is_deterministic_across_runs_and_worker_counts() {
    let rig = Rig::new();
    let state = WorldFreeFlyCameraState::looking_at(
        Vec3::new(16.5, -6.0, 20.5),
        Vec3::new(16.5, -7.0, 14.5),
    );
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let first = parallel(&scene, &camera, 48, 36, RenderQuality::Full, 8);
    for workers in [8, 3, 5, 8] {
        assert_eq!(
            parallel(&scene, &camera, 48, 36, RenderQuality::Full, workers),
            first,
            "{workers} workers"
        );
    }
}

#[test]
fn odd_dimensions_and_band_sizes_cover_every_row() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    for (w, h, band_rows) in [(37, 23, 4), (33, 17, 5), (7, 9, 1), (50, 31, 100)] {
        let reference = serial(&scene, &camera, w, h, RenderQuality::Full);
        let mut fb = Framebuffer::new(w, h);
        let config = ParallelRenderConfig::with_workers(3).with_band_rows(band_rows);
        render_parallel(&mut fb, &camera, &scene, RenderQuality::Full, &config);
        assert_eq!(fb.pixels(), &reference[..], "{w}x{h} bands of {band_rows}");
    }
    // Zero-sized frames are a no-op.
    let mut empty = Framebuffer::new(0, 5);
    render_parallel(
        &mut empty,
        &camera,
        &scene,
        RenderQuality::Full,
        &ParallelRenderConfig::with_workers(4),
    );
    assert!(empty.pixels().is_empty());
}

#[test]
fn the_interactive_profile_is_traced_identically_in_parallel() {
    let rig = Rig::new();
    let state =
        WorldFreeFlyCameraState::looking_at(Vec3::new(15.0, 11.0, 21.0), Vec3::new(8.0, 6.0, 10.0));
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Interactive);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Interactive, 6),
        reference
    );
    // And it is really the Interactive image, not the Full one.
    assert_ne!(
        reference,
        serial(&scene, &camera, 64, 48, RenderQuality::Full)
    );
}

#[test]
fn the_full_profile_is_traced_identically_in_parallel() {
    let rig = Rig::new();
    let state =
        WorldFreeFlyCameraState::looking_at(Vec3::new(15.0, 11.0, 21.0), Vec3::new(8.0, 6.0, 10.0));
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Full, 6),
        reference
    );
}

#[test]
fn the_world_renders_identically_from_every_benchmark_pose() {
    let rig = Rig::new();
    let mut red_black = |p: Vec3, t: Vec3| {
        let mut s = WorldFreeFlyCameraState::looking_at(p, t);
        s.realm = camera::world_free_fly::WorldRealm::RedBlack;
        s.local_up = camera::world_free_fly::WorldRealm::RedBlack.up();
        s.pitch = -s.pitch;
        s
    };
    let poses = [
        world_free_fly_camera(),
        red_black(Vec3::new(12.0, -34.0, 36.0), Vec3::new(12.0, -20.0, 12.0)),
        red_black(Vec3::new(16.5, -7.0, 13.8), Vec3::new(16.5, -7.0, 10.0)),
        WorldFreeFlyCameraState::looking_at(
            Vec3::new(12.0, -34.0, 36.0),
            Vec3::new(12.0, -48.0, 60.0),
        ),
    ];
    let workers = ParallelRenderConfig::detect().workers.max(2);
    for state in poses {
        let camera = state.camera(4.0 / 3.0);
        let scene = rig.scene(&state);
        let reference = serial(&scene, &camera, 40, 30, RenderQuality::Full);
        assert_eq!(
            parallel(&scene, &camera, 40, 30, RenderQuality::Full, workers),
            reference
        );
    }
}

#[test]
fn the_catalog_renders_identically_in_parallel() {
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
    let camera = gallery_camera(4.0 / 3.0);
    let scene = VoxelScene {
        world: catalog.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: renderer::shading::DEFAULT_AMBIENT_FACTOR,
        background: Background::Solid(gallery_background()),
        texture_manager: &manager,
        max_distance: GALLERY_MAX_DISTANCE,
    };
    let reference = serial(&scene, &camera, 64, 48, RenderQuality::Full);
    assert_eq!(
        parallel(&scene, &camera, 64, 48, RenderQuality::Full, 8),
        reference
    );
    assert_eq!(catalog.entries().len(), 46);
}

#[test]
fn no_pixel_is_left_unwritten() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let mut fb = Framebuffer::new(53, 41);
    fb.clear(MARKER);
    render_parallel(
        &mut fb,
        &camera,
        &scene,
        RenderQuality::Full,
        &ParallelRenderConfig::with_workers(7),
    );
    assert!(fb.pixels().iter().all(|p| *p != MARKER));
    assert!(fb.pixels().iter().all(|p| p.a == 1.0));
}

#[test]
fn bands_are_dealt_without_overlap_or_duplicates() {
    for (height, band_rows, workers) in [
        (600, 4, 20),
        (150, 4, 20),
        (23, 5, 3),
        (9, 1, 64),
        (1, 4, 4),
        (0, 4, 4),
    ] {
        let assignment = assign_bands(height, band_rows, workers);
        assert!(assignment.len() <= workers.max(1));
        let mut rows = vec![0usize; height];
        for bands in &assignment {
            for (start, end) in bands {
                assert!(start < end && *end <= height);
                for r in *start..*end {
                    rows[r] += 1;
                }
            }
        }
        assert!(
            rows.iter().all(|&n| n == 1),
            "{height} rows, {band_rows} per band, {workers} workers: {rows:?}"
        );
        // Round-robin: consecutive bands go to consecutive workers.
        if height >= band_rows * workers {
            for (w, bands) in assignment.iter().enumerate() {
                assert_eq!(bands[0].0, w * band_rows);
            }
        }
    }
    assert_eq!(THREADS_ENV_VAR, "RBRW_THREADS");
    assert!(ParallelRenderConfig::detect().workers >= 1);
}
