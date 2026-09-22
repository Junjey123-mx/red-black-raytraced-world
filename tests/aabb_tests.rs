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

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
}

use core::aabb::Aabb;
use core::math::Vec3;
use core::ray::Ray;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn canonical_bounds_are_preserved() {
    let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

    assert!(approx_eq(aabb.min.x, -1.0));
    assert!(approx_eq(aabb.min.y, -1.0));
    assert!(approx_eq(aabb.min.z, -1.0));
    assert!(approx_eq(aabb.max.x, 1.0));
    assert!(approx_eq(aabb.max.y, 1.0));
    assert!(approx_eq(aabb.max.z, 1.0));
}

#[test]
fn negative_bounds_are_valid() {
    let aabb = Aabb::new(Vec3::new(-5.0, -6.0, -7.0), Vec3::new(-2.0, -1.0, -3.0));

    assert!(approx_eq(aabb.min.x, -5.0));
    assert!(approx_eq(aabb.min.y, -6.0));
    assert!(approx_eq(aabb.min.z, -7.0));
    assert!(approx_eq(aabb.max.x, -2.0));
    assert!(approx_eq(aabb.max.y, -1.0));
    assert!(approx_eq(aabb.max.z, -3.0));
}

#[test]
fn constructor_normalizes_inverted_extremes() {
    let aabb = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0));

    assert!(aabb.min.x <= aabb.max.x);
    assert!(aabb.min.y <= aabb.max.y);
    assert!(aabb.min.z <= aabb.max.z);

    assert!(approx_eq(aabb.min.x, -1.0));
    assert!(approx_eq(aabb.max.x, 1.0));
}

#[test]
fn constructor_normalizes_mixed_component_order() {
    let aabb = Aabb::new(Vec3::new(-2.0, 5.0, 0.0), Vec3::new(3.0, -1.0, -4.0));

    assert!(approx_eq(aabb.min.x, -2.0));
    assert!(approx_eq(aabb.max.x, 3.0));
    assert!(approx_eq(aabb.min.y, -1.0));
    assert!(approx_eq(aabb.max.y, 5.0));
    assert!(approx_eq(aabb.min.z, -4.0));
    assert!(approx_eq(aabb.max.z, 0.0));
}

#[test]
fn center_and_half_extent_are_correct() {
    let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 2.0, 6.0));

    let center = aabb.center();
    assert!(approx_eq(center.x, 2.0));
    assert!(approx_eq(center.y, 1.0));
    assert!(approx_eq(center.z, 3.0));

    let half_extent = aabb.half_extent();
    assert!(approx_eq(half_extent.x, 2.0));
    assert!(approx_eq(half_extent.y, 1.0));
    assert!(approx_eq(half_extent.z, 3.0));
}

#[test]
fn bounds_stay_finite() {
    let aabb = Aabb::new(
        Vec3::new(-100.0, 50.0, -25.0),
        Vec3::new(100.0, -50.0, 25.0),
    );

    assert!(aabb.min.x.is_finite());
    assert!(aabb.min.y.is_finite());
    assert!(aabb.min.z.is_finite());
    assert!(aabb.max.x.is_finite());
    assert!(aabb.max.y.is_finite());
    assert!(aabb.max.z.is_finite());
}

fn unit_cube_at_origin() -> Aabb {
    Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

#[test]
fn frontal_hit_has_expected_distance() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let hit = aabb.intersect(&ray, 0.0, f32::INFINITY);
    assert!(hit.is_some());

    let (t_enter, t_exit) = hit.unwrap();
    assert!(approx_eq(t_enter, 4.0));
    assert!(approx_eq(t_exit, 6.0));
}

#[test]
fn lateral_miss_returns_none() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    assert!(aabb.intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn ray_originating_inside_hits_with_negative_enter() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0));

    let hit = aabb.intersect(&ray, 0.0, f32::INFINITY);
    assert!(hit.is_some());

    let (t_enter, t_exit) = hit.unwrap();
    assert!(approx_eq(t_enter, 0.0));
    assert!(approx_eq(t_exit, 1.0));
}

#[test]
fn negative_direction_hits_correctly() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    let hit = aabb.intersect(&ray, 0.0, f32::INFINITY);
    assert!(hit.is_some());

    let (t_enter, t_exit) = hit.unwrap();
    assert!(approx_eq(t_enter, 4.0));
    assert!(approx_eq(t_exit, 6.0));
}

#[test]
fn parallel_ray_outside_slab_misses() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(5.0, 5.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    assert!(aabb.intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn parallel_ray_inside_slab_hits() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    assert!(aabb.intersect(&ray, 0.0, f32::INFINITY).is_some());
}

#[test]
fn box_behind_origin_returns_none() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 1.0));

    assert!(aabb.intersect(&ray, 0.0, f32::INFINITY).is_none());
}

#[test]
fn near_zero_direction_components_do_not_panic_or_produce_nan() {
    let aabb = unit_cube_at_origin();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(1e-9, 1e-9, -1.0));

    if let Some((t_enter, t_exit)) = aabb.intersect(&ray, 0.0, f32::INFINITY) {
        assert!(t_enter.is_finite());
        assert!(t_exit.is_finite());
    }
}

#[test]
fn no_hit_ever_produces_nan_components() {
    let aabb = unit_cube_at_origin();
    let rays = [
        Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Ray::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0)),
        Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 1.0)),
    ];

    for ray in rays {
        if let Some((t_enter, t_exit)) = aabb.intersect(&ray, 0.0, f32::INFINITY) {
            assert!(!t_enter.is_nan());
            assert!(!t_exit.is_nan());
        }
    }
}
