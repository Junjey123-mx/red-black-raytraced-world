#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
}

use core::math::Vec3;
use core::refraction::refract;

const EPS: f32 = 1e-5;
const AIR: f32 = 1.0;
const WATER: f32 = 1.33;
const GLASS: f32 = 1.5;

fn up() -> Vec3 {
    Vec3::new(0.0, 1.0, 0.0)
}

/// Incoming direction hitting a horizontal surface at `degrees` from the
/// normal, traveling down and toward +x.
fn incoming(degrees: f32) -> Vec3 {
    let a = degrees.to_radians();
    Vec3::new(a.sin(), -a.cos(), 0.0)
}

/// Angle (degrees) between `v` and the downward axis.
fn angle_from_down(v: Vec3) -> f32 {
    v.dot(Vec3::new(0.0, -1.0, 0.0))
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[test]
fn air_to_glass_bends_toward_the_normal_following_snell() {
    let t = refract(incoming(45.0), up(), AIR, GLASS).expect("air->glass always refracts");
    let expected = ((45.0_f32.to_radians().sin() * AIR / GLASS).asin()).to_degrees();

    assert!((angle_from_down(t) - expected).abs() < 1e-3, "t = {t:?}");
    assert!(angle_from_down(t) < 45.0);
}

#[test]
fn air_to_water_bends_less_than_air_to_glass() {
    let water = refract(incoming(45.0), up(), AIR, WATER).unwrap();
    let glass = refract(incoming(45.0), up(), AIR, GLASS).unwrap();
    let expected = ((45.0_f32.to_radians().sin() * AIR / WATER).asin()).to_degrees();

    assert!((angle_from_down(water) - expected).abs() < 1e-3);
    assert!(angle_from_down(water) > angle_from_down(glass));
}

#[test]
fn glass_to_air_bends_away_from_the_normal() {
    let t = refract(incoming(20.0), up(), GLASS, AIR).expect("below the critical angle");
    let expected = ((20.0_f32.to_radians().sin() * GLASS / AIR).asin()).to_degrees();

    assert!((angle_from_down(t) - expected).abs() < 1e-3, "t = {t:?}");
    assert!(angle_from_down(t) > 20.0);
}

#[test]
fn normal_incidence_goes_straight_through_without_lateral_deviation() {
    for (eta_i, eta_t) in [(AIR, GLASS), (AIR, WATER), (GLASS, AIR)] {
        let t = refract(Vec3::new(0.0, -1.0, 0.0), up(), eta_i, eta_t).unwrap();
        assert!(t.x.abs() < EPS && t.z.abs() < EPS);
        assert!((t.y + 1.0).abs() < EPS);
    }
}

#[test]
fn oblique_incidence_keeps_the_ray_in_its_plane_of_incidence() {
    let d = Vec3::new(0.3, -0.8, 0.5).normalize();
    let t = refract(d, up(), AIR, GLASS).unwrap();

    // Tangential component keeps its direction and shrinks by eta_i/eta_t.
    let (dx, dz) = (d.x, d.z);
    assert!((t.x * dz - t.z * dx).abs() < EPS);
    assert!(t.x.hypot(t.z) < dx.hypot(dz));
    assert!(t.y < 0.0);
}

#[test]
fn total_internal_reflection_returns_none_beyond_the_critical_angle() {
    // Glass -> air critical angle is asin(1/1.5) ~ 41.8 degrees.
    assert!(refract(incoming(60.0), up(), GLASS, AIR).is_none());
    assert!(refract(incoming(45.0), up(), GLASS, AIR).is_none());
    assert!(refract(incoming(40.0), up(), GLASS, AIR).is_some());
    // Entering a denser medium can never be TIR, even at grazing incidence.
    assert!(refract(incoming(89.9), up(), AIR, GLASS).is_some());
}

#[test]
fn the_transmitted_direction_is_unit_length() {
    for degrees in [0.0, 10.0, 30.0, 60.0, 85.0] {
        let t = refract(incoming(degrees), up(), AIR, WATER).unwrap();
        assert!((t.length() - 1.0).abs() < EPS, "deg {degrees}: {t:?}");
    }
}

#[test]
fn the_normal_may_point_either_way_and_need_not_be_unit() {
    let d = incoming(35.0);
    let a = refract(d, up(), AIR, GLASS).unwrap();
    let b = refract(d, Vec3::new(0.0, -4.0, 0.0), AIR, GLASS).unwrap();

    assert!((a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS && (a.z - b.z).abs() < EPS);
}

#[test]
fn equal_indices_do_not_bend_the_ray() {
    let d = incoming(50.0);
    let t = refract(d, up(), GLASS, GLASS).unwrap();

    assert!((t.x - d.x).abs() < EPS && (t.y - d.y).abs() < EPS);
}

#[test]
fn outputs_are_never_nan() {
    for degrees in [0.0, 1.0, 41.0, 42.0, 89.0, 90.0] {
        for (eta_i, eta_t) in [(AIR, GLASS), (GLASS, AIR), (WATER, AIR)] {
            if let Some(t) = refract(incoming(degrees), up(), eta_i, eta_t) {
                assert!(finite(t), "deg {degrees}: {t:?}");
            }
        }
    }
}

#[test]
fn degenerate_indices_and_vectors_are_handled_safely() {
    let d = incoming(30.0);

    assert!(refract(d, up(), 0.0, GLASS).is_none());
    assert!(refract(d, up(), AIR, -1.0).is_none());
    assert!(refract(d, up(), f32::NAN, GLASS).is_none());
    assert!(refract(d, up(), AIR, f32::INFINITY).is_none());
    assert!(refract(Vec3::zero(), up(), AIR, GLASS).is_none());
    assert!(refract(d, Vec3::zero(), AIR, GLASS).is_none());
}
