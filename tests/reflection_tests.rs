#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
}

use core::math::Vec3;
use core::reflection::reflect;

const EPS: f32 = 1e-5;

fn approx(a: Vec3, b: Vec3) -> bool {
    (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS && (a.z - b.z).abs() < EPS
}

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[test]
fn frontal_incidence_bounces_straight_back() {
    let r = reflect(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    assert!(approx(r, Vec3::new(0.0, 1.0, 0.0)), "r = {r:?}");
}

#[test]
fn incidence_at_forty_five_degrees_mirrors_the_normal_component() {
    let d = Vec3::new(1.0, -1.0, 0.0).normalize();
    let r = reflect(d, Vec3::new(0.0, 1.0, 0.0));
    assert!(approx(r, Vec3::new(1.0, 1.0, 0.0).normalize()), "r = {r:?}");
}

#[test]
fn reflection_is_symmetric_about_the_normal() {
    let n = Vec3::new(0.0, 1.0, 0.0);
    let d = Vec3::new(0.3, -0.8, 0.5).normalize();
    let r = reflect(d, n);

    // Equal angle to the normal, opposite side of it, same tangential part.
    assert!((r.dot(n) + d.dot(n)).abs() < EPS);
    assert!((r.x - d.x).abs() < EPS && (r.z - d.z).abs() < EPS);
    // Reflecting again returns the original direction.
    assert!(approx(reflect(r, n), d));
}

#[test]
fn the_reflected_direction_is_unit_length() {
    for d in [
        Vec3::new(1.0, -2.0, 3.0),
        Vec3::new(-0.2, -0.1, 0.9),
        Vec3::new(4.0, -0.001, 0.0),
    ] {
        let r = reflect(d.normalize(), Vec3::new(0.2, 1.0, -0.1).normalize());
        assert!((r.length() - 1.0).abs() < EPS);
    }
}

#[test]
fn the_sign_of_the_normal_does_not_change_the_mirror() {
    let d = Vec3::new(0.4, -0.7, 0.2).normalize();
    let n = Vec3::new(0.0, 1.0, 0.0);
    assert!(approx(reflect(d, n), reflect(d, -n)));
}

#[test]
fn a_non_unit_normal_is_normalized_internally() {
    let d = Vec3::new(0.4, -0.7, 0.2).normalize();
    assert!(approx(
        reflect(d, Vec3::new(0.0, 5.0, 0.0)),
        reflect(d, Vec3::new(0.0, 1.0, 0.0))
    ));
}

#[test]
fn degenerate_input_never_produces_nan() {
    let zero = Vec3::zero();
    let up = Vec3::new(0.0, 1.0, 0.0);
    let d = Vec3::new(0.0, -1.0, 0.0);

    assert!(finite(reflect(d, zero)));
    assert!(finite(reflect(zero, up)));
    assert!(finite(reflect(zero, zero)));
    assert!(finite(reflect(d, Vec3::new(f32::NAN, 0.0, 0.0))));
    assert!(finite(reflect(Vec3::new(f32::INFINITY, 0.0, 0.0), up)));
}
