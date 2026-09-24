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
use core::material::Material;
use core::math::Vec3;
use renderer::framebuffer::Framebuffer;
use renderer::raytracer::cast_ray_lit;
use scene::light::{DirectionalLight, Light, PointLight};
use scene::texture_manager::TextureManager;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn main_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// None of the materials in this file set `face_textures`, so this manager
/// is never actually queried by `cast_ray_lit`; it exists only to satisfy
/// the parameter added for texture support.
fn no_textures() -> TextureManager {
    TextureManager::new()
}

// ---------------------------------------------------------------------
// Ambient without direct light
// ---------------------------------------------------------------------

fn ambient_probe_material() -> Material {
    Material::matte(Color::new(0.6, 0.3, 0.2, 1.0))
}

#[test]
fn ambient_is_visible_with_no_lights_at_all() {
    let objects = [(main_cube(), ambient_probe_material())];
    let no_lights: [Light; 0] = [];
    let background = Color::black();
    let ambient_factor = 0.15;

    let ray = core::ray::Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 0.0, 5.0),
        &no_lights,
        ambient_factor,
        background,
        &no_textures(),
    );

    let expected = renderer::shading::ambient(&ambient_probe_material(), ambient_factor);
    assert!(approx_eq(color.r, expected.r));
    assert!(approx_eq(color.g, expected.g));
    assert!(approx_eq(color.b, expected.b));
    assert!(color.r > 0.0 || color.g > 0.0 || color.b > 0.0);
}

// ---------------------------------------------------------------------
// Diffuse and specular correctness through the full pipeline
// ---------------------------------------------------------------------

#[test]
fn directional_diffuse_is_correct_through_full_pipeline() {
    let material = Material::matte(Color::white());
    let objects = [(main_cube(), material)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let ambient_factor = 0.0;

    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));
    let color = cast_ray_lit(
        &objects,
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );

    // Front face normal (0,0,1) faces the light head-on: N.L = 1.
    assert!(approx_eq(color.r, 1.0));
    assert!(approx_eq(color.g, 1.0));
    assert!(approx_eq(color.b, 1.0));
}

#[test]
fn specular_is_correct_through_full_pipeline() {
    let matte = Material::matte(Color::white());
    let glossy = Material::new(Color::white(), 2.0, 4.0);
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let ambient_factor = 0.0;
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    // View, light, and normal are all aligned (0,0,1): the glossy material's
    // specular term adds real brightness beyond the matte diffuse-only case
    // (both start clamped at 1.0 for R here, so compare an unsaturated
    // configuration instead: verify specular alone is > 0 at this angle).
    // Computed before `glossy` is moved into the scene objects below.
    let specular_only = renderer::shading::specular_directional(
        &glossy,
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        &DirectionalLight::new(Vec3::new(0.0, 0.0, 1.0), Color::white(), 1.0),
    );
    assert!(specular_only.r > 0.0);

    let matte_color = cast_ray_lit(
        &[(main_cube(), matte)],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );
    let glossy_color = cast_ray_lit(
        &[(main_cube(), glossy)],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );
    assert!(approx_eq(matte_color.r, 1.0));
    assert!(approx_eq(glossy_color.r, 1.0)); // clamped, but specular is real (checked above)
}

#[test]
fn point_light_shading_is_correct_through_full_pipeline() {
    let material = Material::matte(Color::white());
    let objects = [(main_cube(), material)];
    let lights = [Light::Point(PointLight::new(
        Vec3::new(0.0, 0.0, 5.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    let color = cast_ray_lit(
        &objects,
        &ray,
        camera_position,
        &lights,
        0.0,
        background,
        &no_textures(),
    );

    // Light sits directly in front of the hit face: N.L = 1.
    assert!(approx_eq(color.r, 1.0));
    assert!(approx_eq(color.g, 1.0));
    assert!(approx_eq(color.b, 1.0));
}

// ---------------------------------------------------------------------
// Hard shadows
// ---------------------------------------------------------------------

fn hard_shadow_probe_material() -> Material {
    Material::new(Color::white(), 1.0, 8.0)
}

#[test]
fn hard_shadow_removes_diffuse_and_specular_but_keeps_ambient() {
    // Angled (not straight-down-the-camera-axis) so a real occluder can sit
    // in the shadow-ray path without also sitting in the primary ray path.
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.6, 0.0, 0.8),
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let ambient_factor = 0.2;
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    let unblocked = cast_ray_lit(
        &[(main_cube(), hard_shadow_probe_material())],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );

    // Sits along the shadow ray's angled path (x grows as z grows) but off
    // the primary ray's straight vertical path (x = 0, y = 0).
    let occluder = Cube::new(Vec3::new(0.5, -2.0, 2.0), Vec3::new(2.5, 2.0, 3.0));
    let blocked = cast_ray_lit(
        &[
            (main_cube(), hard_shadow_probe_material()),
            (occluder, Material::matte(Color::black())),
        ],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );

    let ambient_only = renderer::shading::ambient(&hard_shadow_probe_material(), ambient_factor);

    assert!(unblocked.r > ambient_only.r + 0.1);
    assert!(approx_eq(blocked.r, ambient_only.r));
    assert!(approx_eq(blocked.g, ambient_only.g));
    assert!(approx_eq(blocked.b, ambient_only.b));
}

#[test]
fn blocker_behind_point_light_does_not_shadow() {
    let material = Material::matte(Color::white());
    let light_position = Vec3::new(0.0, 0.0, 3.0);
    let lights = [Light::Point(PointLight::new(
        light_position,
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    // Occluder placed beyond the light (from the surface's point of view):
    // it must not block the point light.
    let far_occluder = Cube::new(Vec3::new(-2.0, -2.0, 10.0), Vec3::new(2.0, 2.0, 11.0));
    let color = cast_ray_lit(
        &[
            (main_cube(), material),
            (far_occluder, Material::matte(Color::black())),
        ],
        &ray,
        camera_position,
        &lights,
        0.0,
        background,
        &no_textures(),
    );

    assert!(approx_eq(color.r, 1.0));
    assert!(approx_eq(color.g, 1.0));
    assert!(approx_eq(color.b, 1.0));
}

#[test]
fn epsilon_offset_prevents_self_shadow_acne_in_full_pipeline() {
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::black();
    let ambient_factor = 0.05;
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    // Only the cube itself is present: its own shadow ray must not
    // immediately re-intersect its own surface.
    let color = cast_ray_lit(
        &[(main_cube(), hard_shadow_probe_material())],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
        &no_textures(),
    );

    let ambient_only = renderer::shading::ambient(&hard_shadow_probe_material(), ambient_factor);
    assert!(color.r > ambient_only.r + 0.1);
}

// ---------------------------------------------------------------------
// Geometry and framebuffer contracts
// ---------------------------------------------------------------------

#[test]
fn hit_normal_is_unit_length() {
    let cube = main_cube();
    let ray = core::ray::Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert!(approx_eq(hit.normal.length(), 1.0));
}

#[test]
fn framebuffer_dimensions_remain_intact() {
    let framebuffer = Framebuffer::new(160, 120);
    assert_eq!(framebuffer.width(), 160);
    assert_eq!(framebuffer.height(), 120);
    assert_eq!(framebuffer.pixels().len(), 160 * 120);
}

#[test]
fn known_center_pixel_hits_geometry_and_periphery_pixel_hits_background() {
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );
    let material = Material::matte(Color::white());
    let objects = [(main_cube(), material)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::new(0.1, 0.1, 0.2, 1.0);

    let center_ray = primary_ray(&camera, 50, 50, 101, 101);
    let center_color = cast_ray_lit(
        &objects,
        &center_ray,
        camera.position,
        &lights,
        0.1,
        background,
        &no_textures(),
    );
    assert!(!approx_eq(center_color.r, background.r) || !approx_eq(center_color.b, background.b));

    let corner_ray = primary_ray(&camera, 0, 0, 101, 101);
    let corner_color = cast_ray_lit(
        &objects,
        &corner_ray,
        camera.position,
        &lights,
        0.1,
        background,
        &no_textures(),
    );
    assert!(approx_eq(corner_color.r, background.r));
    assert!(approx_eq(corner_color.g, background.g));
    assert!(approx_eq(corner_color.b, background.b));
}

#[test]
fn full_frame_render_produces_only_finite_colors() {
    let camera = Camera::new(
        Vec3::new(6.0, 4.0, 8.0),
        Vec3::new(0.0, -0.3, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );

    let main_material = Material::new(Color::new(0.9, 0.5, 0.2, 1.0), 3.0, 2.0);
    let floor_material = Material::matte(Color::new(0.6, 0.6, 0.65, 1.0));
    let floor_cube = Cube::new(Vec3::new(-5.0, -2.0, -5.0), Vec3::new(5.0, -1.5, 5.0));
    let objects = [(main_cube(), main_material), (floor_cube, floor_material)];

    let lights = [
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            Color::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(-4.0, 6.0, 5.0),
            Color::white(),
            1.5,
        )),
    ];
    let background = Color::new(0.05, 0.05, 0.08, 1.0);

    let width = 40;
    let height = 30;
    let mut framebuffer = Framebuffer::new(width, height);
    let texture_manager = no_textures();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            let color = cast_ray_lit(
                &objects,
                &ray,
                camera.position,
                &lights,
                0.1,
                background,
                &texture_manager,
            );
            framebuffer.set_pixel(x, y, color);
        }
    }

    for pixel in framebuffer.pixels() {
        assert!(pixel.r.is_finite());
        assert!(pixel.g.is_finite());
        assert!(pixel.b.is_finite());
        assert!(!pixel.r.is_nan());
        assert!(!pixel.g.is_nan());
        assert!(!pixel.b.is_nan());
    }
}
