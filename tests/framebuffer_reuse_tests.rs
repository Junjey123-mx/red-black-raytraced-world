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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_free_fly::WorldFreeFlyCameraState;
use core::color::Color;
use core::math::Vec3;
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::raytracer::{RenderQuality, VoxelScene};
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
    threads: ParallelRenderConfig,
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
            threads: ParallelRenderConfig::with_workers(4),
        }
    }

    fn render_into(&self, fb: &mut Framebuffer, state: &WorldFreeFlyCameraState, q: RenderQuality) {
        let scene = VoxelScene {
            world: self.scene.world(),
            materials: &self.materials,
            camera_position: state.position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        render_parallel(fb, &state.camera(4.0 / 3.0), &scene, q, &self.threads);
    }
}

const MARKER: Color = Color {
    r: 0.123,
    g: 0.456,
    b: 0.789,
    a: 0.5,
};

#[test]
fn resizing_to_the_same_size_keeps_the_allocation_and_contents() {
    let mut fb = Framebuffer::new(40, 30);
    fb.clear(MARKER);
    let before = fb.pixels().as_ptr();
    fb.resize(40, 30);
    assert_eq!(fb.pixels().as_ptr(), before);
    assert_eq!((fb.width(), fb.height(), fb.pixels().len()), (40, 30, 1200));
    assert!(fb.pixels().iter().all(|p| *p == MARKER));
}

#[test]
fn resizing_to_another_size_reallocates_the_pixel_count() {
    let mut fb = Framebuffer::new(40, 30);
    fb.clear(MARKER);
    fb.resize(20, 15);
    assert_eq!((fb.width(), fb.height(), fb.pixels().len()), (20, 15, 300));
    // A new size starts black: nothing of the old image is kept.
    assert!(fb.pixels().iter().all(|p| *p == Color::black()));
    fb.resize(80, 60);
    assert_eq!(fb.pixels().len(), 4800);
    assert!(fb.pixels().iter().all(|p| *p == Color::black()));
}

#[test]
fn the_rgba_bytes_equal_the_per_pixel_conversion() {
    let rig = Rig::new();
    let mut fb = Framebuffer::new(32, 24);
    rig.render_into(&mut fb, &world_free_fly_camera(), RenderQuality::Full);
    let mut bytes = Vec::new();
    fb.write_rgba8(&mut bytes);
    assert_eq!(bytes.len(), 32 * 24 * 4);
    for (i, pixel) in fb.pixels().iter().enumerate() {
        assert_eq!(&bytes[i * 4..i * 4 + 4], &pixel.to_rgba8());
    }
    // Row-major, no flip: the first bytes are pixel (0, 0).
    assert_eq!(&bytes[..4], &fb.get_pixel(0, 0).unwrap().to_rgba8());
    let last = fb.get_pixel(31, 23).unwrap().to_rgba8();
    assert_eq!(&bytes[bytes.len() - 4..], &last);
    // Every representable value converts exactly like `to_rgba8`.
    let mut sweep = Framebuffer::new(1024, 1);
    for i in 0..1024 {
        let v = i as f32 / 1023.0;
        sweep.set_pixel(i, 0, Color::new(v, 1.0 - v, v * 0.5, v));
    }
    sweep.set_pixel(0, 0, Color::new(-0.5, 1.5, f32::NAN.max(0.0), 0.5));
    let mut sweep_bytes = Vec::new();
    sweep.write_rgba8(&mut sweep_bytes);
    for (i, pixel) in sweep.pixels().iter().enumerate() {
        assert_eq!(
            &sweep_bytes[i * 4..i * 4 + 4],
            &pixel.to_rgba8(),
            "pixel {i}"
        );
    }
    // The buffer is reused, not reallocated, on the next write.
    let capacity = bytes.capacity();
    let ptr = bytes.as_ptr();
    fb.write_rgba8(&mut bytes);
    assert_eq!((bytes.capacity(), bytes.as_ptr()), (capacity, ptr));
}

#[test]
fn alpha_is_preserved_in_the_presented_bytes() {
    let mut fb = Framebuffer::new(3, 2);
    fb.set_pixel(0, 0, Color::new(1.0, 0.5, 0.0, 1.0));
    fb.set_pixel(1, 0, Color::new(0.0, 0.0, 0.0, 0.0));
    fb.set_pixel(2, 0, Color::new(0.2, 0.4, 0.6, 0.5));
    let mut bytes = Vec::new();
    fb.write_rgba8(&mut bytes);
    assert_eq!(&bytes[..4], &[255, 128, 0, 255]);
    assert_eq!(&bytes[4..8], &[0, 0, 0, 0]);
    assert_eq!(&bytes[8..12], &[51, 102, 153, 128]);
    // Traced pixels are always opaque.
    let rig = Rig::new();
    let mut traced = Framebuffer::new(16, 12);
    rig.render_into(
        &mut traced,
        &world_free_fly_camera(),
        RenderQuality::Interactive,
    );
    let mut b = Vec::new();
    traced.write_rgba8(&mut b);
    assert!(b.chunks(4).all(|px| px[3] == 255));
}

#[test]
fn switching_preview_scales_reuses_one_framebuffer_correctly() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let mut fb = Framebuffer::new(40, 30);
    let mut fresh_40 = Framebuffer::new(40, 30);
    let mut fresh_20 = Framebuffer::new(20, 15);
    rig.render_into(&mut fresh_40, &state, RenderQuality::Interactive);
    rig.render_into(&mut fresh_20, &state, RenderQuality::Interactive);
    for (w, h, fresh) in [
        (40, 30, &fresh_40),
        (20, 15, &fresh_20),
        (40, 30, &fresh_40),
        (20, 15, &fresh_20),
    ] {
        fb.resize(w, h);
        rig.render_into(&mut fb, &state, RenderQuality::Interactive);
        assert_eq!((fb.width(), fb.height()), (w, h));
        assert_eq!(fb.pixels(), fresh.pixels());
    }
}

#[test]
fn the_full_frame_reuses_its_storage_across_frames() {
    let rig = Rig::new();
    let mut fb = Framebuffer::new(80, 60);
    let ptr = fb.pixels().as_ptr();
    let home = world_free_fly_camera();
    let pond =
        WorldFreeFlyCameraState::looking_at(Vec3::new(7.5, 10.0, 25.0), Vec3::new(7.5, 5.0, 18.0));
    rig.render_into(&mut fb, &home, RenderQuality::Full);
    let first = fb.pixels().to_vec();
    fb.resize(80, 60);
    rig.render_into(&mut fb, &pond, RenderQuality::Full);
    assert_eq!(fb.pixels().as_ptr(), ptr);
    assert_ne!(fb.pixels(), &first[..]);
    let mut fresh = Framebuffer::new(80, 60);
    rig.render_into(&mut fresh, &pond, RenderQuality::Full);
    assert_eq!(fb.pixels(), fresh.pixels());
}

#[test]
fn no_stale_pixels_survive_a_reuse() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let mut fb = Framebuffer::new(40, 30);
    fb.clear(MARKER);
    rig.render_into(&mut fb, &state, RenderQuality::Full);
    assert!(fb.pixels().iter().all(|p| *p != MARKER));
    // Grow, mark, render again: every pixel of the new size is written.
    fb.resize(48, 36);
    fb.clear(MARKER);
    rig.render_into(&mut fb, &state, RenderQuality::Full);
    assert_eq!(fb.pixels().len(), 48 * 36);
    assert!(fb.pixels().iter().all(|p| *p != MARKER));
}

#[test]
fn the_catalog_presents_the_same_image_through_a_reused_buffer() {
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
    let threads = ParallelRenderConfig::with_workers(4);
    let mut fresh = Framebuffer::new(64, 48);
    render_parallel(&mut fresh, &camera, &scene, RenderQuality::Full, &threads);
    let mut reused = Framebuffer::new(16, 12);
    reused.clear(MARKER);
    reused.resize(64, 48);
    render_parallel(&mut reused, &camera, &scene, RenderQuality::Full, &threads);
    assert_eq!(reused.pixels(), fresh.pixels());
    let (mut a, mut b) = (Vec::new(), Vec::new());
    fresh.write_rgba8(&mut a);
    reused.write_rgba8(&mut b);
    assert_eq!(a, b);
    // The viewer updates the texture in place and only creates one when
    // the size changes; there is no per-pixel image drawing any more.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains(".update_texture(&self.rgba)"));
    assert_eq!(app.matches("Self::create_texture(").count(), 2);
    assert!(
        app.contains(
            "if framebuffer.width() != self.width || framebuffer.height() != self.height {"
        )
    );
    assert!(!app.contains("draw_pixel") && !app.contains("framebuffer_to_image"));
    assert!(app.contains("framebuffer.resize(width, height);"));
}
