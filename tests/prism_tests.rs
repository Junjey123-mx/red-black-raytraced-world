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
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

use core::cube::Cube;
use core::hit::Face;
use core::math::Vec3;
use core::prism::Prism;
use core::ray::Ray;

const EPS: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn half_slab() -> Prism {
    Prism::new(v(0.0, 0.0, 0.0), v(1.0, 0.5, 1.0))
}

#[test]
fn front_hit_on_general_prism() {
    let ray = Ray::new(v(0.5, 0.25, 5.0), v(0.0, 0.0, -1.0));
    let hit = half_slab().intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert!(approx(hit.distance, 4.0));
    assert!(approx(hit.point.z, 1.0));
    assert_eq!(hit.face, Face::PositiveZ);
    assert_eq!(hit.normal, v(0.0, 0.0, 1.0));
}

#[test]
fn ray_above_the_slab_misses() {
    let ray = Ray::new(v(0.5, 0.75, 5.0), v(0.0, 0.0, -1.0));
    assert!(half_slab().intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn ray_pointing_away_misses() {
    let ray = Ray::new(v(0.5, 0.25, 5.0), v(0.0, 0.0, 1.0));
    assert!(half_slab().intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn thin_prism_is_hit_on_its_broad_face() {
    let door = Prism::new(v(0.0, 0.0, 0.8125), v(1.0, 1.0, 1.0));
    let ray = Ray::new(v(0.5, 0.5, 5.0), v(0.0, 0.0, -1.0));
    let hit = door.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert!(approx(hit.distance, 4.0));
    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn thin_prism_is_hit_on_its_narrow_edge() {
    let door = Prism::new(v(0.0, 0.0, 0.8125), v(1.0, 1.0, 1.0));
    let ray = Ray::new(v(-3.0, 0.5, 0.9), v(1.0, 0.0, 0.0));
    let hit = door.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert!(approx(hit.distance, 3.0));
    assert_eq!(hit.face, Face::NegativeX);
}

#[test]
fn thin_prism_is_missed_through_the_empty_part_of_the_cell() {
    let door = Prism::new(v(0.0, 0.0, 0.8125), v(1.0, 1.0, 1.0));
    let ray = Ray::new(v(0.5, 0.5, 0.4), v(1.0, 0.0, 0.0));
    let ray_through_gap = Ray::new(v(-1.0, 0.5, 0.4), v(1.0, 0.0, 0.0));

    assert!(door.intersect(&ray, 0.0, f32::INFINITY).is_none());
    assert!(
        door.intersect(&ray_through_gap, 0.0, f32::INFINITY)
            .is_none()
    );
}

#[test]
fn ray_from_inside_reports_the_exit_surface() {
    let ray = Ray::new(v(0.5, 0.25, 0.5), v(0.0, 1.0, 0.0));
    let hit = half_slab().intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert!(approx(hit.distance, 0.25));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn every_face_reports_its_outward_normal() {
    let prism = Prism::new(v(0.25, 0.25, 0.25), v(0.75, 0.75, 0.75));
    let cases = [
        (v(-2.0, 0.5, 0.5), v(1.0, 0.0, 0.0), Face::NegativeX),
        (v(3.0, 0.5, 0.5), v(-1.0, 0.0, 0.0), Face::PositiveX),
        (v(0.5, -2.0, 0.5), v(0.0, 1.0, 0.0), Face::NegativeY),
        (v(0.5, 3.0, 0.5), v(0.0, -1.0, 0.0), Face::PositiveY),
        (v(0.5, 0.5, -2.0), v(0.0, 0.0, 1.0), Face::NegativeZ),
        (v(0.5, 0.5, 3.0), v(0.0, 0.0, -1.0), Face::PositiveZ),
    ];

    for (origin, direction, face) in cases {
        let hit = prism
            .intersect(&Ray::new(origin, direction), 0.0, f32::INFINITY)
            .unwrap();
        assert_eq!(hit.face, face);
        assert_eq!(hit.normal, face.normal());
    }
}

#[test]
fn uv_stays_within_unit_range() {
    let prism = Prism::new(v(0.2, 0.0, 0.4), v(0.8, 0.5, 0.6));
    let origins = [
        (v(0.5, 0.25, 5.0), v(0.0, 0.0, -1.0)),
        (v(0.5, 5.0, 0.5), v(0.0, -1.0, 0.0)),
        (v(-5.0, 0.25, 0.5), v(1.0, 0.0, 0.0)),
        (v(5.0, 0.1, 0.55), v(-1.0, 0.0, 0.0)),
    ];

    for (origin, direction) in origins {
        let hit = prism
            .intersect(&Ray::new(origin, direction), 0.0, f32::INFINITY)
            .unwrap();
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}

#[test]
fn unit_prism_matches_the_unit_cube() {
    let prism = Prism::unit();
    let cube = Cube::new(v(0.0, 0.0, 0.0), v(1.0, 1.0, 1.0));
    let rays = [
        Ray::new(v(0.3, 0.6, 4.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(-2.0, 0.4, 0.7), v(1.0, 0.1, 0.0)),
        Ray::new(v(0.5, 5.0, 0.5), v(0.1, -1.0, 0.05)),
        Ray::new(v(0.5, 0.5, 0.5), v(0.0, 0.0, 1.0)),
    ];

    for ray in rays {
        let a = prism.intersect(&ray, 0.0, f32::INFINITY);
        let b = cube.intersect(&ray, 0.0, f32::INFINITY);
        assert_eq!(a, b);
    }
}

#[test]
fn corners_are_normalized_and_exposed() {
    let prism = Prism::new(v(0.6, 0.5, 0.9), v(0.4, 0.0, 0.1));

    assert_eq!(prism.min(), v(0.4, 0.0, 0.1));
    assert_eq!(prism.max(), v(0.6, 0.5, 0.9));
    assert!(prism.is_within_unit_cell());
    assert!(!Prism::new(v(0.0, 0.0, 0.0), v(1.5, 1.0, 1.0)).is_within_unit_cell());
    assert_eq!(prism.aabb().min, prism.min());
}

#[test]
fn translation_moves_both_corners() {
    let moved = half_slab().translated(v(3.0, 2.0, -1.0));
    assert_eq!(moved.min(), v(3.0, 2.0, -1.0));
    assert_eq!(moved.max(), v(4.0, 2.5, 0.0));
}
