#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;

    pub use camera::Camera;
}

use camera::Camera;
use core::math::Vec3;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn construction_preserves_position_target_and_up() {
    let position = Vec3::new(0.0, 0.0, 5.0);
    let target = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);

    let camera = Camera::new(position, target, up, 60.0, 16.0 / 9.0);

    assert!(approx_eq(camera.position.x, position.x));
    assert!(approx_eq(camera.position.y, position.y));
    assert!(approx_eq(camera.position.z, position.z));

    assert!(approx_eq(camera.target.x, target.x));
    assert!(approx_eq(camera.target.y, target.y));
    assert!(approx_eq(camera.target.z, target.z));

    assert!(approx_eq(camera.up.x, up.x));
    assert!(approx_eq(camera.up.y, up.y));
    assert!(approx_eq(camera.up.z, up.z));
}

#[test]
fn construction_preserves_fov_and_aspect_ratio_within_range() {
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.5,
    );

    assert!(approx_eq(camera.fov, 60.0));
    assert!(approx_eq(camera.aspect_ratio, 1.5));
}

#[test]
fn all_fields_are_finite() {
    let camera = Camera::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(-1.0, -2.0, -3.0),
        Vec3::new(0.0, 1.0, 0.0),
        75.0,
        4.0 / 3.0,
    );

    assert!(camera.position.x.is_finite());
    assert!(camera.position.y.is_finite());
    assert!(camera.position.z.is_finite());
    assert!(camera.target.x.is_finite());
    assert!(camera.target.y.is_finite());
    assert!(camera.target.z.is_finite());
    assert!(camera.up.x.is_finite());
    assert!(camera.up.y.is_finite());
    assert!(camera.up.z.is_finite());
    assert!(camera.fov.is_finite());
    assert!(camera.aspect_ratio.is_finite());
}

#[test]
fn degenerate_fov_and_aspect_ratio_stay_finite_and_positive() {
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        0.0,
    );

    assert!(camera.fov.is_finite());
    assert!(camera.fov > 0.0);
    assert!(camera.aspect_ratio.is_finite());
    assert!(camera.aspect_ratio > 0.0);
}
