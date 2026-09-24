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

fn door(orientation: Orientation) -> BlockGeometry {
    block_geometry(BlockType::WoodDoor, orientation)
}

#[test]
fn door_is_a_single_thin_prism() {
    let geometry = door(Orientation::South);
    let leaf = geometry.parts()[0];

    assert_eq!(geometry.kind(), GeometryKind::Prism);
    assert_eq!(geometry.part_count(), 1);
    assert!(leaf.is_within_unit_cell());
    assert!(leaf.size().z < 0.25);
    assert!(approx(leaf.size().x, 1.0));
    assert!(approx(leaf.size().y, 1.0));
}

#[test]
fn door_is_not_a_full_cube() {
    assert!(!door(Orientation::South).is_full_cube());
}

#[test]
fn ray_hits_the_leaf_from_the_front() {
    let ray = Ray::new(v(0.5, 0.5, 5.0), v(0.0, 0.0, -1.0));
    let hit = door(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 1.0));
    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn ray_hits_the_leaf_from_behind() {
    let ray = Ray::new(v(0.5, 0.5, -5.0), v(0.0, 0.0, 1.0));
    let hit = door(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 0.8125));
    assert_eq!(hit.face, Face::NegativeZ);
}

#[test]
fn ray_passes_through_the_empty_rest_of_the_cell() {
    // Along X at z = 0.4 the ray is well clear of the leaf.
    let ray = Ray::new(v(-3.0, 0.5, 0.4), v(1.0, 0.0, 0.0));
    assert!(
        door(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );

    // A vertical ray at z = 0.4 also falls through the cell.
    let down = Ray::new(v(0.5, 5.0, 0.4), v(0.0, -1.0, 0.0));
    assert!(
        door(Orientation::South)
            .intersect_local(&down, 0.0, FAR)
            .is_none()
    );
}

#[test]
fn leaf_edge_is_hit_along_its_thickness() {
    let ray = Ray::new(v(-3.0, 0.5, 0.9), v(1.0, 0.0, 0.0));
    let hit = door(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert_eq!(hit.face, Face::NegativeX);
    assert!(approx(hit.point.x, 0.0));
}

#[test]
fn cardinal_orientations_move_the_leaf_to_each_edge() {
    let cases = [
        (
            Orientation::South,
            v(0.5, 0.5, 5.0),
            v(0.0, 0.0, -1.0),
            Face::PositiveZ,
        ),
        (
            Orientation::North,
            v(0.5, 0.5, -5.0),
            v(0.0, 0.0, 1.0),
            Face::NegativeZ,
        ),
        (
            Orientation::East,
            v(5.0, 0.5, 0.5),
            v(-1.0, 0.0, 0.0),
            Face::PositiveX,
        ),
        (
            Orientation::West,
            v(-5.0, 0.5, 0.5),
            v(1.0, 0.0, 0.0),
            Face::NegativeX,
        ),
    ];

    for (orientation, origin, direction, face) in cases {
        let hit = door(orientation)
            .intersect_local(&Ray::new(origin, direction), 0.0, FAR)
            .unwrap();
        assert_eq!(hit.face, face, "{orientation:?}");
        // The leaf sits on that edge, so the hit is at the cell boundary.
        let boundary = hit.point.x.abs().max(hit.point.z.abs());
        assert!(approx(boundary, 1.0) || approx(hit.point.x.min(hit.point.z), 0.0));
    }

    let north = door(Orientation::North).parts()[0];
    assert!(approx(north.min().z, 0.0) && approx(north.max().z, 0.1875));
    let east = door(Orientation::East).parts()[0];
    assert!(approx(east.min().x, 0.8125) && approx(east.max().x, 1.0));
}

#[test]
fn normals_match_the_reported_faces() {
    let rays = [
        Ray::new(v(0.5, 0.5, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.5, 0.5, -5.0), v(0.0, 0.0, 1.0)),
        Ray::new(v(0.5, 5.0, 0.9), v(0.0, -1.0, 0.0)),
        Ray::new(v(-3.0, 0.5, 0.9), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = door(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert_eq!(hit.normal, hit.face.normal());
    }
}

#[test]
fn uv_is_within_range() {
    let rays = [
        Ray::new(v(0.3, 0.7, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.3, 0.7, -5.0), v(0.0, 0.0, 1.0)),
        Ray::new(v(0.3, 5.0, 0.9), v(0.0, -1.0, 0.0)),
        Ray::new(v(-3.0, 0.7, 0.9), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = door(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}
