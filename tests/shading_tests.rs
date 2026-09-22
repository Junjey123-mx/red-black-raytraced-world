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
    #[path = "../src/core/material.rs"]
    pub mod material;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/light.rs"]
    pub mod light;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
}

use core::color::Color;
use core::material::Material;
use core::math::Vec3;
use renderer::shading::{
    ambient, diffuse_directional, diffuse_point, specular_directional, specular_point,
};
use scene::light::{DirectionalLight, PointLight};

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn zero_ambient_factor_produces_black() {
    let material = Material::matte(Color::new(0.8, 0.3, 0.1, 1.0));
    let color = ambient(&material, 0.0);

    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn full_ambient_factor_matches_albedo() {
    let albedo = Color::new(0.6, 0.4, 0.2, 1.0);
    let material = Material::matte(albedo);
    let color = ambient(&material, 1.0);

    assert!(approx_eq(color.r, albedo.r));
    assert!(approx_eq(color.g, albedo.g));
    assert!(approx_eq(color.b, albedo.b));
}

#[test]
fn partial_ambient_factor_scales_albedo() {
    let albedo = Color::new(0.8, 0.8, 0.8, 1.0);
    let material = Material::matte(albedo);
    let color = ambient(&material, 0.5);

    assert!(approx_eq(color.r, 0.4));
    assert!(approx_eq(color.g, 0.4));
    assert!(approx_eq(color.b, 0.4));
}

#[test]
fn negative_ambient_factor_is_clamped_to_zero_contribution() {
    let material = Material::matte(Color::new(0.5, 0.5, 0.5, 1.0));
    let color = ambient(&material, -1.0);

    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn ambient_output_stays_within_normalized_color_range() {
    let material = Material::matte(Color::new(0.9, 0.9, 0.9, 1.0));
    let color = ambient(&material, 5.0);

    assert!((0.0..=1.0).contains(&color.r));
    assert!((0.0..=1.0).contains(&color.g));
    assert!((0.0..=1.0).contains(&color.b));
}

#[test]
fn ambient_result_is_finite() {
    let material = Material::glossy(Color::new(0.2, 0.7, 0.4, 1.0));
    let color = ambient(&material, 0.3);

    assert!(color.r.is_finite());
    assert!(color.g.is_finite());
    assert!(color.b.is_finite());
}

#[test]
fn normal_facing_light_produces_maximum_diffuse() {
    let material = Material::matte(Color::new(1.0, 1.0, 1.0, 1.0));
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);

    let color = diffuse_directional(&material, normal, &light);
    assert!(approx_eq(color.r, 1.0));
    assert!(approx_eq(color.g, 1.0));
    assert!(approx_eq(color.b, 1.0));
}

#[test]
fn normal_perpendicular_to_light_produces_zero_diffuse() {
    let material = Material::matte(Color::new(1.0, 1.0, 1.0, 1.0));
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(1.0, 0.0, 0.0);

    let color = diffuse_directional(&material, normal, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn normal_facing_away_from_light_produces_zero_diffuse() {
    let material = Material::matte(Color::new(1.0, 1.0, 1.0, 1.0));
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, -1.0, 0.0);

    let color = diffuse_directional(&material, normal, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn zero_intensity_produces_zero_diffuse() {
    let material = Material::matte(Color::white());
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 0.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);

    let color = diffuse_directional(&material, normal, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn diffuse_combines_albedo_and_light_color() {
    let material = Material::matte(Color::new(1.0, 0.5, 0.0, 1.0));
    let light = DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::new(0.5, 0.5, 0.5, 1.0),
        1.0,
    );
    let normal = Vec3::new(0.0, 1.0, 0.0);

    let color = diffuse_directional(&material, normal, &light);
    assert!(approx_eq(color.r, 0.5));
    assert!(approx_eq(color.g, 0.25));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn diffuse_result_is_finite_across_angles() {
    let material = Material::matte(Color::new(0.7, 0.3, 0.6, 1.0));
    let light = DirectionalLight::new(Vec3::new(1.0, 1.0, 1.0), Color::white(), 1.5);

    let normals = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(-1.0, -1.0, -1.0),
    ];

    for normal in normals {
        let color = diffuse_directional(&material, normal, &light);
        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
    }
}

#[test]
fn viewer_aligned_with_reflection_produces_strong_highlight() {
    let material = Material::glossy(Color::white());
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let view_direction = Vec3::new(0.0, 1.0, 0.0);

    let color = specular_directional(&material, normal, view_direction, &light);
    assert!(color.r > 0.5);
}

#[test]
fn viewer_away_from_reflection_reduces_highlight() {
    let material = Material::glossy(Color::white());
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);

    let aligned_view = Vec3::new(0.0, 1.0, 0.0);
    let angled_view = Vec3::new(0.9, 0.1, 0.0).normalize();

    let aligned = specular_directional(&material, normal, aligned_view, &light);
    let angled = specular_directional(&material, normal, angled_view, &light);

    assert!(aligned.r > angled.r);
}

#[test]
fn zero_specular_material_produces_no_highlight() {
    let material = Material::matte(Color::white());
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let view_direction = Vec3::new(0.0, 1.0, 0.0);

    let color = specular_directional(&material, normal, view_direction, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn higher_shininess_concentrates_the_highlight() {
    let low_shininess = Material::new(Color::white(), 1.0, 4.0);
    let high_shininess = Material::new(Color::white(), 1.0, 256.0);
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let off_axis_view = Vec3::new(0.5, 0.866, 0.0).normalize();

    let low = specular_directional(&low_shininess, normal, off_axis_view, &light);
    let high = specular_directional(&high_shininess, normal, off_axis_view, &light);

    assert!(high.r < low.r);
}

#[test]
fn back_facing_surface_produces_no_specular_even_when_view_aligns_with_light() {
    let material = Material::glossy(Color::white());
    let light = DirectionalLight::new(Vec3::new(0.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, -1.0, 0.0);
    let view_direction = Vec3::new(0.0, 1.0, 0.0);

    let color = specular_directional(&material, normal, view_direction, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn specular_result_is_finite() {
    let material = Material::glossy(Color::new(0.9, 0.9, 1.0, 1.0));
    let light = DirectionalLight::new(Vec3::new(1.0, 1.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let view_direction = Vec3::new(0.3, 0.9, 0.1).normalize();

    let color = specular_directional(&material, normal, view_direction, &light);
    assert!(color.r.is_finite());
    assert!(color.g.is_finite());
    assert!(color.b.is_finite());
}

#[test]
fn point_light_in_front_of_surface_produces_diffuse() {
    let material = Material::matte(Color::white());
    let light = PointLight::new(Vec3::new(0.0, 5.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::zero();

    let color = diffuse_point(&material, normal, hit_point, &light);
    assert!(color.r > 0.0);
}

#[test]
fn point_light_behind_surface_produces_zero_diffuse() {
    let material = Material::matte(Color::white());
    let light = PointLight::new(Vec3::new(0.0, -5.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::zero();

    let color = diffuse_point(&material, normal, hit_point, &light);
    assert!(approx_eq(color.r, 0.0));
    assert!(approx_eq(color.g, 0.0));
    assert!(approx_eq(color.b, 0.0));
}

#[test]
fn point_light_lateral_to_surface_produces_partial_diffuse() {
    let material = Material::matte(Color::white());
    let light = PointLight::new(Vec3::new(5.0, 5.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::zero();

    let color = diffuse_point(&material, normal, hit_point, &light);
    assert!(color.r > 0.0);
    assert!(color.r < 1.0);
}

#[test]
fn point_light_specular_is_visible_when_view_aligns_with_reflection() {
    let material = Material::glossy(Color::white());
    let light = PointLight::new(Vec3::new(0.0, 5.0, 0.0), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::zero();
    let view_direction = Vec3::new(0.0, 1.0, 0.0);

    let color = specular_point(&material, normal, hit_point, view_direction, &light);
    assert!(color.r > 0.5);
}

#[test]
fn point_light_coincident_with_hit_point_is_handled_safely() {
    let material = Material::glossy(Color::white());
    let light = PointLight::new(Vec3::zero(), Color::white(), 1.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::zero();
    let view_direction = Vec3::new(0.0, 1.0, 0.0);

    let diffuse = diffuse_point(&material, normal, hit_point, &light);
    let specular = specular_point(&material, normal, hit_point, view_direction, &light);

    assert!(diffuse.r.is_finite());
    assert!(specular.r.is_finite());
    assert!(!diffuse.r.is_nan());
    assert!(!specular.r.is_nan());
}

#[test]
fn point_light_diffuse_and_specular_results_are_finite() {
    let material = Material::glossy(Color::new(0.7, 0.4, 0.2, 1.0));
    let light = PointLight::new(Vec3::new(3.0, 2.0, -1.0), Color::white(), 1.5);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let hit_point = Vec3::new(0.1, 0.0, 0.2);
    let view_direction = Vec3::new(0.2, 0.9, 0.1).normalize();

    let diffuse = diffuse_point(&material, normal, hit_point, &light);
    let specular = specular_point(&material, normal, hit_point, view_direction, &light);

    assert!(diffuse.r.is_finite());
    assert!(diffuse.g.is_finite());
    assert!(diffuse.b.is_finite());
    assert!(specular.r.is_finite());
    assert!(specular.g.is_finite());
    assert!(specular.b.is_finite());
}
