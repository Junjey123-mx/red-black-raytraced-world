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
use core::face_textures::FaceTextures;
use core::hit::Face;
use core::material::Material;
use core::math::Vec3;
use core::ray::Ray;
use core::texture::CpuTexture;
use renderer::raytracer::cast_ray_lit;
use scene::light::{DirectionalLight, Light};
use scene::texture_manager::TextureManager;

fn grass_png_path(name: &str) -> String {
    format!(
        "{}/assets/textures/overworld/grass/{}.png",
        env!("CARGO_MANIFEST_DIR"),
        name
    )
}

/// Loads the three real Grass Block PNGs (the same assets `app.rs` uses)
/// into a fresh `TextureManager`, loading `side` exactly once and reusing
/// its `TextureId` for all four lateral faces.
fn load_grass() -> (TextureManager, FaceTextures) {
    let mut manager = TextureManager::new();
    let top = manager.load(grass_png_path("top")).unwrap();
    let side = manager.load(grass_png_path("side")).unwrap();
    let bottom = manager.load(grass_png_path("bottom")).unwrap();

    let face_textures = FaceTextures::new(side, side, top, bottom, side, side);
    (manager, face_textures)
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// Structural (not pixel-exact) green classifier matching the same
/// heuristic used to build the palette from the reference image: the green
/// channel at or above the red channel reads as "grass-family".
fn is_greenish(c: Color) -> bool {
    c.g >= c.r
}

fn green_fraction(texture: &CpuTexture) -> f32 {
    let total = texture.width() * texture.height();
    let green = texture.pixels().iter().filter(|c| is_greenish(**c)).count();
    green as f32 / total as f32
}

// ---------------------------------------------------------------------
// 1-6. The three PNGs exist, load, have the expected dimensions, and
// produce distinct TextureIds.
// ---------------------------------------------------------------------

#[test]
fn top_png_exists_and_loads() {
    let mut manager = TextureManager::new();
    assert!(manager.load(grass_png_path("top")).is_ok());
}

#[test]
fn side_png_exists_and_loads() {
    let mut manager = TextureManager::new();
    assert!(manager.load(grass_png_path("side")).is_ok());
}

#[test]
fn bottom_png_exists_and_loads() {
    let mut manager = TextureManager::new();
    assert!(manager.load(grass_png_path("bottom")).is_ok());
}

#[test]
fn all_three_grass_textures_are_sixteen_by_sixteen() {
    let (manager, face_textures) = load_grass();
    for face in [Face::PositiveY, Face::NegativeY, Face::PositiveX] {
        let id = face_textures.texture_for_face(face);
        let texture = manager.get(id).unwrap();
        assert_eq!(texture.width(), 16);
        assert_eq!(texture.height(), 16);
    }
}

#[test]
fn all_three_grass_textures_produce_valid_ids() {
    let (manager, face_textures) = load_grass();
    for face in [Face::PositiveY, Face::NegativeY, Face::PositiveX] {
        let id = face_textures.texture_for_face(face);
        assert!(manager.get(id).is_some());
    }
}

#[test]
fn top_side_and_bottom_are_three_distinct_textures() {
    let (_manager, face_textures) = load_grass();
    let top = face_textures.texture_for_face(Face::PositiveY);
    let side = face_textures.texture_for_face(Face::PositiveX);
    let bottom = face_textures.texture_for_face(Face::NegativeY);

    assert_ne!(top, side);
    assert_ne!(top, bottom);
    assert_ne!(side, bottom);
}

// ---------------------------------------------------------------------
// 7. The four lateral faces resolve to the exact same `side` TextureId.
// ---------------------------------------------------------------------

#[test]
fn all_four_lateral_faces_share_the_same_side_texture_id() {
    let (_manager, face_textures) = load_grass();
    let positive_x = face_textures.texture_for_face(Face::PositiveX);
    let negative_x = face_textures.texture_for_face(Face::NegativeX);
    let positive_z = face_textures.texture_for_face(Face::PositiveZ);
    let negative_z = face_textures.texture_for_face(Face::NegativeZ);

    assert_eq!(positive_x, negative_x);
    assert_eq!(positive_x, positive_z);
    assert_eq!(positive_x, negative_z);
}

#[test]
fn only_three_textures_are_stored_despite_six_faces() {
    // side is loaded once and shared four ways: the manager must hold
    // exactly 3 textures (top, side, bottom), not 6.
    let (manager, _face_textures) = load_grass();
    assert_eq!(manager.len(), 3);
}

// ---------------------------------------------------------------------
// 8-9. Face -> asset mapping.
// ---------------------------------------------------------------------

#[test]
fn positive_y_resolves_to_top() {
    let (manager, face_textures) = load_grass();
    let mut fresh = TextureManager::new();
    let expected_top = fresh.load(grass_png_path("top")).unwrap();

    let resolved = face_textures.texture_for_face(Face::PositiveY);
    // Compare underlying pixel data rather than raw ids (ids are scoped to
    // the manager that minted them): both must be the top texture.
    let resolved_texture = manager.get(resolved).unwrap();
    let expected_texture = fresh.get(expected_top).unwrap();
    assert_eq!(resolved_texture.pixels(), expected_texture.pixels());
}

#[test]
fn negative_y_resolves_to_bottom() {
    let (manager, face_textures) = load_grass();
    let mut fresh = TextureManager::new();
    let expected_bottom = fresh.load(grass_png_path("bottom")).unwrap();

    let resolved = face_textures.texture_for_face(Face::NegativeY);
    let resolved_texture = manager.get(resolved).unwrap();
    let expected_texture = fresh.get(expected_bottom).unwrap();
    assert_eq!(resolved_texture.pixels(), expected_texture.pixels());
}

// ---------------------------------------------------------------------
// 10-12. Real hits through a real Cube sample the correct asset.
// ---------------------------------------------------------------------

#[test]
fn real_hit_on_positive_y_samples_top() {
    let (manager, face_textures) = load_grass();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];

    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_eq!(hit.face, Face::PositiveY);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 5.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    let top_id = face_textures.texture_for_face(Face::PositiveY);
    let top_texture = manager.get(top_id).unwrap();
    let expected = renderer::texture_sampling::sample_nearest(top_texture, hit.uv);
    assert_eq!(color, expected);
}

#[test]
fn real_hit_on_a_lateral_face_samples_side() {
    let (manager, face_textures) = load_grass();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];

    let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_eq!(hit.face, Face::PositiveX);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(5.0, 0.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    let side_id = face_textures.texture_for_face(Face::PositiveX);
    let side_texture = manager.get(side_id).unwrap();
    let expected = renderer::texture_sampling::sample_nearest(side_texture, hit.uv);
    assert_eq!(color, expected);
}

#[test]
fn real_hit_on_negative_y_samples_bottom() {
    let (manager, face_textures) = load_grass();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];

    let ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let hit = unit_cube().intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_eq!(hit.face, Face::NegativeY);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, -5.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    let bottom_id = face_textures.texture_for_face(Face::NegativeY);
    let bottom_texture = manager.get(bottom_id).unwrap();
    let expected = renderer::texture_sampling::sample_nearest(bottom_texture, hit.uv);
    assert_eq!(color, expected);
}

// ---------------------------------------------------------------------
// 13-15. Structural palette contracts (not pixel-exact).
// ---------------------------------------------------------------------

#[test]
fn top_is_predominantly_green_family() {
    let mut manager = TextureManager::new();
    let id = manager.load(grass_png_path("top")).unwrap();
    let texture = manager.get(id).unwrap();

    assert!(
        green_fraction(texture) > 0.9,
        "top.png should be almost entirely grass-family green"
    );
}

#[test]
fn side_contains_both_green_and_brown_components() {
    let mut manager = TextureManager::new();
    let id = manager.load(grass_png_path("side")).unwrap();
    let texture = manager.get(id).unwrap();
    let fraction = green_fraction(texture);

    assert!(fraction > 0.0, "side.png must contain some grass fringe");
    assert!(
        fraction < 0.5,
        "side.png must be dirt-dominant, not majority green"
    );
}

#[test]
fn bottom_does_not_contain_sides_green_stripe() {
    let mut manager = TextureManager::new();
    let id = manager.load(grass_png_path("bottom")).unwrap();
    let texture = manager.get(id).unwrap();

    assert!(
        green_fraction(texture) < 0.02,
        "bottom.png must not carry side's characteristic green fringe"
    );
}

// ---------------------------------------------------------------------
// 16. The textured pipeline still applies lighting and hard shadows.
// ---------------------------------------------------------------------

#[test]
fn grass_material_still_receives_lighting_and_hard_shadows() {
    let (manager, face_textures) = load_grass();
    let material = || Material::new(Color::black(), 0.04, 8.0).with_face_textures(face_textures);
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
    let side_id = face_textures.texture_for_face(hit.face);
    let side_texture = manager.get(side_id).unwrap();
    let sampled_albedo = renderer::texture_sampling::sample_nearest(side_texture, hit.uv);
    let ambient_only = Color::new(
        sampled_albedo.r * ambient_factor,
        sampled_albedo.g * ambient_factor,
        sampled_albedo.b * ambient_factor,
        1.0,
    );

    assert!(unblocked.r > ambient_only.r + 0.01 || unblocked.g > ambient_only.g + 0.01);
    assert!((blocked.r - ambient_only.r).abs() < 1e-4);
    assert!((blocked.g - ambient_only.g).abs() < 1e-4);
    assert!((blocked.b - ambient_only.b).abs() < 1e-4);
}

// ---------------------------------------------------------------------
// 17. Existing uniform/diagnostic materials keep working with a
// grass-loaded TextureManager present in the same call.
// ---------------------------------------------------------------------

#[test]
fn uniform_material_is_unaffected_by_a_grass_loaded_manager() {
    let (manager, _unused_face_textures) = load_grass();
    let uniform_material = Material::matte(Color::white());
    let objects = [(unit_cube(), uniform_material)];
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
        &manager,
    );

    assert!((color.r - 1.0).abs() < 1e-4);
    assert!((color.g - 1.0).abs() < 1e-4);
    assert!((color.b - 1.0).abs() < 1e-4);
}

// ---------------------------------------------------------------------
// 18. No file is read within the pixel loop: repeated calls against the
// same already-loaded manager stay perfectly stable.
// ---------------------------------------------------------------------

#[test]
fn repeated_grass_shading_calls_stay_deterministic() {
    let (manager, face_textures) = load_grass();
    let material = Material::matte(Color::black()).with_face_textures(face_textures);
    let objects = [(unit_cube(), material)];
    let no_lights: [Light; 0] = [];
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));

    let first = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(0.0, 5.0, 0.0),
        &no_lights,
        1.0,
        Color::black(),
        &manager,
    );

    for _ in 0..500 {
        let repeated = cast_ray_lit(
            &objects,
            &ray,
            Vec3::new(0.0, 5.0, 0.0),
            &no_lights,
            1.0,
            Color::black(),
            &manager,
        );
        assert_eq!(first, repeated);
    }
}
