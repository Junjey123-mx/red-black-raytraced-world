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
use core::hit::{Face, HitRecord};
use core::math::Vec3;
use core::ray::Ray;

const EPS: f32 = 1e-4;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn assert_uv(hit: &HitRecord, expected_face: Face, expected_u: f32, expected_v: f32) {
    assert_eq!(hit.face, expected_face);
    assert!(hit.uv.x.is_finite());
    assert!(hit.uv.y.is_finite());
    assert!((0.0..=1.0).contains(&hit.uv.x));
    assert!((0.0..=1.0).contains(&hit.uv.y));
    assert!(
        approx_eq(hit.uv.x, expected_u),
        "u: expected {expected_u}, got {}",
        hit.uv.x
    );
    assert!(
        approx_eq(hit.uv.y, expected_v),
        "v: expected {expected_v}, got {}",
        hit.uv.y
    );
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

// Every non-center point below is deliberately kept strictly inside the two
// secondary axes (never exactly +-1) so the hit lands unambiguously inside a
// single face rather than on an edge or corner shared with a neighbor.

// --- +Z face: u = lx, v = 1 - ly ---

#[test]
fn positive_z_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveZ, 0.5, 0.5);
}

#[test]
fn positive_z_interior_quadrant() {
    let cube = unit_cube();
    // x=-0.5 (lx=0.25), y=0.5 (ly=0.75 -> v=0.25)
    let ray = Ray::new(Vec3::new(-0.5, 0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveZ, 0.25, 0.25);
}

#[test]
fn positive_z_opposite_quadrant() {
    let cube = unit_cube();
    // x=0.5 (lx=0.75), y=-0.5 (ly=0.25 -> v=0.75)
    let ray = Ray::new(Vec3::new(0.5, -0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveZ, 0.75, 0.75);
}

// --- -Z face: u = 1 - lx, v = 1 - ly ---

#[test]
fn negative_z_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeZ, 0.5, 0.5);
}

#[test]
fn negative_z_is_horizontally_mirrored_relative_to_positive_z() {
    let cube = unit_cube();
    // Same (x, y) = (-0.5, 0.5) as `positive_z_interior_quadrant`, but on
    // the opposite face: u must flip (0.75 instead of 0.25) while v stays.
    let ray = Ray::new(Vec3::new(-0.5, 0.5, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeZ, 0.75, 0.25);
}

// --- +X face: u = 1 - lz, v = 1 - ly ---

#[test]
fn positive_x_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveX, 0.5, 0.5);
}

#[test]
fn positive_x_interior_quadrant() {
    let cube = unit_cube();
    // y=0.5 (ly=0.75 -> v=0.25), z=-0.5 (lz=0.25 -> u=0.75)
    let ray = Ray::new(Vec3::new(5.0, 0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveX, 0.75, 0.25);
}

// --- -X face: u = lz, v = 1 - ly ---

#[test]
fn negative_x_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeX, 0.5, 0.5);
}

#[test]
fn negative_x_interior_quadrant() {
    let cube = unit_cube();
    // Same (y, z) = (0.5, -0.5) as `positive_x_interior_quadrant`: u must
    // flip (0.25 instead of 0.75) while v stays, confirming no accidental
    // mirroring shared between +X and -X.
    let ray = Ray::new(Vec3::new(-5.0, 0.5, -0.5), Vec3::new(1.0, 0.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeX, 0.25, 0.25);
}

// --- +Y face (top view): u = lx, v = lz ---

#[test]
fn positive_y_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveY, 0.5, 0.5);
}

#[test]
fn positive_y_interior_quadrant() {
    let cube = unit_cube();
    // x=-0.5 (lx=0.25 -> u=0.25), z=0.5 (lz=0.75 -> v=0.75)
    let ray = Ray::new(Vec3::new(-0.5, 5.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveY, 0.25, 0.75);
}

// --- -Y face (bottom view): u = lx, v = 1 - lz ---

#[test]
fn negative_y_center() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeY, 0.5, 0.5);
}

#[test]
fn negative_y_interior_quadrant() {
    let cube = unit_cube();
    // Same (x, z) = (-0.5, 0.5) as `positive_y_interior_quadrant`: v must
    // flip (0.25 instead of 0.75) while u stays.
    let ray = Ray::new(Vec3::new(-0.5, -5.0, 0.5), Vec3::new(0.0, 1.0, 0.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::NegativeY, 0.25, 0.25);
}

// --- Cross-cutting contracts ---

#[test]
fn hit_from_inside_cube_produces_valid_uv() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0));
    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_uv(&hit, Face::PositiveZ, 0.5, 0.5);
}

#[test]
fn edge_hit_preserves_deterministic_face_and_uv() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

    let hit_a = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let hit_b = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit_a.face, hit_b.face);
    assert!(approx_eq(hit_a.uv.x, hit_b.uv.x));
    assert!(approx_eq(hit_a.uv.y, hit_b.uv.y));
    assert!(hit_a.uv.x.is_finite());
    assert!(hit_a.uv.y.is_finite());
    assert!((0.0..=1.0).contains(&hit_a.uv.x));
    assert!((0.0..=1.0).contains(&hit_a.uv.y));
}

#[test]
fn corner_hit_preserves_deterministic_face_and_uv() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(1.0, 1.0, 1.0));

    let hit_a = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let hit_b = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    // Face selection here is owned by the pre-existing tie-break policy
    // (verified independently in cube_intersection_tests.rs); this test only
    // checks that UV computation is consistent with whichever face wins.
    assert_eq!(hit_a.face, Face::NegativeX);
    assert_eq!(hit_a.face, hit_b.face);
    assert!(approx_eq(hit_a.uv.x, hit_b.uv.x));
    assert!(approx_eq(hit_a.uv.y, hit_b.uv.y));
    assert!(hit_a.uv.x.is_finite());
    assert!(hit_a.uv.y.is_finite());
}

#[test]
fn repeated_calls_produce_identical_face_and_uv() {
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.3, 5.0, 0.2), Vec3::new(0.0, -1.0, 0.0));

    let hit_a = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    let hit_b = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();

    assert_eq!(hit_a.face, hit_b.face);
    assert!(approx_eq(hit_a.uv.x, hit_b.uv.x));
    assert!(approx_eq(hit_a.uv.y, hit_b.uv.y));
}

#[test]
fn translated_cube_produces_same_local_uv_as_origin_cube() {
    let origin_cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    let translated_cube = Cube::new(Vec3::new(9.0, 19.0, -31.0), Vec3::new(11.0, 21.0, -29.0));

    let ray_origin = Ray::new(Vec3::new(0.3, 5.0, 0.2), Vec3::new(0.0, -1.0, 0.0));
    let ray_translated = Ray::new(Vec3::new(10.3, 25.0, -29.8), Vec3::new(0.0, -1.0, 0.0));

    let hit_origin = origin_cube
        .intersect(&ray_origin, 0.0, f32::INFINITY)
        .unwrap();
    let hit_translated = translated_cube
        .intersect(&ray_translated, 0.0, f32::INFINITY)
        .unwrap();

    assert_eq!(hit_origin.face, hit_translated.face);
    assert!(approx_eq(hit_origin.uv.x, hit_translated.uv.x));
    assert!(approx_eq(hit_origin.uv.y, hit_translated.uv.y));
}

#[test]
fn all_six_face_centers_map_near_half_half() {
    let cube = unit_cube();
    let rays = [
        Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0)),
        Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ];

    for ray in rays {
        let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
        assert!(approx_eq(hit.uv.x, 0.5));
        assert!(approx_eq(hit.uv.y, 0.5));
    }
}
