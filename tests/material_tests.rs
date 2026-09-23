#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

use core::color::Color;
use core::material::Material;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn matte_material_has_zero_specular() {
    let material = Material::matte(Color::new(0.8, 0.2, 0.2, 1.0));
    assert!(approx_eq(material.specular, 0.0));
}

#[test]
fn glossy_material_has_positive_specular_and_high_shininess() {
    let material = Material::glossy(Color::white());
    assert!(material.specular > 0.0);
    assert!(material.shininess > 1.0);
}

#[test]
fn constructor_preserves_albedo() {
    let albedo = Color::new(0.1, 0.4, 0.9, 1.0);
    let material = Material::new(albedo, 0.5, 32.0);

    assert!(approx_eq(material.albedo.r, albedo.r));
    assert!(approx_eq(material.albedo.g, albedo.g));
    assert!(approx_eq(material.albedo.b, albedo.b));
}

#[test]
fn negative_specular_is_clamped_to_zero() {
    let material = Material::new(Color::black(), -5.0, 32.0);
    assert!(material.specular >= 0.0);
    assert!(approx_eq(material.specular, 0.0));
}

#[test]
fn shininess_is_clamped_to_a_valid_positive_range() {
    let too_low = Material::new(Color::black(), 0.5, -10.0);
    let too_high = Material::new(Color::black(), 0.5, 1_000_000.0);

    assert!(too_low.shininess >= 1.0);
    assert!(too_high.shininess.is_finite());
    assert!(too_high.shininess > 0.0);
}

#[test]
fn all_fields_are_finite() {
    let material = Material::new(Color::new(0.5, 0.5, 0.5, 1.0), 0.5, 64.0);

    assert!(material.albedo.r.is_finite());
    assert!(material.albedo.g.is_finite());
    assert!(material.albedo.b.is_finite());
    assert!(material.specular.is_finite());
    assert!(material.shininess.is_finite());
}
