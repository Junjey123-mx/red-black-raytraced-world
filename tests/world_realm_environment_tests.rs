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

use camera::portal_crossing::PortalCrossingEvent;
use camera::world_free_fly::{
    FreeFlyInput, PORTAL_TRANSITION_DURATION, WorldRealm, apply_free_fly_input,
};
use core::color::Color;
use core::math::Vec3;
use renderer::skybox::Background;
use scene::environment::WorldEnvironmentProfile;
use scene::light::Light;
use scene::material_gallery::gallery_background;
use scene::world::{world_background, world_free_fly_camera, world_lights, world_sky};

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn finite(c: Color) -> bool {
    [c.r, c.g, c.b, c.a]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
}

const UP: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const DOWN: Vec3 = Vec3 {
    x: 0.0,
    y: -1.0,
    z: 0.0,
};

#[test]
fn the_overworld_sky_is_the_unchanged_day_profile() {
    let state = world_free_fly_camera();
    assert_eq!(state.environment(), WorldEnvironmentProfile::DAY);
    assert_eq!(state.background(), world_background());
    let day = WorldEnvironmentProfile::DAY;
    assert_eq!(day.sky_zenith, Color::new(0.33, 0.56, 0.92, 1.0));
    assert_eq!(day.sky_horizon, Color::new(0.72, 0.85, 0.97, 1.0));
    assert_eq!(day.sky_haze, Color::new(0.80, 0.88, 0.95, 1.0));
    assert_eq!(world_sky().zenith, day.sky_zenith);
    assert_eq!(world_sky().up, UP);
    let up = world_background().color(UP);
    assert!(up.b > up.g && up.g > up.r);
}

#[test]
fn the_red_black_sky_is_dark_cornelian_but_readable() {
    let rb = WorldEnvironmentProfile::RED_BLACK;
    assert_ne!(rb, WorldEnvironmentProfile::DAY);
    for c in [rb.sky_zenith, rb.sky_horizon, rb.sky_haze] {
        assert!(finite(c));
        assert!(c.r > c.g && c.r > c.b * 1.5, "not cornelian: {c:?}");
        assert!(c.b > c.g, "wine, not orange: {c:?}");
        assert!(
            luminance(c) > 0.015 && luminance(c) < 0.25,
            "not dark-but-readable: {c:?}"
        );
        // No neon magenta, no pure red.
        assert!(c.r < 0.6 && c.b < 0.4 && c.g < 0.15);
    }
    assert!(luminance(rb.sky_horizon) > luminance(rb.sky_zenith));
    let sky = rb.sky(DOWN);
    assert_eq!(sky.color(DOWN), rb.sky_zenith);
    assert!(luminance(sky.color(Vec3::new(1.0, 0.0, 0.0))) > luminance(sky.color(DOWN)));
    // Sanity against the day sky: far darker and red-dominant.
    assert!(luminance(rb.sky_zenith) < luminance(WorldEnvironmentProfile::DAY.sky_zenith) * 0.2);
}

#[test]
fn the_midpoint_mixes_both_profiles() {
    let day = WorldEnvironmentProfile::DAY;
    let rb = WorldEnvironmentProfile::RED_BLACK;
    let mid = day.blend(&rb, 0.5);
    for (m, d, r) in [
        (mid.sky_zenith, day.sky_zenith, rb.sky_zenith),
        (mid.sky_horizon, day.sky_horizon, rb.sky_horizon),
        (mid.sky_haze, day.sky_haze, rb.sky_haze),
    ] {
        assert!((m.r - (d.r + r.r) / 2.0).abs() < 1e-5);
        assert!((m.g - (d.g + r.g) / 2.0).abs() < 1e-5);
        assert!((m.b - (d.b + r.b) / 2.0).abs() < 1e-5);
        assert!(finite(m));
    }
    assert!((mid.ambient_scale - (day.ambient_scale + rb.ambient_scale) / 2.0).abs() < 1e-5);
    assert_eq!(day.blend(&rb, 0.0), day);
    assert_eq!(day.blend(&rb, 1.0), rb);
    assert_eq!(day.blend(&rb, f32::NAN), day);
    assert_eq!(day.blend(&rb, 7.0), rb);
}

#[test]
fn the_forward_transition_goes_celeste_to_cornelian_in_sync_with_the_roll() {
    let mut state = world_free_fly_camera();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    let mut previous = luminance(state.background().color(state.sky_up()));
    let mut samples = 0;
    while state.transition().is_some() {
        state.advance_transition(PORTAL_TRANSITION_DURATION / 16.0);
        let env = state.environment();
        let eased = state.transition().map(|t| t.eased()).unwrap_or(1.0);
        assert_eq!(
            env,
            WorldEnvironmentProfile::DAY.blend(&WorldEnvironmentProfile::RED_BLACK, eased)
        );
        let now = luminance(state.background().color(state.sky_up()));
        assert!(
            now <= previous + 1e-4,
            "the zenith brightened mid-transition"
        );
        previous = now;
        samples += 1;
    }
    assert!(samples >= 15);
    assert_eq!(state.environment(), WorldEnvironmentProfile::RED_BLACK);
    assert_eq!(state.sky_up(), DOWN);
    // Hung along -Y: the darkest zenith is straight down in world terms.
    let bg = state.background();
    assert_eq!(
        bg.color(DOWN),
        WorldEnvironmentProfile::RED_BLACK.sky_zenith
    );
    assert!(luminance(bg.color(UP)) > luminance(bg.color(DOWN)));
}

#[test]
fn the_reverse_transition_goes_cornelian_to_celeste() {
    let mut state = world_free_fly_camera();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(10.0);
    assert_eq!(state.realm, WorldRealm::RedBlack);
    state.begin_transition(PortalCrossingEvent::RedBlackToOverworld);
    state.advance_transition(PORTAL_TRANSITION_DURATION / 2.0);
    let mid = state.environment();
    assert_eq!(
        mid,
        WorldEnvironmentProfile::RED_BLACK.blend(&WorldEnvironmentProfile::DAY, 0.5)
    );
    state.advance_transition(10.0);
    assert_eq!(state.environment(), WorldEnvironmentProfile::DAY);
    assert_eq!(state.sky_up(), UP);
    assert_eq!(state.background(), world_background());
}

#[test]
fn the_gradient_hangs_along_the_realm_up() {
    let day = WorldEnvironmentProfile::DAY;
    let normal = day.sky(UP);
    let flipped = day.sky(DOWN);
    assert_eq!(normal.color(UP), day.sky_zenith);
    assert_eq!(flipped.color(DOWN), day.sky_zenith);
    assert_eq!(normal.color(DOWN), day.sky_haze);
    assert_eq!(flipped.color(UP), day.sky_haze);
    let side = Vec3::new(0.0, 0.0, 1.0);
    assert_eq!(normal.color(side), flipped.color(side));
    // Mid-transition the sky's vertical leaves +Y: at the midpoint it is
    // the rolled view up, never a zero vector.
    let mut state = world_free_fly_camera();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(PORTAL_TRANSITION_DURATION / 2.0);
    let up = state.sky_up();
    assert!((up.length() - 1.0).abs() < 1e-4);
    assert!(up.y.abs() < 0.2, "{up:?}");
    for d in [UP, DOWN, side, Vec3::new(-0.3, 0.7, 0.2)] {
        assert!(finite(state.background().color(d)));
    }
}

#[test]
fn reset_returns_to_the_day_sky_and_the_catalog_background_is_untouched() {
    let mut state = world_free_fly_camera();
    state.begin_transition(PortalCrossingEvent::OverworldToRedBlack);
    state.advance_transition(0.3);
    apply_free_fly_input(
        &mut state,
        &FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert_eq!(state.environment(), WorldEnvironmentProfile::DAY);
    assert_eq!(state.background(), world_background());
    assert_eq!(gallery_background(), Color::new(0.05, 0.05, 0.08, 1.0));
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("Background::Solid(gallery_background())"));
    assert!(matches!(
        Background::Solid(gallery_background()),
        Background::Solid(_)
    ));
    // No new lights: still the sun plus the four bounded point lights.
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| matches!(l, Light::Point(_)))
            .count(),
        4
    );
}
