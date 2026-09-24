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

#[path = "."]
mod scene {
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
}

use core::cube::Cube;
use core::hit::Face;
use core::math::Vec3;
use core::prism::Prism;
use core::ray::Ray;
use scene::block_geometry::BlockGeometry;

const EPS: f32 = 1e-4;
const FAR: f32 = f32::INFINITY;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Two-step stair facing +Z: full-width lower half plus an upper half at the
/// back (z in [0, 0.5]).
fn stair_parts() -> Vec<Prism> {
    vec![
        Prism::new(v(0.0, 0.0, 0.0), v(1.0, 0.5, 1.0)),
        Prism::new(v(0.0, 0.5, 0.0), v(1.0, 1.0, 0.5)),
    ]
}

fn stairs() -> BlockGeometry {
    BlockGeometry::composite(stair_parts()).unwrap()
}

fn toward_neg_z(x: f32, y: f32) -> Ray {
    Ray::new(v(x, y, 5.0), v(0.0, 0.0, -1.0))
}

#[test]
fn full_cube_is_hit() {
    let hit = BlockGeometry::FullCube
        .intersect_local(&toward_neg_z(0.5, 0.5), 0.0, FAR)
        .unwrap();

    assert!(approx(hit.distance, 4.0));
    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn prism_is_hit_and_misses_outside() {
    let geometry = BlockGeometry::prism(Prism::new(v(0.4, 0.0, 0.4), v(0.6, 1.0, 0.6)));

    let hit = geometry
        .intersect_local(&toward_neg_z(0.5, 0.5), 0.0, FAR)
        .unwrap();
    assert!(approx(hit.distance, 4.4));
    assert!(
        geometry
            .intersect_local(&toward_neg_z(0.9, 0.5), 0.0, FAR)
            .is_none()
    );
}

#[test]
fn composite_hits_the_upper_step_first_at_the_top() {
    // Ray from +Z at y = 0.75 only reaches the upper (back) part, at z = 0.5.
    let hit = stairs()
        .intersect_local(&toward_neg_z(0.5, 0.75), 0.0, FAR)
        .unwrap();

    assert!(approx(hit.distance, 4.5));
    assert_eq!(hit.face, Face::PositiveZ);
    assert!(approx(hit.point.z, 0.5));
}

#[test]
fn composite_hits_the_lower_step_front_at_low_height() {
    let hit = stairs()
        .intersect_local(&toward_neg_z(0.5, 0.25), 0.0, FAR)
        .unwrap();

    assert!(approx(hit.distance, 4.0));
    assert!(approx(hit.point.z, 1.0));
}

#[test]
fn nearest_hit_wins_from_above() {
    // Coming straight down at z = 0.75 only the lower step top (y = 0.5)
    // exists; at z = 0.25 the upper step top (y = 1.0) is nearer.
    let front = stairs()
        .intersect_local(&Ray::new(v(0.5, 5.0, 0.75), v(0.0, -1.0, 0.0)), 0.0, FAR)
        .unwrap();
    let back = stairs()
        .intersect_local(&Ray::new(v(0.5, 5.0, 0.25), v(0.0, -1.0, 0.0)), 0.0, FAR)
        .unwrap();

    assert!(approx(front.point.y, 0.5));
    assert!(approx(back.point.y, 1.0));
    assert_eq!(front.face, Face::PositiveY);
    assert_eq!(back.face, Face::PositiveY);
}

#[test]
fn part_order_does_not_change_the_result() {
    let mut reversed = stair_parts();
    reversed.reverse();
    let reversed = BlockGeometry::composite(reversed).unwrap();

    let rays = [
        toward_neg_z(0.5, 0.75),
        toward_neg_z(0.5, 0.25),
        Ray::new(v(0.5, 5.0, 0.25), v(0.0, -1.0, 0.0)),
        Ray::new(v(-3.0, 0.75, 0.25), v(1.0, 0.0, 0.0)),
        Ray::new(v(0.3, 0.9, 3.0), v(0.1, -0.2, -1.0)),
    ];

    for ray in rays {
        assert_eq!(
            stairs().intersect_local(&ray, 0.0, FAR),
            reversed.intersect_local(&ray, 0.0, FAR)
        );
    }
}

#[test]
fn composite_misses_through_the_empty_space() {
    // Above the lower step and in front of the upper one there is air:
    // a ray at y = 0.75 traveling along +X through z = 0.75 misses both.
    let ray = Ray::new(v(-3.0, 0.75, 0.75), v(1.0, 0.0, 0.0));
    assert!(stairs().intersect_local(&ray, 0.0, FAR).is_none());
}

#[test]
fn ray_from_inside_a_part_reports_its_exit() {
    // z = 0.75 lies inside the lower step only.
    let ray = Ray::new(v(0.5, 0.25, 0.75), v(0.0, 1.0, 0.0));
    let hit = stairs().intersect_local(&ray, 0.0, FAR).unwrap();

    assert!(approx(hit.distance, 0.25));
    assert!(approx(hit.point.y, 0.5));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn face_normal_and_uv_are_preserved() {
    let hit = stairs()
        .intersect_local(&toward_neg_z(0.25, 0.25), 0.0, FAR)
        .unwrap();

    assert_eq!(hit.face, Face::PositiveZ);
    assert_eq!(hit.normal, v(0.0, 0.0, 1.0));
    assert!((0.0..=1.0).contains(&hit.uv.x));
    assert!((0.0..=1.0).contains(&hit.uv.y));
    assert!(approx(hit.uv.x, 0.25));
    assert!(approx(hit.uv.y, 0.5));
}

#[test]
fn max_distance_bounds_the_hit() {
    assert!(
        stairs()
            .intersect_local(&toward_neg_z(0.5, 0.25), 0.0, 3.0)
            .is_none()
    );
}

#[test]
fn full_cube_matches_the_previous_cube_behavior() {
    let cube = Cube::new(v(0.0, 0.0, 0.0), v(1.0, 1.0, 1.0));
    let rays = [
        toward_neg_z(0.3, 0.6),
        Ray::new(v(-2.0, 0.4, 0.7), v(1.0, 0.1, 0.0)),
        Ray::new(v(0.5, 5.0, 0.5), v(0.1, -1.0, 0.05)),
        Ray::new(v(0.5, 0.5, 0.5), v(0.0, 0.0, 1.0)),
        Ray::new(v(3.0, 3.0, 3.0), v(1.0, 1.0, 1.0)),
    ];

    for ray in rays {
        assert_eq!(
            BlockGeometry::FullCube.intersect_local(&ray, 0.0, FAR),
            cube.intersect(&ray, 0.0, FAR)
        );
    }
}
