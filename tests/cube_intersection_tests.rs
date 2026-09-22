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

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

use core::cube::Cube;
use core::hit::Face;
use core::math::Vec3;
use core::ray::Ray;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

#[test]
fn hits_positive_x_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::PositiveX);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.x, 1.0));
    assert!(approx_eq(hit.normal.x, 1.0));
}

#[test]
fn hits_negative_x_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::NegativeX);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.x, -1.0));
    assert!(approx_eq(hit.normal.x, -1.0));
}

#[test]
fn hits_positive_y_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::PositiveY);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.y, 1.0));
    assert!(approx_eq(hit.normal.y, 1.0));
}

#[test]
fn hits_negative_y_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::NegativeY);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.y, -1.0));
    assert!(approx_eq(hit.normal.y, -1.0));
}

#[test]
fn hits_positive_z_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::PositiveZ);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.z, 1.0));
    assert!(approx_eq(hit.normal.z, 1.0));
}

#[test]
fn hits_negative_z_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::NegativeZ);
    assert!(approx_eq(hit.distance, 4.0));
    assert!(approx_eq(hit.point.z, -1.0));
    assert!(approx_eq(hit.normal.z, -1.0));
}

#[test]
fn miss_returns_none() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    assert!(cube.intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn ray_from_inside_hits_exit_face() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit.face, Face::PositiveZ);
    assert!(approx_eq(hit.distance, 1.0));
    assert!(approx_eq(hit.point.z, 1.0));
}

#[test]
fn negative_direction_along_diagonal_hits_correctly() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 5.0, 0.0), Vec3::new(-1.0, -1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert!(hit.distance.is_finite());
    assert!(hit.point.x <= 1.0 + EPS);
    assert!(hit.point.y <= 1.0 + EPS);
    assert!(approx_eq(hit.point.x, 1.0));
    assert!(approx_eq(hit.point.y, 1.0));
}

#[test]
fn grazing_hit_along_tangent_edge_is_finite_and_deterministic() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

    let hit_a = cube.intersect(&ray, 0.0, f32::INFINITY);
    let hit_b = cube.intersect(&ray, 0.0, f32::INFINITY);

    assert!(hit_a.is_some());
    let hit_a = hit_a.unwrap();
    let hit_b = hit_b.unwrap();

    assert!(hit_a.distance.is_finite());
    assert!(!hit_a.distance.is_nan());
    assert_eq!(hit_a.face, hit_b.face);
    assert!(approx_eq(hit_a.distance, hit_b.distance));
}

#[test]
fn corner_hit_face_selection_is_deterministic() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(1.0, 1.0, 1.0));

    let hit_a = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let hit_b = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit_a.face, Face::NegativeX);
    assert_eq!(hit_a.face, hit_b.face);
    assert!(approx_eq(hit_a.point.x, -1.0));
    assert!(approx_eq(hit_a.point.y, -1.0));
    assert!(approx_eq(hit_a.point.z, -1.0));
}
