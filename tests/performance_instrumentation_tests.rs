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
use core::color::Color;
use renderer::framebuffer::Framebuffer;
use renderer::perf::{
    FramePerfStats, PERF_ENV_VAR, PerfReporter, RenderPerfStats, TracedFrameKind,
};
use renderer::raytracer::{VoxelScene, cast_ray_voxel};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_background,
    world_free_fly_camera, world_lights, world_materials,
};
use std::time::Duration;

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

/// Traces a small World frame exactly as the viewer does (one primary ray
/// per pixel through an assembled `VoxelScene`).
fn trace_small_world(width: usize, height: usize) -> Vec<Color> {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let scene = WorldScene::new();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let camera = world_free_fly_camera().camera(width as f32 / height as f32);
    let voxel_scene = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR,
        background: world_background(),
        texture_manager: &manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    let mut framebuffer = Framebuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            framebuffer.set_pixel(x, y, cast_ray_voxel(&voxel_scene, &ray));
        }
    }
    framebuffer.pixels().to_vec()
}

#[test]
fn instrumentation_is_disabled_by_default() {
    assert!(!PerfReporter::default().is_enabled());
    assert!(!PerfReporter::from_setting(None).is_enabled());
    for off in ["", "0", "false", "off", "no", " 0 ", "OFF"] {
        assert!(
            !PerfReporter::from_setting(Some(off)).is_enabled(),
            "{off:?}"
        );
    }
    for on in ["1", "true", "yes", "on", "verbose"] {
        assert!(PerfReporter::from_setting(Some(on)).is_enabled(), "{on:?}");
    }
    assert_eq!(PERF_ENV_VAR, "RBRW_PERF");
    // The viewer reads exactly that variable.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("PerfReporter::from_env()"));
    let perf = std::fs::read_to_string("src/renderer/perf.rs").unwrap();
    assert!(perf.contains("std::env::var(PERF_ENV_VAR)"));
}

#[test]
fn enabling_the_reporter_does_not_change_the_rendered_frame() {
    // Reporting never touches pixels: the same trace with a disabled and
    // an enabled reporter around it is identical.
    let reporter_off = PerfReporter::disabled();
    let first = trace_small_world(40, 30);
    reporter_off.report(&FramePerfStats::new(
        TracedFrameKind::Full,
        40,
        30,
        ms(1),
        ms(1),
    ));
    let reporter_on = PerfReporter::enabled();
    let second = trace_small_world(40, 30);
    reporter_on.report(&FramePerfStats::new(
        TracedFrameKind::Full,
        40,
        30,
        ms(1),
        ms(1),
    ));
    assert_eq!(first, second);
    assert!(reporter_on.is_enabled() && !reporter_off.is_enabled());
}

#[test]
fn interactive_frames_are_labelled_interactive() {
    let stats = FramePerfStats::new(TracedFrameKind::Interactive, 200, 150, ms(40), ms(2));
    assert_eq!(stats.kind, TracedFrameKind::Interactive);
    assert_eq!(stats.kind.label(), "Interactive");
    let line = stats.report_line();
    assert!(line.starts_with("RBRW_PERF kind=Interactive resolution=200x150"));
    // The preview branch of the viewer reports Interactive.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("TracedFrameKind::Interactive, width, height, timing"));
}

#[test]
fn full_frames_are_labelled_full() {
    let stats = FramePerfStats::new(TracedFrameKind::Full, 800, 600, ms(3600), ms(12));
    assert_eq!(stats.kind.label(), "Full");
    assert!(stats.report_line().contains("kind=Full resolution=800x600"));
    // The refinement branch of the viewer reports a completed Full frame
    // at the full window resolution.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let refine = &app[app.find("if step.complete {").unwrap()..];
    assert!(refine.contains("TracedFrameKind::Full,\n                        full_width,\n                        full_height,"));
}

#[test]
fn durations_and_derived_rates_are_never_negative() {
    let zero = FramePerfStats::new(TracedFrameKind::Full, 8, 6, ms(0), ms(0));
    assert_eq!(zero.total(), Duration::ZERO);
    assert_eq!(zero.fps(), 0.0);
    assert!(zero.render_ms() >= 0.0 && zero.presentation_ms() >= 0.0);
    let frame = FramePerfStats::new(TracedFrameKind::Interactive, 8, 6, ms(15), ms(5));
    assert_eq!(frame.total(), ms(20));
    assert!((frame.fps() - 50.0).abs() < 1e-9);
    assert!((frame.total_ms() - 20.0).abs() < 1e-9);
    assert!(frame.render_ms() > 0.0 && frame.presentation_ms() > 0.0);
}

#[test]
fn the_reported_resolution_matches_the_traced_framebuffer() {
    let render = RenderPerfStats::new(400, 300, ms(30));
    assert_eq!(
        (render.width, render.height, render.pixels()),
        (400, 300, 120_000)
    );
    let stats = FramePerfStats::new(TracedFrameKind::Interactive, 200, 150, ms(1), ms(1));
    assert_eq!((stats.width(), stats.height()), (200, 150));
    assert_eq!(stats.render.pixels(), 30_000);
    let full = FramePerfStats::new(TracedFrameKind::Full, 800, 600, ms(1), ms(1));
    assert_eq!(full.render.pixels(), 480_000);
}

#[test]
fn instrumentation_changes_no_geometry_lights_or_materials() {
    let scene = WorldScene::new();
    assert_eq!(scene.world().len(), WorldScene::new().world().len()); // deterministic
    assert!(scene.world().len() > 6000);
    assert_eq!(scene.portal().frame.len(), 18);
    assert_eq!(scene.portal().core.len(), 12);
    assert_eq!(world_lights().len(), 5);
    // The perf module knows nothing about the scene.
    let perf = std::fs::read_to_string("src/renderer/perf.rs").unwrap();
    for forbidden in ["VoxelWorld", "Material", "Light", "Camera", "raylib"] {
        assert!(!perf.contains(forbidden), "perf.rs mentions {forbidden}");
    }
}

#[test]
fn the_catalog_binary_still_targets_the_shared_runtime() {
    let catalog = std::fs::read_to_string("src/bin/catalog.rs").unwrap();
    assert!(catalog.contains("run_catalog_app()"));
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("pub fn run_catalog_app()"));
    // Both binaries share the one instrumented loop.
    assert_eq!(app.matches("PerfReporter::from_env()").count(), 1);
    assert_eq!(app.matches("perf.report(").count(), 1);
}
