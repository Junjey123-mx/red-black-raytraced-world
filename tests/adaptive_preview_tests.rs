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

use renderer::preview::{
    AdaptivePreview, DEMOTE_AFTER, INTERACTIVE_FRAME_BUDGET, PREVIEW_SCALES, PROMOTE_AFTER,
    PROMOTE_MARGIN,
};
use renderer::raytracer::RenderQuality;
use scene::world::WorldScene;
use std::time::Duration;

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

fn fast() -> Duration {
    // At x4 this predicts 4 * 5 = 20 ms at x2: well inside 80 % of 40 ms.
    ms(5)
}

fn slow() -> Duration {
    ms(60)
}

#[test]
fn the_default_scale_is_the_sharper_valid_one() {
    let preview = AdaptivePreview::new();
    assert_eq!(preview.scale(), 2);
    assert!(PREVIEW_SCALES.contains(&preview.scale()));
    assert_eq!(PREVIEW_SCALES, [2, 4]);
    assert_eq!(AdaptivePreview::default(), preview);
    assert_eq!(preview.resolution(800, 600), (400, 300));
    assert_eq!(INTERACTIVE_FRAME_BUDGET, ms(40));
    assert!((33..=50).contains(&(INTERACTIVE_FRAME_BUDGET.as_millis() as u64)));
}

#[test]
fn fast_frames_promote_back_to_the_sharper_scale() {
    let mut preview = AdaptivePreview::new();
    for _ in 0..DEMOTE_AFTER {
        preview.record(slow());
    }
    assert_eq!(preview.scale(), 4);
    for i in 0..PROMOTE_AFTER - 1 {
        assert_eq!(preview.record(fast()), 4, "frame {i}");
    }
    assert_eq!(preview.record(fast()), 2);
    assert_eq!(preview.resolution(800, 600), (400, 300));
}

#[test]
fn slow_frames_demote_to_the_coarser_scale() {
    let mut preview = AdaptivePreview::new();
    for i in 0..DEMOTE_AFTER - 1 {
        assert_eq!(preview.record(slow()), 2, "frame {i}");
    }
    assert_eq!(preview.record(slow()), 4);
    assert_eq!(preview.resolution(800, 600), (200, 150));
    // A frame exactly on budget is not over budget.
    let mut on_budget = AdaptivePreview::new();
    for _ in 0..10 {
        assert_eq!(on_budget.record(INTERACTIVE_FRAME_BUDGET), 2);
    }
}

#[test]
fn hysteresis_prevents_scale_thrashing() {
    // Alternating fast/slow frames never flip the scale every frame.
    let mut preview = AdaptivePreview::new();
    let mut flips = 0;
    let mut last = preview.scale();
    for i in 0..200 {
        let t = if i % 2 == 0 { slow() } else { fast() };
        let s = preview.record(t);
        if s != last {
            flips += 1;
            last = s;
        }
    }
    assert_eq!(
        flips, 0,
        "alternating frames flipped the scale {flips} times"
    );
    // Streaks reset on a contradicting frame.
    let mut p = AdaptivePreview::new();
    p.record(slow());
    p.record(slow());
    assert_eq!(p.over_budget_streak(), 2);
    p.record(fast());
    assert_eq!(p.over_budget_streak(), 0);
    // Promotion needs the predicted x2 cost within the margin, not just a
    // frame under budget at x4.
    let mut q = AdaptivePreview::new();
    for _ in 0..DEMOTE_AFTER {
        q.record(slow());
    }
    let borderline = Duration::from_secs_f64(
        INTERACTIVE_FRAME_BUDGET.as_secs_f64() * PROMOTE_MARGIN / 4.0 + 0.001,
    );
    for _ in 0..PROMOTE_AFTER * 2 {
        assert_eq!(q.record(borderline), 4);
    }
    assert_eq!(q.under_budget_streak(), 0);
}

#[test]
fn the_scale_never_goes_below_the_minimum() {
    let mut preview = AdaptivePreview::new();
    for _ in 0..100 {
        assert_eq!(preview.record(ms(1)), AdaptivePreview::min_scale());
    }
    assert_eq!(AdaptivePreview::min_scale(), 2);
    assert_eq!(preview.resolution(800, 600), (400, 300));
}

#[test]
fn the_scale_never_goes_above_the_maximum() {
    let mut preview = AdaptivePreview::new();
    for _ in 0..100 {
        preview.record(ms(500));
    }
    assert_eq!(preview.scale(), AdaptivePreview::max_scale());
    assert_eq!(AdaptivePreview::max_scale(), 4);
    assert_eq!(preview.resolution(800, 600), (200, 150));
    assert_eq!(preview.resolution(3, 2), (1, 1));
}

#[test]
fn full_frames_ignore_the_preview_scale() {
    // The Full branch of the viewer traces the full window regardless of
    // the controller; only the Interactive branch consults it.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let full_branch = &app[app.find("} else if needs_full_frame {").unwrap()..];
    let full_branch = &full_branch[..full_branch.find("needs_full_frame = false;").unwrap()];
    assert!(full_branch.contains(
        "full_width,
                full_height,"
    ));
    assert!(full_branch.contains("RenderQuality::Full"));
    assert!(!full_branch.contains("preview."));
    assert_eq!(RenderQuality::Full.label(), "Full");
}

#[test]
fn interactive_frames_use_the_controller_resolution() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let interactive = &app[app.find("if mode.update(&rl) {").unwrap()
        ..app.find("} else if needs_full_frame {").unwrap()];
    assert!(interactive.contains("preview.scale()"));
    assert!(
        interactive.contains("let (width, height) = (full_width / scale, full_height / scale);")
    );
    assert!(interactive.contains("RenderQuality::Interactive"));
    assert!(interactive.contains("preview.record(timing.render)"));
    let mut preview = AdaptivePreview::new();
    assert_eq!(preview.resolution(800, 600), (400, 300));
    for _ in 0..DEMOTE_AFTER {
        preview.record(slow());
    }
    assert_eq!(preview.resolution(800, 600), (200, 150));
}

#[test]
fn reset_restores_a_clean_controller_and_touches_no_scene() {
    let mut preview = AdaptivePreview::new();
    for _ in 0..DEMOTE_AFTER {
        preview.record(slow());
    }
    preview.record(fast());
    assert_eq!(preview.scale(), 4);
    preview.reset();
    assert_eq!(preview, AdaptivePreview::new());
    assert_eq!(
        (preview.over_budget_streak(), preview.under_budget_streak()),
        (0, 0)
    );
    assert_eq!(preview.record(fast()), 2);
    // The controller knows nothing about the scene.
    let src = std::fs::read_to_string("src/renderer/preview.rs").unwrap();
    for forbidden in ["VoxelWorld", "Material", "Camera", "raylib", "WorldScene"] {
        assert!(!src.contains(forbidden), "{forbidden}");
    }
    assert_eq!(WorldScene::new().world().len(), 6784);
}

#[test]
fn the_catalog_keeps_its_fixed_preview_unless_it_opts_in() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    // Default: fixed downscale. Only WorldMode opts in.
    assert!(app.contains(
        "fn adaptive_preview(&self) -> bool {
        false
    }"
    ));
    assert_eq!(
        app.matches(
            "fn adaptive_preview(&self) -> bool {
        true
    }"
        )
        .count(),
        1
    );
    let world_impl = &app[app.find("impl ViewerMode for WorldMode").unwrap()..];
    assert!(world_impl.contains(
        "fn adaptive_preview(&self) -> bool {
        true"
    ));
    let catalog_impl = &app[app.find("impl ViewerMode for CatalogMode").unwrap()
        ..app.find("impl ViewerMode for WorldMode").unwrap()];
    assert!(!catalog_impl.contains("adaptive_preview"));
    assert!(app.contains("const PREVIEW_DOWNSCALE: usize = 4;"));
    assert!(
        app.contains(
            "PREVIEW_DOWNSCALE
            };"
        ) || app.contains("PREVIEW_DOWNSCALE }")
    );
}
