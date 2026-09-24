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

    pub use camera::Camera;
    pub use projection::primary_ray;
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
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
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

use camera::{Camera, primary_ray};
use core::color::Color;
use core::cube::Cube;
use core::hit::Face;
use core::math::Vec3;
use renderer::framebuffer::Framebuffer;
use renderer::raytracer::cast_ray;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn scene_camera() -> Camera {
    Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    )
}

fn scene_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

#[test]
fn center_pixel_hits_cube_front_face() {
    let camera = scene_camera();
    let cube = scene_cube();
    let ray = primary_ray(&camera, 50, 50, 101, 101);

    let hit = cube
        .intersect(&ray, 0.0, f32::INFINITY)
        .expect("center pixel must hit the cube");

    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn known_periphery_pixel_misses_and_renders_background() {
    let camera = scene_camera();
    let cube = scene_cube();
    let background = Color::new(0.05, 0.05, 0.08, 1.0);

    // Far corner of a wide frame around a small centered cube: well outside
    // its projected silhouette.
    let ray = primary_ray(&camera, 0, 0, 101, 101);
    assert!(cube.intersect(&ray, 0.0, f32::INFINITY).is_none());

    let color = cast_ray(&cube, &ray, background);
    assert!(approx_eq(color.r, background.r));
    assert!(approx_eq(color.g, background.g));
    assert!(approx_eq(color.b, background.b));
}

#[test]
fn center_ray_direction_points_forward() {
    let camera = scene_camera();
    let ray = primary_ray(&camera, 50, 50, 101, 101);

    assert!(approx_eq(ray.direction.x, 0.0));
    assert!(approx_eq(ray.direction.y, 0.0));
    assert!(approx_eq(ray.direction.z, -1.0));
}

#[test]
fn hit_normal_is_unit_length() {
    let camera = scene_camera();
    let cube = scene_cube();
    let ray = primary_ray(&camera, 50, 50, 101, 101);

    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert!(approx_eq(hit.normal.length(), 1.0));
}

#[test]
fn framebuffer_dimensions_match_requested_size() {
    let framebuffer = Framebuffer::new(101, 101);
    assert_eq!(framebuffer.width(), 101);
    assert_eq!(framebuffer.height(), 101);
    assert_eq!(framebuffer.pixels().len(), 101 * 101);
}

#[test]
fn full_frame_render_produces_only_finite_colors() {
    let camera = scene_camera();
    let cube = scene_cube();
    let background = Color::new(0.05, 0.05, 0.08, 1.0);

    let width = 40;
    let height = 40;
    let mut framebuffer = Framebuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            let color = cast_ray(&cube, &ray, background);
            framebuffer.set_pixel(x, y, color);
        }
    }

    for pixel in framebuffer.pixels() {
        assert!(pixel.r.is_finite());
        assert!(pixel.g.is_finite());
        assert!(pixel.b.is_finite());
        assert!(pixel.a.is_finite());
        assert!(!pixel.r.is_nan());
        assert!(!pixel.g.is_nan());
        assert!(!pixel.b.is_nan());
    }

    // The centered cube must have produced at least one hit pixel distinct
    // from the background, proving the render is not an all-miss frame.
    let center = framebuffer
        .get_pixel(width / 2, height / 2)
        .expect("center pixel must exist");
    assert!(!approx_eq(center.b, background.b) || !approx_eq(center.r, background.r));
}
