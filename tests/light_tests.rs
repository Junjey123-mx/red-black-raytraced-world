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
use scene::light::{DirectionalLight, Light};

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

    match light {
        Light::Directional(directional) => {
            assert!(approx_eq(directional.direction.length(), 1.0));
        }
    }
}
