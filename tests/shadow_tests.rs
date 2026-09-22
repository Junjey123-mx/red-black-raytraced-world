#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/light.rs"]
    pub mod light;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
}

use core::color::Color;
use core::cube::Cube;
use core::material::Material;
use core::math::Vec3;
use renderer::raytracer::cast_ray_lit;
use renderer::shadows::{
    DIRECTIONAL_SHADOW_RANGE, SHADOW_EPSILON, is_occluded, shadow_ray_to_directional_light,
    shadow_ray_to_point_light,
};
use scene::light::{DirectionalLight, Light, PointLight};

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn directional_shadow_ray_origin_is_offset_outward_along_normal() {
    let hit_point = Vec3::new(0.0, 0.0, 1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);

    let ray = shadow_ray_to_directional_light(hit_point, normal, &light);

    let offset = ray.origin - hit_point;
    assert!(approx_eq(offset.dot(normal), SHADOW_EPSILON));
}

#[test]
fn directional_shadow_ray_direction_matches_light_convention() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let light = DirectionalLight::new(Vec3::new(1.0, 0.0, 0.0), Color::white(), 1.0);

    let ray = shadow_ray_to_directional_light(hit_point, normal, &light);

    assert!(approx_eq(ray.direction.x, light.direction.x));
    assert!(approx_eq(ray.direction.y, light.direction.y));
    assert!(approx_eq(ray.direction.z, light.direction.z));
}

#[test]
fn point_shadow_ray_origin_is_offset_outward_along_normal() {
    let hit_point = Vec3::new(0.0, 0.0, 1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = PointLight::new(Vec3::new(0.0, 5.0, 5.0), Color::white(), 1.0);

    let (ray, _distance) = shadow_ray_to_point_light(hit_point, normal, &light);

    let offset = ray.origin - hit_point;
    assert!(approx_eq(offset.dot(normal), SHADOW_EPSILON));
}

#[test]
fn point_shadow_ray_distance_matches_offset_origin_to_light() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let light = PointLight::new(Vec3::new(0.0, 5.0, 0.0), Color::white(), 1.0);

    let (ray, distance) = shadow_ray_to_point_light(hit_point, normal, &light);

    let expected = (light.position - ray.origin).length();
    assert!(approx_eq(distance, expected));
    assert!(distance > 0.0);
    assert!(distance < 5.0 + SHADOW_EPSILON);
}

#[test]
fn point_shadow_ray_coincident_with_light_stays_finite() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let light = PointLight::new(
        Vec3::new(0.0, SHADOW_EPSILON * 0.1, 0.0),
        Color::white(),
        1.0,
    );

    let (ray, distance) = shadow_ray_to_point_light(hit_point, normal, &light);

    assert!(ray.direction.x.is_finite());
    assert!(ray.direction.y.is_finite());
    assert!(ray.direction.z.is_finite());
    assert!(distance.is_finite());
    assert!(distance >= 0.0);
}

#[test]
fn directional_shadow_range_is_effectively_unbounded_for_scene_scale() {
    assert!(DIRECTIONAL_SHADOW_RANGE > 1000.0);
    assert!(DIRECTIONAL_SHADOW_RANGE.is_finite());
}

/// Demonstrates the epsilon's purpose end to end: a shadow ray built from a
/// point exactly on a cube's own surface, cast away from that same cube,
/// must not immediately re-intersect it (no shadow acne).
#[test]
fn epsilon_offset_prevents_immediate_self_intersection_with_originating_cube() {
    let cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    let hit_point = Vec3::new(0.0, 0.0, 1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 0.0, 1.0), Color::white(), 1.0);

    let ray = shadow_ray_to_directional_light(hit_point, normal, &light);

    let self_hit = cube.intersect(&ray, 0.0, DIRECTIONAL_SHADOW_RANGE);
    assert!(self_hit.is_none());
}

/// A real, separated occluder in the light's path must still be detected
/// through the epsilon-offset shadow ray.
#[test]
fn epsilon_offset_still_detects_a_real_separated_occluder() {
    let occluder = Cube::new(Vec3::new(-1.0, -1.0, 4.0), Vec3::new(1.0, 1.0, 6.0));
    let hit_point = Vec3::new(0.0, 0.0, 1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 0.0, 1.0), Color::white(), 1.0);

    let ray = shadow_ray_to_directional_light(hit_point, normal, &light);

    let blocked = occluder.intersect(&ray, 0.0, DIRECTIONAL_SHADOW_RANGE);
    assert!(blocked.is_some());
}

#[test]
fn shadow_ray_construction_never_produces_nan() {
    let hit_point = Vec3::new(0.3, -0.2, 0.9);
    let normal = Vec3::new(0.1, 0.9, 0.2).normalize();
    let directional = DirectionalLight::new(Vec3::new(-1.0, 1.0, 0.5), Color::white(), 1.0);
    let point = PointLight::new(Vec3::new(2.0, 3.0, -4.0), Color::white(), 1.0);

    let directional_ray = shadow_ray_to_directional_light(hit_point, normal, &directional);
    assert!(!directional_ray.origin.x.is_nan());
    assert!(!directional_ray.direction.x.is_nan());

    let (point_ray, point_distance) = shadow_ray_to_point_light(hit_point, normal, &point);
    assert!(!point_ray.origin.x.is_nan());
    assert!(!point_ray.direction.x.is_nan());
    assert!(!point_distance.is_nan());
}

fn main_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// Placed along the shadow-ray path from the main cube's top face toward a
/// point light above and to the side, but nowhere near a straight-down
/// primary ray.
fn side_occluder_cube() -> Cube {
    Cube::new(Vec3::new(1.5, 1.3, -1.0), Vec3::new(3.5, 2.7, 1.0))
}

#[test]
fn is_occluded_true_when_blocker_lies_between_hit_and_point_light() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::white(), 1.0);
    let occluder = Cube::new(Vec3::new(-1.0, -1.0, 1.0), Vec3::new(1.0, 1.0, 2.0));

    let (shadow_ray, distance) = shadow_ray_to_point_light(hit_point, normal, &light);
    assert!(is_occluded(
        std::iter::once(&occluder),
        &shadow_ray,
        distance
    ));
}

#[test]
fn is_occluded_false_when_blocker_lies_behind_point_light() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::white(), 1.0);
    let occluder = Cube::new(Vec3::new(-1.0, -1.0, 5.0), Vec3::new(1.0, 1.0, 6.0));

    let (shadow_ray, distance) = shadow_ray_to_point_light(hit_point, normal, &light);
    assert!(!is_occluded(
        std::iter::once(&occluder),
        &shadow_ray,
        distance
    ));
}

#[test]
fn is_occluded_false_when_no_blocker_exists() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::white(), 1.0);

    let (shadow_ray, distance) = shadow_ray_to_point_light(hit_point, normal, &light);
    let no_objects: [Cube; 0] = [];
    assert!(!is_occluded(no_objects.iter(), &shadow_ray, distance));
}

#[test]
fn is_occluded_true_for_directional_blocker() {
    let hit_point = Vec3::zero();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 0.0, 1.0), Color::white(), 1.0);
    let occluder = Cube::new(Vec3::new(-1.0, -1.0, 4.0), Vec3::new(1.0, 1.0, 6.0));

    let shadow_ray = shadow_ray_to_directional_light(hit_point, normal, &light);
    assert!(is_occluded(
        std::iter::once(&occluder),
        &shadow_ray,
        DIRECTIONAL_SHADOW_RANGE
    ));
}

#[test]
fn cast_ray_lit_returns_background_on_miss() {
    let objects = [(main_cube(), Material::matte(Color::white()))];
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::white(),
        1.0,
    ))];
    let ray = core::ray::Ray::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
    let background = Color::new(0.2, 0.3, 0.4, 1.0);

    let color = cast_ray_lit(
        &objects,
        &ray,
        Vec3::new(10.0, 10.0, 10.0),
        &lights,
        0.1,
        background,
    );

    assert!(approx_eq(color.r, background.r));
    assert!(approx_eq(color.g, background.g));
    assert!(approx_eq(color.b, background.b));
}

#[test]
fn cast_ray_lit_ambient_persists_while_diffuse_and_specular_vanish_when_blocked() {
    let material = Material::glossy(Color::white());
    let camera_position = Vec3::new(0.0, 5.0, 0.0);
    let ray = core::ray::Ray::new(camera_position, Vec3::new(0.0, -1.0, 0.0));
    let lights = [Light::Point(PointLight::new(
        Vec3::new(5.0, 3.0, 0.0),
        Color::white(),
        1.0,
    ))];
    let ambient_factor = 0.1;
    let background = Color::black();

    let unblocked_objects = [(main_cube(), Material::glossy(Color::white()))];
    let unblocked = cast_ray_lit(
        &unblocked_objects,
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
    );

    let blocked_objects = [
        (main_cube(), Material::glossy(Color::white())),
        (side_occluder_cube(), Material::matte(Color::black())),
    ];
    let blocked = cast_ray_lit(
        &blocked_objects,
        &ray,
        camera_position,
        &lights,
        ambient_factor,
        background,
    );

    let ambient_only = renderer::shading::ambient(&material, ambient_factor);

    // Unblocked: diffuse/specular add real brightness above ambient alone.
    assert!(unblocked.r > ambient_only.r + 0.01);
    // Blocked: only the ambient term remains.
    assert!(approx_eq(blocked.r, ambient_only.r));
    assert!(approx_eq(blocked.g, ambient_only.g));
    assert!(approx_eq(blocked.b, ambient_only.b));
}
