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
use core::cube::Cube;
use core::math::Vec3;
use core::ray::Ray;
use renderer::raytracer::cast_ray;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

#[test]
fn hit_produces_expected_diagnostic_color() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let background = Color::black();

    let color = cast_ray(&cube, &ray, background);

    // Normal at the hit is (0, 0, 1); remapped from [-1,1] to [0,1].
    assert!(approx_eq(color.r, 0.5));
    assert!(approx_eq(color.g, 0.5));
    assert!(approx_eq(color.b, 1.0));
    assert!(approx_eq(color.a, 1.0));
}

#[test]
fn miss_returns_background() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let background = Color::new(0.2, 0.3, 0.4, 1.0);

    let color = cast_ray(&cube, &ray, background);

    assert!(approx_eq(color.r, background.r));
    assert!(approx_eq(color.g, background.g));
    assert!(approx_eq(color.b, background.b));
    assert!(approx_eq(color.a, background.a));
}

#[test]
fn output_channels_are_finite_and_in_valid_range() {
    let cube = unit_cube();
    let rays = [
        Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
    ];

    for ray in rays {
        let color = cast_ray(&cube, &ray, Color::black());

        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
        assert!(color.a.is_finite());

        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
        assert!((0.0..=1.0).contains(&color.a));
    }
}

#[test]
fn different_faces_produce_different_diagnostic_colors() {
    let cube = unit_cube();

    let front = cast_ray(
        &cube,
        &Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Color::black(),
    );
    let side = cast_ray(
        &cube,
        &Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        Color::black(),
    );

    assert!(!approx_eq(front.r, side.r) || !approx_eq(front.b, side.b));
}
