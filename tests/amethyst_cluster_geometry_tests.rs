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

fn cluster(orientation: Orientation) -> BlockGeometry {
    block_geometry(BlockType::AmethystCluster, orientation)
}

const ALL: [Orientation; 6] = [
    Orientation::Up,
    Orientation::Down,
    Orientation::North,
    Orientation::South,
    Orientation::East,
    Orientation::West,
];

#[test]
fn cluster_is_a_composite_of_several_crystals() {
    let geometry = cluster(Orientation::Up);

    assert_eq!(geometry.kind(), GeometryKind::Composite);
    assert!(geometry.part_count() >= 4);
    assert!(geometry.parts().iter().all(Prism::is_within_unit_cell));
}

#[test]
fn cluster_is_not_a_full_cube() {
    assert!(!cluster(Orientation::Up).is_full_cube());
}

#[test]
fn crystals_have_different_heights_and_stay_slender() {
    let parts = cluster(Orientation::Up).parts();
    let tallest = parts.iter().map(|p| p.max().y).fold(0.0_f32, f32::max);

    assert!(tallest < 1.0, "the cluster should not fill the cell");
    assert!(parts[0].max().y > parts[1].max().y);
    assert!(parts[0].max().y > parts[3].max().y);
    for crystal in &parts {
        assert!(
            approx(crystal.min().y, 0.0),
            "every crystal rests on the base"
        );
        assert!(crystal.size().x <= 0.3 && crystal.size().z <= 0.3);
    }
}

#[test]
fn ray_hits_the_central_crystal_tip() {
    let ray = Ray::new(v(0.5, 5.0, 0.5), v(0.0, -1.0, 0.0));
    let hit = cluster(Orientation::Up)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.y, 0.8125));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn ray_hits_a_shorter_side_crystal_at_its_own_height() {
    let ray = Ray::new(v(0.2, 5.0, 0.55), v(0.0, -1.0, 0.0));
    let hit = cluster(Orientation::Up)
        .intersect_local(&ray, 0.0, FAR)
        .unwrap();

    assert!(approx(hit.point.y, 0.5));
    assert_eq!(hit.face, Face::PositiveY);
}

#[test]
fn ray_passes_through_the_gaps() {
    let geometry = cluster(Orientation::Up);
    let rays = [
        // Straight down in an empty corner.
        Ray::new(v(0.05, 5.0, 0.05), v(0.0, -1.0, 0.0)),
        Ray::new(v(0.95, 5.0, 0.95), v(0.0, -1.0, 0.0)),
        // Sideways, just above the tallest crystal.
        Ray::new(v(-3.0, 0.9, 0.5), v(1.0, 0.0, 0.0)),
        // Between the crystals at low height, along X in the empty corner.
        Ray::new(v(-3.0, 0.2, 0.95), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        assert!(geometry.intersect_local(&ray, 0.0, FAR).is_none());
    }
}

#[test]
fn different_tips_are_hit_at_different_points() {
    let geometry = cluster(Orientation::Up);
    let heights: Vec<f32> = [
        (0.5, 0.5),
        (0.2, 0.55),
        (0.78, 0.45),
        (0.55, 0.78),
        (0.45, 0.2),
    ]
    .iter()
    .map(|&(x, z)| {
        geometry
            .intersect_local(&Ray::new(v(x, 5.0, z), v(0.0, -1.0, 0.0)), 0.0, FAR)
            .unwrap()
            .point
            .y
    })
    .collect();

    for i in 0..heights.len() {
        for j in (i + 1)..heights.len() {
            assert!(!approx(heights[i], heights[j]), "tips {i} and {j} coincide");
        }
    }
}

#[test]
fn normals_match_faces_and_uv_is_in_range() {
    let rays = [
        Ray::new(v(0.5, 5.0, 0.5), v(0.0, -1.0, 0.0)),
        Ray::new(v(0.5, 0.3, 5.0), v(0.0, 0.0, -1.0)),
        Ray::new(v(-3.0, 0.3, 0.55), v(1.0, 0.0, 0.0)),
    ];
    for ray in rays {
        let hit = cluster(Orientation::Up)
            .intersect_local(&ray, 0.0, FAR)
            .unwrap();
        assert_eq!(hit.normal, hit.face.normal());
        assert!((0.0..=1.0).contains(&hit.uv.x));
        assert!((0.0..=1.0).contains(&hit.uv.y));
    }
}

#[test]
fn down_hangs_the_cluster_from_the_top() {
    let parts = cluster(Orientation::Down).parts();
    assert!(parts.iter().all(|p| approx(p.max().y, 1.0)));

    let hit = cluster(Orientation::Down)
        .intersect_local(&Ray::new(v(0.5, -5.0, 0.5), v(0.0, 1.0, 0.0)), 0.0, FAR)
        .unwrap();
    assert_eq!(hit.face, Face::NegativeY);
    assert!(approx(hit.point.y, 1.0 - 0.8125));
}

#[test]
fn cardinal_growth_points_the_tips_sideways() {
    let tall = |o: Orientation| cluster(o).parts()[0];

    let east = tall(Orientation::East);
    assert!(approx(east.min().x, 0.0) && approx(east.max().x, 0.8125));
    let west = tall(Orientation::West);
    assert!(approx(west.max().x, 1.0) && approx(west.min().x, 1.0 - 0.8125));
    let south = tall(Orientation::South);
    assert!(approx(south.min().z, 0.0) && approx(south.max().z, 0.8125));
    let north = tall(Orientation::North);
    assert!(approx(north.max().z, 1.0) && approx(north.min().z, 1.0 - 0.8125));
}

#[test]
fn every_orientation_keeps_the_structure_inside_the_cell() {
    let canonical_volume: f32 = cluster(Orientation::Up)
        .parts()
        .iter()
        .map(|p| p.size().x * p.size().y * p.size().z)
        .sum();

    for orientation in ALL {
        let geometry = cluster(orientation);
        assert_eq!(geometry.part_count(), 5);
        assert!(geometry.parts().iter().all(Prism::is_within_unit_cell));
        assert!(!geometry.is_full_cube());

        let volume: f32 = geometry
            .parts()
            .iter()
            .map(|p| p.size().x * p.size().y * p.size().z)
            .sum();
        assert!((volume - canonical_volume).abs() <= 1e-5);
    }
}
