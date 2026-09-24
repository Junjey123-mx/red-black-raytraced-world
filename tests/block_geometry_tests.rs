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

use core::math::Vec3;
use core::prism::Prism;
use scene::block_geometry::{BlockGeometry, GeometryKind};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn slab() -> Prism {
    Prism::new(v(0.0, 0.0, 0.0), v(1.0, 0.5, 1.0))
}

fn post() -> Prism {
    Prism::new(v(0.4, 0.0, 0.4), v(0.6, 1.0, 0.6))
}

#[test]
fn full_cube_is_one_unit_part() {
    let geometry = BlockGeometry::FullCube;

    assert_eq!(geometry.kind(), GeometryKind::FullCube);
    assert!(geometry.is_full_cube());
    assert_eq!(geometry.part_count(), 1);
    assert_eq!(geometry.parts(), vec![Prism::unit()]);
}

#[test]
fn single_prism_geometry_keeps_its_prism() {
    let geometry = BlockGeometry::prism(slab());

    assert_eq!(geometry.kind(), GeometryKind::Prism);
    assert!(!geometry.is_full_cube());
    assert_eq!(geometry.part_count(), 1);
    assert_eq!(geometry.parts(), vec![slab()]);
}

#[test]
fn composite_geometry_keeps_every_part_in_order() {
    let geometry = BlockGeometry::composite(vec![slab(), post()]).unwrap();

    assert_eq!(geometry.kind(), GeometryKind::Composite);
    assert_eq!(geometry.part_count(), 2);
    assert_eq!(geometry.parts(), vec![slab(), post()]);
}

#[test]
fn composite_part_count_matches_the_input() {
    for count in 1..=5 {
        let parts = vec![post(); count];
        let geometry = BlockGeometry::composite(parts).unwrap();
        assert_eq!(geometry.part_count(), count);
    }
}

#[test]
fn empty_composite_is_rejected() {
    assert!(BlockGeometry::composite(Vec::new()).is_none());
}

#[test]
fn geometry_describes_shape_only() {
    // The value is fully determined by its prisms: two geometries built from
    // the same parts are equal, and there is no material, texture, block
    // type, or world position to distinguish them.
    let a = BlockGeometry::composite(vec![slab(), post()]).unwrap();
    let b = BlockGeometry::composite(vec![slab(), post()]).unwrap();
    assert_eq!(a, b);
    assert_ne!(a, BlockGeometry::FullCube);
}

#[test]
fn parts_stay_inside_the_unit_cell() {
    let geometry = BlockGeometry::composite(vec![slab(), post()]).unwrap();
    assert!(geometry.parts().iter().all(Prism::is_within_unit_cell));
}
