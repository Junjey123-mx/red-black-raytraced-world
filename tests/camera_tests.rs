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

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;

    pub use camera::Camera;
    pub use projection::primary_ray;
}

use camera::{Camera, primary_ray};
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

#[test]
fn basis_vectors_are_unit_length() {
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        16.0 / 9.0,
    );
    let basis = camera.basis();

    assert!(approx_eq(basis.forward.length(), 1.0));
    assert!(approx_eq(basis.right.length(), 1.0));
    assert!(approx_eq(basis.up.length(), 1.0));
}

#[test]
fn basis_vectors_are_mutually_orthogonal() {
    let camera = Camera::new(
        Vec3::new(2.0, 3.0, 5.0),
        Vec3::new(-1.0, 0.5, 2.0),
        Vec3::new(0.0, 1.0, 0.0),
        75.0,
        4.0 / 3.0,
    );
    let basis = camera.basis();

    assert!(approx_eq(basis.forward.dot(basis.right), 0.0));
    assert!(approx_eq(basis.forward.dot(basis.up), 0.0));
    assert!(approx_eq(basis.right.dot(basis.up), 0.0));
}

#[test]
fn canonical_orientation_looking_down_negative_z() {
    let camera = Camera::new(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );
    let basis = camera.basis();

    assert!(approx_eq(basis.forward.x, 0.0));
    assert!(approx_eq(basis.forward.y, 0.0));
    assert!(approx_eq(basis.forward.z, -1.0));

    assert!(approx_eq(basis.right.x, 1.0));
    assert!(approx_eq(basis.right.y, 0.0));
    assert!(approx_eq(basis.right.z, 0.0));

    assert!(approx_eq(basis.up.x, 0.0));
    assert!(approx_eq(basis.up.y, 1.0));
    assert!(approx_eq(basis.up.z, 0.0));
}

#[test]
fn degenerate_up_parallel_to_forward_stays_finite_and_orthonormal() {
    let camera = Camera::new(
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );
    let basis = camera.basis();

    assert!(basis.forward.x.is_finite());
    assert!(basis.forward.y.is_finite());
    assert!(basis.forward.z.is_finite());
    assert!(basis.right.x.is_finite());
    assert!(basis.right.y.is_finite());
    assert!(basis.right.z.is_finite());
    assert!(basis.up.x.is_finite());
    assert!(basis.up.y.is_finite());
    assert!(basis.up.z.is_finite());

    assert!(approx_eq(basis.forward.length(), 1.0));
    assert!(approx_eq(basis.right.length(), 1.0));
    assert!(approx_eq(basis.up.length(), 1.0));

    assert!(approx_eq(basis.forward.dot(basis.right), 0.0));
    assert!(approx_eq(basis.forward.dot(basis.up), 0.0));
    assert!(approx_eq(basis.right.dot(basis.up), 0.0));
}

fn straight_camera() -> Camera {
    Camera::new(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
        1.0,
    )
}

#[test]
fn center_pixel_ray_points_approximately_forward() {
    let camera = straight_camera();
    let ray = primary_ray(&camera, 50, 50, 101, 101);

    assert!(approx_eq(ray.direction.x, 0.0));
    assert!(approx_eq(ray.direction.y, 0.0));
    assert!(approx_eq(ray.direction.z, -1.0));
}

#[test]
fn left_and_right_pixels_diverge_in_opposite_directions() {
    let camera = straight_camera();
    let left = primary_ray(&camera, 0, 50, 101, 101);
    let right = primary_ray(&camera, 100, 50, 101, 101);

    assert!(left.direction.x < 0.0);
    assert!(right.direction.x > 0.0);
}

#[test]
fn aspect_ratio_widens_horizontal_spread() {
    let mut camera = straight_camera();
    camera.aspect_ratio = 1.0;
    let narrow = primary_ray(&camera, 100, 50, 101, 101);

    camera.aspect_ratio = 3.0;
    let wide = primary_ray(&camera, 100, 50, 101, 101);

    assert!(wide.direction.x > narrow.direction.x);
}

#[test]
fn wider_fov_increases_ray_dispersion() {
    let mut camera = straight_camera();
    camera.fov = 30.0;
    let narrow = primary_ray(&camera, 100, 50, 101, 101);

    camera.fov = 120.0;
    let wide = primary_ray(&camera, 100, 50, 101, 101);

    assert!(wide.direction.x.abs() > narrow.direction.x.abs());
}

#[test]
fn all_primary_ray_directions_are_finite_and_unit_length() {
    let camera = straight_camera();
    let width = 8;
    let height = 6;

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            assert!(ray.direction.x.is_finite());
            assert!(ray.direction.y.is_finite());
            assert!(ray.direction.z.is_finite());
            assert!(approx_eq(ray.direction.length(), 1.0));
        }
    }
}

#[test]
fn primary_ray_originates_at_camera_position() {
    let camera = straight_camera();
    let ray = primary_ray(&camera, 20, 30, 101, 101);

    assert!(approx_eq(ray.origin.x, camera.position.x));
    assert!(approx_eq(ray.origin.y, camera.position.y));
    assert!(approx_eq(ray.origin.z, camera.position.z));
}
