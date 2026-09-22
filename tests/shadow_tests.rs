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
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
}

use core::color::Color;
use core::cube::Cube;
use core::math::Vec3;
use renderer::shadows::{
    DIRECTIONAL_SHADOW_RANGE, SHADOW_EPSILON, shadow_ray_to_directional_light,
    shadow_ray_to_point_light,
};
use scene::light::{DirectionalLight, PointLight};

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
