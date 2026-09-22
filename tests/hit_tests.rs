#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }

    #[path = "../src/core/hit.rs"]
    pub mod hit;
}

use core::hit::{Face, HitRecord};
use core::math::Vec3;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn each_face_has_its_canonical_unit_normal() {
    let cases = [
        (Face::PositiveX, Vec3::new(1.0, 0.0, 0.0)),
        (Face::NegativeX, Vec3::new(-1.0, 0.0, 0.0)),
        (Face::PositiveY, Vec3::new(0.0, 1.0, 0.0)),
        (Face::NegativeY, Vec3::new(0.0, -1.0, 0.0)),
        (Face::PositiveZ, Vec3::new(0.0, 0.0, 1.0)),
        (Face::NegativeZ, Vec3::new(0.0, 0.0, -1.0)),
    ];

    for (face, expected_normal) in cases {
        let normal = face.normal();
        assert!(approx_eq(normal.x, expected_normal.x));
        assert!(approx_eq(normal.y, expected_normal.y));
        assert!(approx_eq(normal.z, expected_normal.z));
        assert!(approx_eq(normal.length(), 1.0));
    }
}

#[test]
fn hit_record_preserves_distance_and_point() {
    let point = Vec3::new(1.0, 2.0, 3.0);
    let hit = HitRecord::new(4.5, point, Face::PositiveY);

    assert!(approx_eq(hit.distance, 4.5));
    assert!(approx_eq(hit.point.x, point.x));
    assert!(approx_eq(hit.point.y, point.y));
    assert!(approx_eq(hit.point.z, point.z));
}

#[test]
fn hit_record_face_is_preserved() {
    let hit = HitRecord::new(1.0, Vec3::zero(), Face::NegativeZ);
    assert_eq!(hit.face, Face::NegativeZ);
}

#[test]
fn hit_record_normal_is_unit_and_matches_face() {
    let hit = HitRecord::new(2.0, Vec3::new(1.0, 0.0, 0.0), Face::PositiveX);

    assert!(approx_eq(hit.normal.length(), 1.0));
    assert!(approx_eq(hit.normal.x, 1.0));
    assert!(approx_eq(hit.normal.y, 0.0));
    assert!(approx_eq(hit.normal.z, 0.0));
}

#[test]
fn distinct_faces_are_not_equal() {
    assert_ne!(Face::PositiveX, Face::NegativeX);
    assert_ne!(Face::PositiveY, Face::PositiveZ);
}
