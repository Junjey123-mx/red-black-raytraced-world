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

fn core(orientation: Orientation) -> BlockGeometry {
    block_geometry(BlockType::PortalCoreDarkCrimson, orientation)
}

#[test]
fn portal_core_is_a_thin_centered_slab() {
    let geometry = core(Orientation::South);
    let slab = geometry.parts()[0];

    assert_eq!(geometry.kind(), GeometryKind::Prism);
    assert!(slab.is_within_unit_cell());
    assert!(slab.size().z <= 0.15);
    assert!(approx(slab.size().x, 1.0));
    assert!(approx(slab.size().y, 1.0));
    assert!(approx((slab.min().z + slab.max().z) * 0.5, 0.5));
}

#[test]
fn portal_core_is_not_a_full_cube() {
    assert!(!core(Orientation::South).is_full_cube());
}

#[test]
fn ray_hits_the_membrane_from_both_sides() {
    let front = core(Orientation::South)
        .intersect_local(&Ray::new(v(0.5, 0.5, 5.0), v(0.0, 0.0, -1.0)), 0.0, FAR)
        .unwrap();
    let back = core(Orientation::South)
        .intersect_local(&Ray::new(v(0.5, 0.5, -5.0), v(0.0, 0.0, 1.0)), 0.0, FAR)
        .unwrap();

    assert_eq!(front.face, Face::PositiveZ);
    assert!(approx(front.point.z, 0.5625));
    assert_eq!(back.face, Face::NegativeZ);
    assert!(approx(back.point.z, 0.4375));
}

#[test]
fn ray_passes_through_the_rest_of_the_cell() {
    // Parallel to the membrane, in front of it.
    let ray = Ray::new(v(-3.0, 0.5, 0.2), v(1.0, 0.0, 0.0));
    assert!(
        core(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .is_none()
    );

    // Falling straight down beside it.
    let down = Ray::new(v(0.5, 5.0, 0.8), v(0.0, -1.0, 0.0));
    assert!(
        core(Orientation::South)
            .intersect_local(&down, 0.0, FAR)
            .is_none()
    );
}

#[test]
fn oriented_east_the_plane_turns_to_face_x() {
    let geometry = core(Orientation::East);
    let slab = geometry.parts()[0];

    assert!(slab.size().x <= 0.15);
    assert!(approx(slab.size().z, 1.0));

    let hit = geometry
        .intersect_local(&Ray::new(v(5.0, 0.5, 0.5), v(-1.0, 0.0, 0.0)), 0.0, FAR)
        .unwrap();
    assert_eq!(hit.face, Face::PositiveX);

    // A ray along Z at x = 0.2 now runs beside the plane and misses.
    let beside = Ray::new(v(0.2, 0.5, 5.0), v(0.0, 0.0, -1.0));
    assert!(geometry.intersect_local(&beside, 0.0, FAR).is_none());
}

#[test]
fn north_and_south_share_the_same_centered_plane() {
    let north = core(Orientation::North).parts()[0];
    let south = core(Orientation::South).parts()[0];

    assert!(approx(north.min().z, south.min().z));
    assert!(approx(north.max().z, south.max().z));
}

#[test]
fn normals_match_faces_and_uv_is_in_range() {
    let rays = [
        Ray::new(v(0.3, 0.7, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(0.3, 0.7, -5.0), v(0.0, 0.0, 1.0)),
        Ray::new(v(0.3, 5.0, 0.5), v(0.0, -1.0, 0.0)),
        Ray::new(v(-3.0, 0.7, 0.5), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = core(Orientation::South)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert_eq!(hit.normal, hit.face.normal());
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}
