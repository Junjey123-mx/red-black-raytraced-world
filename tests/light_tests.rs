#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/color.rs"]
    pub mod color;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/light.rs"]
    pub mod light;
}

use core::color::Color;
use core::math::Vec3;
use scene::light::{DirectionalLight, Light, PointLight};

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn direction_is_normalized_on_construction() {
    let light = DirectionalLight::new(Vec3::new(0.0, 3.0, 4.0), Color::white(), 1.0);
    assert!(approx_eq(light.direction.length(), 1.0));
}

/// Documents and locks the fixed convention: `direction` already points
/// from the surface toward the light, so a light "toward +Y" keeps a
/// positive Y direction component (no implicit negation anywhere).
#[test]
fn direction_points_toward_the_light_not_the_travel_direction() {
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    assert!(approx_eq(light.direction.x, 0.0));
    assert!(approx_eq(light.direction.y, 1.0));
    assert!(approx_eq(light.direction.z, 0.0));
}

#[test]
fn color_and_intensity_are_preserved() {
    let color = Color::new(1.0, 0.9, 0.8, 1.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), color, 2.5);

    assert!(approx_eq(light.color.r, color.r));
    assert!(approx_eq(light.color.g, color.g));
    assert!(approx_eq(light.color.b, color.b));
    assert!(approx_eq(light.intensity, 2.5));
}

#[test]
fn negative_intensity_is_clamped_to_zero() {
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), -5.0);
    assert!(light.intensity >= 0.0);
    assert!(approx_eq(light.intensity, 0.0));
}

#[test]
fn zero_direction_input_stays_finite() {
    let light = DirectionalLight::new(Vec3::zero(), Color::white(), 1.0);
    assert!(light.direction.x.is_finite());
    assert!(light.direction.y.is_finite());
    assert!(light.direction.z.is_finite());
}

#[test]
fn light_enum_wraps_directional_variant() {
    let light = Light::Directional(DirectionalLight::new(
        Vec3::new(1.0, 0.0, 0.0),
        Color::white(),
        1.0,
    ));

    if let Light::Directional(directional) = light {
        assert!(approx_eq(directional.direction.length(), 1.0));
    } else {
        panic!("expected Light::Directional");
    }
}

#[test]
fn point_light_preserves_position() {
    let position = Vec3::new(2.0, 3.0, -1.0);
    let light = PointLight::new(position, Color::white(), 1.0);

    assert!(approx_eq(light.position.x, position.x));
    assert!(approx_eq(light.position.y, position.y));
    assert!(approx_eq(light.position.z, position.z));
}

#[test]
fn point_light_preserves_color_and_intensity() {
    let color = Color::new(0.9, 0.7, 0.5, 1.0);
    let light = PointLight::new(Vec3::zero(), color, 3.0);

    assert!(approx_eq(light.color.r, color.r));
    assert!(approx_eq(light.color.g, color.g));
    assert!(approx_eq(light.color.b, color.b));
    assert!(approx_eq(light.intensity, 3.0));
}

#[test]
fn point_light_negative_intensity_is_clamped_to_zero() {
    let light = PointLight::new(Vec3::zero(), Color::white(), -2.0);
    assert!(light.intensity >= 0.0);
    assert!(approx_eq(light.intensity, 0.0));
}

#[test]
fn hit_to_point_light_vector_is_finite_and_has_valid_distance() {
    let light = PointLight::new(Vec3::new(4.0, 3.0, 0.0), Color::white(), 1.0);
    let hit_point = Vec3::zero();

    let to_light = light.position - hit_point;
    assert!(to_light.x.is_finite());
    assert!(to_light.y.is_finite());
    assert!(to_light.z.is_finite());

    let distance = to_light.length();
    assert!(distance.is_finite());
    assert!(distance >= 0.0);
    assert!(approx_eq(distance, 5.0));
}

#[test]
fn light_enum_wraps_point_variant() {
    let light = Light::Point(PointLight::new(
        Vec3::new(1.0, 1.0, 1.0),
        Color::white(),
        2.0,
    ));

    if let Light::Point(point) = light {
        assert!(approx_eq(point.intensity, 2.0));
    } else {
        panic!("expected Light::Point");
    }
}
