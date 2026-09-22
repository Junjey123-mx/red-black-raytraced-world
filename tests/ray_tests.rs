#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

use core::math::Vec3;
use core::ray::Ray;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
}

#[test]
fn at_zero_returns_origin() {
    let origin = Vec3::new(1.0, 2.0, 3.0);
    let ray = Ray::new(origin, Vec3::new(0.0, 0.0, 1.0));
    assert!(vec3_approx_eq(ray.at(0.0), origin));
}

#[test]
fn direction_is_normalized_on_construction() {
    let ray = Ray::new(Vec3::zero(), Vec3::new(3.0, 4.0, 0.0));
    assert!(approx_eq(ray.direction.length(), 1.0));
}

#[test]
fn at_known_t_matches_expected_point() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    let point = ray.at(5.0);
    assert!(vec3_approx_eq(point, Vec3::new(5.0, 0.0, 0.0)));
}

#[test]
fn zero_direction_stays_finite_and_at_stays_at_origin() {
    let origin = Vec3::new(1.0, 1.0, 1.0);
    let ray = Ray::new(origin, Vec3::zero());

    assert!(ray.direction.x.is_finite());
    assert!(ray.direction.y.is_finite());
    assert!(ray.direction.z.is_finite());
    assert!(vec3_approx_eq(ray.direction, Vec3::zero()));

    assert!(vec3_approx_eq(ray.at(10.0), origin));
}
