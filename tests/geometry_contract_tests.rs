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
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
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
use core::cube::Cube;
use core::hit::Face;
use core::math::Vec3;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// Camera in front of the cube, looking straight at it (-Z). The center
/// pixel ray should hit the cube's positive-Z face head-on.
#[test]
fn front_camera_center_ray_hits_front_face() {
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );
    let cube = unit_cube();

    let ray = primary_ray(&camera, 50, 50, 101, 101);
    let hit = cube
        .intersect(&ray, 0.0, f32::INFINITY)
        .expect("front camera ray must hit the cube");

    assert_eq!(hit.face, Face::PositiveZ);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.x, 0.0));
    assert!(approx_eq(hit.point.y, 0.0));
    assert!(approx_eq(hit.point.z, 1.0));

    assert!(approx_eq(hit.normal.length(), 1.0));
    assert!(approx_eq(hit.normal.x, 0.0));
    assert!(approx_eq(hit.normal.y, 0.0));
    assert!(approx_eq(hit.normal.z, 1.0));

    assert!(ray.direction.dot(hit.normal) < 0.0);
}

/// Camera to the side of the cube, looking straight at it along -X. The
/// center pixel ray should hit the cube's positive-X (lateral) face.
#[test]
fn lateral_camera_center_ray_hits_side_face() {
    let camera = Camera::new(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        1.0,
    );
    let cube = unit_cube();

    let ray = primary_ray(&camera, 50, 50, 101, 101);
    let hit = cube
        .intersect(&ray, 0.0, f32::INFINITY)
        .expect("lateral camera ray must hit the cube");

    assert_eq!(hit.face, Face::PositiveX);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.x, 1.0));
    assert!(approx_eq(hit.point.y, 0.0));
    assert!(approx_eq(hit.point.z, 0.0));

    assert!(approx_eq(hit.normal.length(), 1.0));
    assert!(approx_eq(hit.normal.x, 1.0));

    assert!(ray.direction.dot(hit.normal) < 0.0);
}

/// Camera above the cube, looking straight down (-Y), with an up vector
/// that is not parallel to forward. The center pixel ray should hit the
/// cube's positive-Y (top) face.
#[test]
fn top_camera_center_ray_hits_top_face() {
    let camera = Camera::new(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
        60.0,
        1.0,
    );
    let cube = unit_cube();

    let ray = primary_ray(&camera, 50, 50, 101, 101);
    let hit = cube
        .intersect(&ray, 0.0, f32::INFINITY)
        .expect("top camera ray must hit the cube");

    assert_eq!(hit.face, Face::PositiveY);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.x, 0.0));
    assert!(approx_eq(hit.point.y, 1.0));
    assert!(approx_eq(hit.point.z, 0.0));

    assert!(approx_eq(hit.normal.length(), 1.0));
    assert!(approx_eq(hit.normal.y, 1.0));

    assert!(ray.direction.dot(hit.normal) < 0.0);
}

#[test]
fn all_three_viewpoints_produce_finite_hits_with_no_nan() {
    let cube = unit_cube();
    let cameras = [
        Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::zero(),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
        ),
        Camera::new(
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::zero(),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
        ),
        Camera::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::zero(),
            Vec3::new(0.0, 0.0, -1.0),
            60.0,
            1.0,
        ),
    ];

    for camera in cameras {
        let ray = primary_ray(&camera, 50, 50, 101, 101);
        let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

        assert!(hit.distance.is_finite());
        assert!(!hit.distance.is_nan());
        assert!(hit.point.x.is_finite());
        assert!(hit.point.y.is_finite());
        assert!(hit.point.z.is_finite());
        assert!(hit.normal.x.is_finite());
        assert!(hit.normal.y.is_finite());
        assert!(hit.normal.z.is_finite());
    }
}
