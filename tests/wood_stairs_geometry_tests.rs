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
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
}

use core::hit::Face;
use core::math::Vec3;
use core::prism::Prism;
use core::ray::Ray;
use scene::block_geometry::{BlockGeometry, GeometryKind};
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::orientation::Orientation;

const EPS: f32 = 1e-4;
const FAR: f32 = f32::INFINITY;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn stairs(orientation: Orientation) -> BlockGeometry {
    block_geometry(BlockType::WoodStairs, orientation)
}

#[test]
fn stairs_are_two_prisms() {
    let geometry = stairs(Orientation::South);

    assert_eq!(geometry.kind(), GeometryKind::Composite);
    assert_eq!(geometry.part_count(), 2);
    assert!(geometry.parts().iter().all(Prism::is_within_unit_cell));
}

#[test]
fn stairs_are_not_a_full_cube() {
    assert!(!stairs(Orientation::South).is_full_cube());
    assert_ne!(stairs(Orientation::South), BlockGeometry::FullCube);
}

#[test]
fn cardinal_orientations_give_four_distinct_layouts() {
    let all = [
        stairs(Orientation::North),
        stairs(Orientation::South),
        stairs(Orientation::East),
        stairs(Orientation::West),
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j]);
        }
    }
}

#[test]
fn ray_hits_the_lower_step_riser() {
    let ray = Ray::new(v(0.5, 0.25, 5.0), v(0.0, 0.0, -1.0));
    let hit = stairs(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 1.0));
    assert_eq!(hit.face, Face::PositiveZ);
    assert_eq!(hit.normal, v(0.0, 0.0, 1.0));
}

#[test]
fn ray_hits_the_upper_step_riser() {
    let ray = Ray::new(v(0.5, 0.75, 5.0), v(0.0, 0.0, -1.0));
    let hit = stairs(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 0.5));
    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn ray_hits_the_lower_step_tread_in_front_of_the_upper_step() {
    let ray = Ray::new(v(0.5, 5.0, 0.75), v(0.0, -1.0, 0.0));
    let hit = stairs(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.y, 0.5));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn ray_through_the_empty_notch_misses() {
    // Above the lower step (y > 0.5) and in front of the upper step
    // (z > 0.5) the cell is empty air.
    let ray = Ray::new(v(-3.0, 0.75, 0.75), v(1.0, 0.0, 0.0));
    assert!(
        stairs(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );

    let ray_z = Ray::new(v(0.5, 0.75, 5.0), v(0.0, -0.05, -1.0));
    // Grazes over the lower step and stops on the upper riser instead of
    // passing through the notch to the far side.
    let hit = stairs(Orientation::South)
        .intersect_local(&ray_z, 0.0, FAR)
        .unwrap();
    assert!(hit.point.z <= 0.5 + EPS);
}

#[test]
fn rotated_stairs_move_the_notch() {
    // The same ray goes through the notch for South stairs but strikes the
    // upper step once the stairs are turned to face North.
    let ray = Ray::new(v(-3.0, 0.75, 0.75), v(1.0, 0.0, 0.0));

    assert!(
        stairs(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );
    let hit = stairs(Orientation::North)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();
    assert_eq!(hit.face, Face::NegativeX);
}

#[test]
fn east_stairs_face_positive_x() {
    // Tall part on the west side: a ray at the top from +X sails over the
    // low half and hits the upper step's face at x = 0.5.
    let ray = Ray::new(v(5.0, 0.75, 0.5), v(-1.0, 0.0, 0.0));
    let hit = stairs(Orientation::East)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.x, 0.5));
    assert_eq!(hit.face, Face::PositiveX);
}

#[test]
fn uv_is_within_range_on_every_visible_face() {
    let rays = [
        Ray::new(v(0.3, 0.25, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.3, 0.75, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.3, 5.0, 0.75), v(0.0, -1.0, 0.0)),
        Ray::new(v(0.3, 5.0, 0.25), v(0.0, -1.0, 0.0)),
        Ray::new(v(-5.0, 0.25, 0.6), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = stairs(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}

#[test]
fn upside_down_stairs_open_downward() {
    // Down flips the stairs: the tall half now hangs at the bottom back, so
    // a ray from below at z = 0.75 hits the flipped lower slab (now on top),
    // reaching y = 0.5 from below.
    let ray = Ray::new(v(0.5, -5.0, 0.75), v(0.0, 1.0, 0.0));
    let hit = stairs(Orientation::Down)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.y, 0.5));
    assert_eq!(hit.face, Face::NegativeY);
}

#[test]
fn other_block_types_are_still_full_cubes() {
    assert!(block_geometry(BlockType::Stone, Orientation::Up).is_full_cube());
    assert!(block_geometry(BlockType::Grass, Orientation::South).is_full_cube());
}
