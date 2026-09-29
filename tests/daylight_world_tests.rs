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
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
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

use core::color::Color;
use core::math::Vec3;
use renderer::skybox::Background;
use scene::light::Light;
use scene::material_gallery::gallery_background;
use scene::world::{
    WORLD_AMBIENT_FACTOR, sun_color, sun_direction, world_background, world_lights, world_sky,
};

fn finite(c: Color) -> bool {
    c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && c.a.is_finite()
}

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

#[test]
fn the_zenith_is_a_medium_celeste_blue() {
    let up = world_sky().color(Vec3::new(0.0, 1.0, 0.0));
    assert!(up.b > up.g && up.g > up.r, "{up:?}");
    assert!(up.b > 0.7, "{up:?}");
    assert!((0.3..0.7).contains(&luminance(up)), "{up:?}");
    assert_eq!(up, world_sky().zenith);
}

#[test]
fn the_horizon_is_lighter_than_the_zenith() {
    let sky = world_sky();
    let zenith = sky.color(Vec3::new(0.0, 1.0, 0.0));
    let horizon = sky.color(Vec3::new(1.0, 0.0, 0.0));
    assert!(luminance(horizon) > luminance(zenith) + 0.15);
    assert!(
        horizon.b >= horizon.g && horizon.g >= horizon.r,
        "{horizon:?}"
    );
    // Intermediate elevations blend monotonically from horizon to zenith.
    let mut previous = luminance(horizon);
    for i in 1..=10 {
        let e = i as f32 / 10.0;
        let c = sky.color(Vec3::new((1.0 - e * e).sqrt(), e, 0.0));
        assert!(luminance(c) <= previous + 1e-4, "elevation {e}: {c:?}");
        previous = luminance(c);
    }
}

#[test]
fn below_the_horizon_there_is_pale_haze_not_darkness() {
    let sky = world_sky();
    let down = sky.color(Vec3::new(0.3, -0.8, 0.2));
    assert!(luminance(down) > 0.7, "{down:?}");
    assert!(down.b >= down.r, "{down:?}");
}

#[test]
fn the_sky_is_never_black_or_red_black() {
    let sky = world_sky();
    for i in 0..64 {
        let a = i as f32 * 0.1;
        for j in -8..=8 {
            let d = Vec3::new(a.cos(), j as f32 / 8.0, a.sin());
            let c = sky.color(d);
            assert!(finite(c));
            assert!(luminance(c) > 0.3, "dark sky at {d:?}: {c:?}");
            assert!(c.b > c.r, "reddish sky at {d:?}: {c:?}");
            assert!(c.r < 0.9, "washed-out sky at {d:?}: {c:?}");
        }
    }
    // Degenerate direction: still a finite sky color.
    assert!(finite(sky.color(Vec3::zero())));
}

#[test]
fn the_world_background_is_the_directional_sky() {
    let background = world_background();
    assert!(background.is_sky());
    assert_eq!(
        background.color(Vec3::new(0.0, 1.0, 0.0)),
        world_sky().zenith
    );
    assert_ne!(
        background.color(Vec3::new(0.0, 1.0, 0.0)),
        background.color(Vec3::new(1.0, 0.0, 0.0))
    );
}

#[test]
fn the_catalog_keeps_its_flat_studio_background() {
    let catalog = Background::Solid(gallery_background());
    assert!(!catalog.is_sky());
    for d in [
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
    ] {
        assert_eq!(catalog.color(d), gallery_background());
    }
    // The studio backdrop is dark; the world sky is not.
    assert!(luminance(gallery_background()) < 0.1);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("Background::Solid(gallery_background())"));
    assert!(app.contains("world_background()"));
}

#[test]
fn a_sun_like_directional_light_comes_diagonally_from_above() {
    let lights = world_lights();
    let suns: Vec<_> = lights
        .iter()
        .filter_map(|l| match l {
            Light::Directional(d) => Some(d),
            Light::Point(_) => None,
        })
        .collect();
    assert_eq!(suns.len(), 1);
    let sun = suns[0];
    assert!(sun.direction.y > 0.6, "{:?}", sun.direction);
    assert!(
        sun.direction.x.abs() > 0.2 && sun.direction.z.abs() > 0.2,
        "{:?}",
        sun.direction
    );
    assert!((sun.direction.length() - 1.0).abs() < 1e-4);
    assert_eq!(sun.direction, sun_direction().normalize());
    assert!(sun.intensity >= 0.9 && sun.intensity <= 1.2);
    // Almost white, faintly warm.
    assert_eq!(sun.color, sun_color());
    assert!(sun.color.r >= sun.color.g && sun.color.g >= sun.color.b);
    assert!(sun.color.b > 0.85);
}

#[test]
fn the_ambient_is_moderate_so_shadows_stay_readable() {
    assert!(WORLD_AMBIENT_FACTOR >= 0.2 && WORLD_AMBIENT_FACTOR <= 0.4);
}
