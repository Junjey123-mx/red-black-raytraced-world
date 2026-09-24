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
    #[path = "../src/core/ray.rs"]
    pub mod ray;
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
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/light.rs"]
    pub mod light;
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

use camera::{Camera, primary_ray};
use core::color::Color;
use core::cube::Cube;
use core::face_textures::FaceTextures;
use core::material::Material;
use core::math::Vec3;
use core::ray::Ray;
use renderer::raytracer::cast_ray_lit;
use renderer::shading::DEFAULT_AMBIENT_FACTOR;
use renderer::texture_sampling::sample_nearest;
use scene::light::{DirectionalLight, Light, PointLight};
use scene::texture_manager::TextureManager;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn approx_color(a: Color, b: Color) -> bool {
    approx_eq(a.r, b.r) && approx_eq(a.g, b.g) && approx_eq(a.b, b.b)
}

const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
const MAGENTA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
const CYAN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

fn face_png_path(name: &str) -> String {
    format!(
        "{}/assets/textures/diagnostic/faces/{}.png",
        env!("CARGO_MANIFEST_DIR"),
        name
    )
}

/// Loads the six real diagnostic PNGs (the same assets `app.rs` uses) into
/// a fresh `TextureManager` and returns a `FaceTextures` built from them.
fn load_diagnostic_face_textures() -> (TextureManager, FaceTextures) {
    let mut manager = TextureManager::new();
    let positive_x = manager.load(face_png_path("positive_x")).unwrap();
    let negative_x = manager.load(face_png_path("negative_x")).unwrap();
    let positive_y = manager.load(face_png_path("positive_y")).unwrap();
    let negative_y = manager.load(face_png_path("negative_y")).unwrap();
    let positive_z = manager.load(face_png_path("positive_z")).unwrap();
    let negative_z = manager.load(face_png_path("negative_z")).unwrap();

    let face_textures = FaceTextures::new(
        positive_x, negative_x, positive_y, negative_y, positive_z, negative_z,
    );

    (manager, face_textures)
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// A manager with nothing loaded, for materials that never set
/// `face_textures` and therefore never query it.
fn no_textures() -> TextureManager {
    TextureManager::new()
}

// ---------------------------------------------------------------------
// 1. A uniform-albedo material continues producing the same result.
// ---------------------------------------------------------------------

#[test]
fn uniform_albedo_material_is_unaffected_by_texture_support() {
    let material = Material::matte(WHITE);
    let objects = [(unit_cube(), material)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    let color = cast_ray_lit(
        &objects,
        &ray,
        camera_position,
        &lights,
        0.0,
        Color::black(),
        &no_textures(),
    );

    // Identical to the pre-existing Category 3 contract
    // (`directional_diffuse_is_correct_through_full_pipeline`): front face
    // normal faces the light head-on, N.L = 1, diffuse = albedo.
    assert!(approx_eq(color.r, 1.0));
    assert!(approx_eq(color.g, 1.0));
    assert!(approx_eq(color.b, 1.0));
}

// ---------------------------------------------------------------------
// 2. A textured surface selects TextureId by Face, and 8. two faces with
//    different PNGs produce different albedos.
// ---------------------------------------------------------------------

#[test]
fn textured_surface_selects_texture_by_face_and_distinct_faces_differ() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];

    let ray_positive_x = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let ray_negative_x = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

    let color_positive_x = cast_ray_lit(
        &objects,
        &ray_positive_x,
        Vec3::new(5.0, 0.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );
    let color_negative_x = cast_ray_lit(
        &objects,
        &ray_negative_x,
        Vec3::new(-5.0, 0.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    assert!(
        approx_color(color_positive_x, RED),
        "got {color_positive_x:?}"
    );
    assert!(
        approx_color(color_negative_x, GREEN),
        "got {color_negative_x:?}"
    );
    assert!(!approx_color(color_positive_x, color_negative_x));
}

// ---------------------------------------------------------------------
// 3. hit.uv (not just hit.face) drives the sampled albedo.
// ---------------------------------------------------------------------

#[test]
fn shading_uses_the_real_hit_uv_not_just_the_face() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];

    // Center of +Z: dominant magenta.
    let center_ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let center_color = cast_ray_lit(
        &objects,
        &center_ray,
        Vec3::new(0.0, 0.0, 5.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );
    assert!(approx_color(center_color, MAGENTA), "got {center_color:?}");

    // +Z's marker is a white horizontal line at row 7, columns 1-4 of a
    // 16x16 texture. Row 7 -> v in [7/16, 8/16); column 2 -> u in
    // [2/16, 3/16). Convert to a world point on the +Z face using the
    // frozen 04P1 convention (u = lx, v = 1 - ly) and land inside that
    // marker instead of the dominant-color region.
    let u = 2.5 / 16.0;
    let v = 7.5 / 16.0;
    let lx = u;
    let ly = 1.0 - v;
    let x = -1.0 + lx * 2.0;
    let y = -1.0 + ly * 2.0;
    let marker_ray = Ray::new(Vec3::new(x, y, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let marker_color = cast_ray_lit(
        &objects,
        &marker_ray,
        Vec3::new(x, y, 5.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );
    assert!(
        approx_color(marker_color, WHITE),
        "expected the white marker, got {marker_color:?}"
    );
    assert!(!approx_color(center_color, marker_color));
}

// ---------------------------------------------------------------------
// 4/5. sample_nearest produces the albedo used by ambient and diffuse.
// ---------------------------------------------------------------------

#[test]
fn sampled_albedo_drives_ambient_contribution() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let texture_id = face_textures.texture_for_face(core::hit::Face::PositiveZ);
    let texture = manager.get(texture_id).unwrap();

    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];
    let ambient_factor = 0.3;

    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let expected_albedo = sample_nearest(texture, hit.uv);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 0.0, 5.0),
        &no_lights,
        ambient_factor,
        Color::black(),
        &manager,
    );

    assert!(approx_eq(color.r, expected_albedo.r * ambient_factor));
    assert!(approx_eq(color.g, expected_albedo.g * ambient_factor));
    assert!(approx_eq(color.b, expected_albedo.b * ambient_factor));
}

#[test]
fn sampled_albedo_drives_diffuse_contribution() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let texture_id = face_textures.texture_for_face(core::hit::Face::PositiveZ);
    let texture = manager.get(texture_id).unwrap();

    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];

    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let expected_albedo = sample_nearest(texture, hit.uv);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 0.0, 5.0),
        &lights,
        0.0,
        Color::black(),
        &manager,
    );

    // Normal faces the light head-on (N.L = 1): diffuse = albedo * light.
    assert!(approx_eq(color.r, expected_albedo.r));
    assert!(approx_eq(color.g, expected_albedo.g));
    assert!(approx_eq(color.b, expected_albedo.b));
}

// ---------------------------------------------------------------------
// 6. Specular still functions on a textured surface.
// ---------------------------------------------------------------------

#[test]
fn specular_still_applies_on_a_textured_surface() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let glossy_textured = Material::new(Color::black(), 2.0, 4.0).with_face_textures(face_textures);
    let objects = [(unit_cube(), glossy_textured)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    // View, light, and normal all aligned at the +Z center: specular is
    // near its peak there, same as the pre-existing (non-textured)
    // Category 3 contract in `lighting_contract_tests.rs`.
    let specular_only = renderer::shading::specular_directional(
        &Material::new(Color::white(), 2.0, 4.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        &DirectionalLight::new(Vec3::new(0.0, 0.0, 1.0), Color::white(), 1.0),
    );
    assert!(specular_only.r > 0.0);

    let color = cast_ray_lit(
        &objects,
        &ray,
        camera_position,
        &lights,
        0.0,
        Color::black(),
        &manager,
    );
    // Magenta diffuse (r=1,g=0,b=1) plus a real specular contribution: the
    // green channel (0 from diffuse) must be pushed above zero by specular.
    assert!(color.g > 0.0, "got {color:?}");
}

// ---------------------------------------------------------------------
// 7. A shadow still removes direct contribution but keeps ambient, on a
//    textured surface.
// ---------------------------------------------------------------------

#[test]
fn hard_shadow_keeps_ambient_on_a_textured_surface() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material = || Material::new(Color::black(), 1.0, 8.0).with_face_textures(face_textures);
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.6, 0.0, 0.8),
        Color::white(),
        1.0,
    ))];
    let ambient_factor = 0.2;
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let ray = Ray::new(camera_position, Vec3::new(0.0, 0.0, -1.0));

    let unblocked = cast_ray_lit(
        &[(unit_cube(), material())],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        Color::black(),
        &manager,
    );

    let occluder = Cube::new(Vec3::new(0.5, -2.0, 2.0), Vec3::new(2.5, 2.0, 3.0));
    let blocked = cast_ray_lit(
        &[
            (unit_cube(), material()),
            (occluder, Material::matte(Color::black())),
        ],
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        Color::black(),
        &manager,
    );

    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let texture_id = face_textures.texture_for_face(hit.face);
    let texture = manager.get(texture_id).unwrap();
    let sampled_albedo = sample_nearest(texture, hit.uv);
    let ambient_only = Color::new(
        sampled_albedo.r * ambient_factor,
        sampled_albedo.g * ambient_factor,
        sampled_albedo.b * ambient_factor,
        1.0,
    );

    assert!(unblocked.r > ambient_only.r + 0.05 || unblocked.b > ambient_only.b + 0.05);
    assert!(approx_eq(blocked.r, ambient_only.r));
    assert!(approx_eq(blocked.g, ambient_only.g));
    assert!(approx_eq(blocked.b, ambient_only.b));
}

// ---------------------------------------------------------------------
// 9. No file is read during an individual shading/raytracing call: the
//    manager is only ever passed as `&TextureManager` (shared reference),
//    and `TextureManager::load` requires `&mut self` — the borrow checker
//    makes calling `load` from inside `cast_ray_lit` a compile error, not
//    just a convention. This test exercises many repeated calls against
//    the same already-loaded manager and checks the result is exactly
//    stable, which would not hold if some hidden path re-touched storage.
// ---------------------------------------------------------------------

#[test]
fn repeated_shading_calls_never_reload_and_stay_deterministic() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let first = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 0.0, 5.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    for _ in 0..1000 {
        let repeated = cast_ray_lit(
            &objects,
            &ray,
            Vec3::new(0.0, 0.0, 5.0),
            &no_lights,
            1.0,
            Color::black(),
            &manager,
        );
        assert!(approx_color(first, repeated));
    }
}

// ---------------------------------------------------------------------
// 10. A miss continues returning background exactly as before, even with
//     a textured object present in the scene.
// ---------------------------------------------------------------------

#[test]
fn miss_returns_background_with_a_textured_object_present() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::white(),
        1.0,
    ))];
    let background = Color::new(0.2, 0.3, 0.4, 1.0);
    let ray = Ray::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(0.0, 0.0, -1.0));

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(10.0, 10.0, 10.0),
        &lights,
        0.1,
        background,
        &manager,
    );

    assert!(approx_eq(color.r, background.r));
    assert!(approx_eq(color.g, background.g));
    assert!(approx_eq(color.b, background.b));
}

// ---------------------------------------------------------------------
// Visual/logical contract: Camera -> primary_ray -> Cube -> HitRecord ->
// FaceTextures -> TextureManager -> CpuTexture -> sample_nearest ->
// shading, through the exact `cast_ray_lit` path `app.rs` uses.
// ---------------------------------------------------------------------

#[test]
fn camera_driven_render_shows_distinct_textures_on_two_visible_faces() {
    let (manager, face_textures) = load_diagnostic_face_textures();
    let material =
        Material::new(Color::new(0.9, 0.5, 0.2, 1.0), 3.0, 2.0).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
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

    // Camera positioned to see both the +Z (front) and +X (right) faces at
    // once, matching the spirit of `app.rs`'s diagnostic camera.
    let camera = Camera::new(
        Vec3::new(3.0, 0.0, 3.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );

    let width = 101;
    let height = 101;

    // A pixel column left of center looks toward +Z; a column right of
    // center looks toward +X, for this camera placement.
    let left_ray = primary_ray(&camera, width / 4, height / 2, width, height);
    let right_ray = primary_ray(&camera, 3 * width / 4, height / 2, width, height);

    let left_hit = unit_cube().intersect(&left_ray, 0.0, f32::INFINITY);
    let right_hit = unit_cube().intersect(&right_ray, 0.0, f32::INFINITY);
    assert!(left_hit.is_some());
    assert!(right_hit.is_some());
    assert_ne!(left_hit.unwrap().face, right_hit.unwrap().face);

    let left_color = cast_ray_lit(
        &objects,
        &left_ray,
        camera.position,
        &lights,
        DEFAULT_AMBIENT_FACTOR,
        background,
        &manager,
    );
    let right_color = cast_ray_lit(
        &objects,
        &right_ray,
        camera.position,
        &lights,
        DEFAULT_AMBIENT_FACTOR,
        background,
        &manager,
    );

    assert!(left_color.r.is_finite() && left_color.g.is_finite() && left_color.b.is_finite());
    assert!(right_color.r.is_finite() && right_color.g.is_finite() && right_color.b.is_finite());
    // Two different faces with two different dominant-color PNGs must not
    // shade to the same color.
    assert!(!approx_color(left_color, right_color));
    // Neither pixel is the flat background: both hit real, lit geometry.
    assert!(!approx_color(left_color, background));
    assert!(!approx_color(right_color, background));
}
