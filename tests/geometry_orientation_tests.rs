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
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
}

use core::math::Vec3;
use core::prism::Prism;
use scene::block_geometry::BlockGeometry;
use scene::geometry_orientation::{orient_geometry, orient_prism};
use scene::orientation::Orientation;

const EPS: f32 = 1e-5;

const ALL: [Orientation; 6] = [
    Orientation::Up,
    Orientation::Down,
    Orientation::North,
    Orientation::South,
    Orientation::East,
    Orientation::West,
];

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn close(a: Vec3, b: Vec3) -> bool {
    (a.x - b.x).abs() <= EPS && (a.y - b.y).abs() <= EPS && (a.z - b.z).abs() <= EPS
}

/// A slab hugging the +Z (south) edge, lower half.
fn south_slab() -> Prism {
    Prism::new(v(0.0, 0.0, 0.75), v(1.0, 0.5, 1.0))
}

fn stairs() -> BlockGeometry {
    BlockGeometry::composite(vec![
        Prism::new(v(0.0, 0.0, 0.0), v(1.0, 0.5, 1.0)),
        Prism::new(v(0.0, 0.5, 0.0), v(1.0, 1.0, 0.5)),
    ])
    .unwrap()
}

#[test]
fn south_and_up_are_the_canonical_identity() {
    for orientation in [Orientation::South, Orientation::Up] {
        assert_eq!(orient_prism(&south_slab(), orientation), south_slab());
    }
}

#[test]
fn cardinal_rotation_moves_the_south_edge_to_each_side() {
    let north = orient_prism(&south_slab(), Orientation::North);
    assert!(close(north.min(), v(0.0, 0.0, 0.0)));
    assert!(close(north.max(), v(1.0, 0.5, 0.25)));

    let east = orient_prism(&south_slab(), Orientation::East);
    assert!(close(east.min(), v(0.75, 0.0, 0.0)));
    assert!(close(east.max(), v(1.0, 0.5, 1.0)));

    let west = orient_prism(&south_slab(), Orientation::West);
    assert!(close(west.min(), v(0.0, 0.0, 0.0)));
    assert!(close(west.max(), v(0.25, 0.5, 1.0)));
}

#[test]
fn composite_orientation_rotates_every_part() {
    let east = orient_geometry(&stairs(), Orientation::East);
    let parts = east.parts();

    // Lower step still spans the whole floor; upper step now sits at the
    // west side (canonical back z in [0, .5] maps to x in [0, .5]).
    assert!(close(parts[0].min(), v(0.0, 0.0, 0.0)));
    assert!(close(parts[0].max(), v(1.0, 0.5, 1.0)));
    assert!(close(parts[1].min(), v(0.0, 0.5, 0.0)));
    assert!(close(parts[1].max(), v(0.5, 1.0, 1.0)));
}

#[test]
fn down_flips_upside_down_and_keeps_facing() {
    let flipped = orient_geometry(&stairs(), Orientation::Down).parts();

    assert!(close(flipped[0].min(), v(0.0, 0.5, 0.0)));
    assert!(close(flipped[0].max(), v(1.0, 1.0, 1.0)));
    assert!(close(flipped[1].min(), v(0.0, 0.0, 0.0)));
    assert!(close(flipped[1].max(), v(1.0, 0.5, 0.5)));
}

#[test]
fn up_and_down_do_not_break_the_structure() {
    for orientation in [Orientation::Up, Orientation::Down] {
        let oriented = orient_geometry(&stairs(), orientation);
        assert_eq!(oriented.part_count(), 2);
        assert!(oriented.parts().iter().all(Prism::is_within_unit_cell));
    }
}

#[test]
fn bounds_stay_inside_the_unit_cell_for_every_orientation() {
    let odd = Prism::new(v(0.1, 0.15, 0.35), v(0.9, 0.85, 0.65));
    let geometry = BlockGeometry::composite(vec![odd, south_slab()]).unwrap();

    for orientation in ALL {
        let oriented = orient_geometry(&geometry, orientation);
        assert!(oriented.parts().iter().all(Prism::is_within_unit_cell));
    }
}

#[test]
fn part_count_never_changes() {
    for orientation in ALL {
        assert_eq!(orient_geometry(&stairs(), orientation).part_count(), 2);
        assert_eq!(
            orient_geometry(&BlockGeometry::prism(south_slab()), orientation).part_count(),
            1
        );
    }
}

#[test]
fn part_volume_is_preserved() {
    let volume = |p: &Prism| p.size().x * p.size().y * p.size().z;
    for orientation in ALL {
        let oriented = orient_prism(&south_slab(), orientation);
        assert!((volume(&oriented) - volume(&south_slab())).abs() <= EPS);
    }
}

#[test]
fn different_orientations_produce_different_layouts() {
    let layouts: Vec<_> = [
        Orientation::North,
        Orientation::South,
        Orientation::East,
        Orientation::West,
    ]
    .iter()
    .map(|&o| orient_geometry(&stairs(), o))
    .collect();

    for i in 0..layouts.len() {
        for j in (i + 1)..layouts.len() {
            assert_ne!(layouts[i], layouts[j], "layouts {i} and {j} coincide");
        }
    }
    assert_ne!(
        orient_geometry(&stairs(), Orientation::Up),
        orient_geometry(&stairs(), Orientation::Down)
    );
}

#[test]
fn four_cardinal_turns_return_to_the_start() {
    // East applied four times is a full revolution.
    let mut geometry = stairs();
    for _ in 0..4 {
        geometry = orient_geometry(&geometry, Orientation::East);
    }
    for (a, b) in geometry.parts().iter().zip(stairs().parts().iter()) {
        assert!(close(a.min(), b.min()) && close(a.max(), b.max()));
    }
}

#[test]
fn full_cube_is_unchanged_by_any_orientation() {
    for orientation in ALL {
        assert_eq!(
            orient_geometry(&BlockGeometry::FullCube, orientation),
            BlockGeometry::FullCube
        );
    }
}
