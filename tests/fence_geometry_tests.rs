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

fn fence(orientation: Orientation) -> BlockGeometry {
    block_geometry(BlockType::Fence, orientation)
}

#[test]
fn fence_is_a_composite_of_post_and_rails() {
    let geometry = fence(Orientation::South);
    let parts = geometry.parts();

    assert_eq!(geometry.kind(), GeometryKind::Composite);
    assert_eq!(parts.len(), 3);
    assert!(parts.iter().all(Prism::is_within_unit_cell));

    // Post: full height, narrow in both horizontal axes.
    assert!(approx(parts[0].size().y, 1.0));
    assert!(parts[0].size().x < 0.3 && parts[0].size().z < 0.3);
    // Rails: span the whole cell along X, thin in the other axes.
    for rail in &parts[1..] {
        assert!(approx(rail.size().x, 1.0));
        assert!(rail.size().y < 0.25 && rail.size().z < 0.25);
    }
}

#[test]
fn fence_is_not_a_full_cube() {
    assert!(!fence(Orientation::South).is_full_cube());
    assert_ne!(fence(Orientation::South), BlockGeometry::FullCube);
}

#[test]
fn ray_hits_the_post() {
    let ray = Ray::new(v(0.5, 0.2, 5.0), v(0.0, 0.0, -1.0));
    let hit = fence(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 0.625));
    assert_eq!(hit.face, Face::PositiveZ);
}

#[test]
fn ray_hits_a_rail_beside_the_post() {
    let ray = Ray::new(v(0.15, 0.45, 5.0), v(0.0, 0.0, -1.0));
    let hit = fence(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.z, 0.5625));
    assert_eq!(hit.face, Face::PositiveZ);
    assert_eq!(hit.normal, v(0.0, 0.0, 1.0));
}

#[test]
fn ray_passes_through_the_gap_between_rails() {
    // y = 0.65 sits between the lower rail (top at 0.5625) and the upper
    // rail (bottom at 0.75), away from the post.
    let ray = Ray::new(v(0.15, 0.65, 5.0), v(0.0, 0.0, -1.0));
    assert!(
        fence(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );
}

#[test]
fn ray_passes_below_and_above_the_rails() {
    for y in [0.2, 0.97] {
        let ray = Ray::new(v(0.15, y, 5.0), v(0.0, 0.0, -1.0));
        assert!(
            fence(Orientation::South)
                .intersect_local(&ray, 0.0, FAR)
                .is_none(),
            "y = {y} should miss"
        );
    }
}

#[test]
fn ray_passes_beside_the_rails_along_the_other_axis() {
    // Along X at z = 0.2 the ray is clear of both the rails and the post.
    let ray = Ray::new(v(-3.0, 0.45, 0.2), v(1.0, 0.0, 0.0));
    assert!(
        fence(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );
}

#[test]
fn ray_from_above_hits_the_post_top() {
    let ray = Ray::new(v(0.5, 5.0, 0.5), v(0.0, -1.0, 0.0));
    let hit = fence(Orientation::South)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.y, 1.0));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn uv_is_within_range() {
    let rays = [
        Ray::new(v(0.5, 0.2, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.15, 0.45, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.15, 5.0, 0.5), v(0.0, -1.0, 0.0)),
        Ray::new(v(-5.0, 0.45, 0.5), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = fence(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}

#[test]
fn east_orientation_turns_the_rails_toward_z() {
    // A ray along Z that hits a rail for South now goes through the gap...
    let along_z = Ray::new(v(0.15, 0.45, 5.0), v(0.0, 0.0, -1.0));
    assert!(
        fence(Orientation::East)
            .intersect_local(&along_z, 0.0, FAR)
            .is_none()
    );

    // ...and a ray along X at the same height/offset now meets the rail.
    let along_x = Ray::new(v(-3.0, 0.45, 0.15), v(1.0, 0.0, 0.0));
    let hit = fence(Orientation::East)
        .intersect_local(&along_x, 0.0, FAR)
        .unwrap();
    assert_eq!(hit.face, Face::NegativeX);
}

#[test]
fn fence_keeps_its_structure_under_every_orientation() {
    for orientation in [
        Orientation::Up,
        Orientation::Down,
        Orientation::North,
        Orientation::South,
        Orientation::East,
        Orientation::West,
    ] {
        let geometry = fence(orientation);
        assert_eq!(geometry.part_count(), 3);
        assert!(geometry.parts().iter().all(Prism::is_within_unit_cell));
    }
}
