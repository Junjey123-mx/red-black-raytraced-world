#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec3::Vec3;
    }
}

#[path = "."]
mod scene {
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
}

use core::math::Vec3;
use scene::orientation::Orientation;
use std::collections::HashSet;

const ALL: [Orientation; 6] = [
    Orientation::Up,
    Orientation::Down,
    Orientation::North,
    Orientation::South,
    Orientation::East,
    Orientation::West,
];

fn approx_vec(a: Vec3, b: Vec3) -> bool {
    (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6 && (a.z - b.z).abs() < 1e-6
}

#[test]
fn all_six_orientations_exist_and_are_distinct() {
    let set: HashSet<Orientation> = ALL.iter().copied().collect();
    assert_eq!(set.len(), 6);
}

#[test]
fn axis_directions_follow_the_frozen_convention() {
    assert!(approx_vec(
        Orientation::Up.axis_direction(),
        Vec3::new(0.0, 1.0, 0.0)
    ));
    assert!(approx_vec(
        Orientation::Down.axis_direction(),
        Vec3::new(0.0, -1.0, 0.0)
    ));
    assert!(approx_vec(
        Orientation::North.axis_direction(),
        Vec3::new(0.0, 0.0, -1.0)
    ));
    assert!(approx_vec(
        Orientation::South.axis_direction(),
        Vec3::new(0.0, 0.0, 1.0)
    ));
    assert!(approx_vec(
        Orientation::East.axis_direction(),
        Vec3::new(1.0, 0.0, 0.0)
    ));
    assert!(approx_vec(
        Orientation::West.axis_direction(),
        Vec3::new(-1.0, 0.0, 0.0)
    ));
}

#[test]
fn every_axis_direction_is_a_unit_vector() {
    for orientation in ALL {
        assert!((orientation.axis_direction().length() - 1.0).abs() < 1e-6);
    }
}

#[test]
fn up_and_down_are_distinct_and_opposite() {
    assert_ne!(Orientation::Up, Orientation::Down);
    let up = Orientation::Up.axis_direction();
    let down = Orientation::Down.axis_direction();
    assert!(approx_vec(up, down * -1.0));
}

#[test]
fn cardinals_are_mutually_distinguishable() {
    let cardinals = [
        Orientation::North,
        Orientation::South,
        Orientation::East,
        Orientation::West,
    ];
    for i in 0..cardinals.len() {
        for j in (i + 1)..cardinals.len() {
            assert_ne!(cardinals[i], cardinals[j]);
            assert!(!approx_vec(
                cardinals[i].axis_direction(),
                cardinals[j].axis_direction()
            ));
        }
    }
}

#[test]
fn opposite_cardinals_point_in_opposite_directions() {
    assert!(approx_vec(
        Orientation::North.axis_direction(),
        Orientation::South.axis_direction() * -1.0
    ));
    assert!(approx_vec(
        Orientation::East.axis_direction(),
        Orientation::West.axis_direction() * -1.0
    ));
}

#[test]
fn is_vertical_is_true_only_for_up_and_down() {
    assert!(Orientation::Up.is_vertical());
    assert!(Orientation::Down.is_vertical());
    assert!(!Orientation::North.is_vertical());
    assert!(!Orientation::South.is_vertical());
    assert!(!Orientation::East.is_vertical());
    assert!(!Orientation::West.is_vertical());
}

#[test]
fn vertical_orientations_align_with_the_y_axis_only() {
    for orientation in ALL {
        let d = orientation.axis_direction();
        assert_eq!(orientation.is_vertical(), d.y != 0.0);
    }
}

#[test]
fn orientation_is_copy_comparable_and_deterministic() {
    let a = Orientation::West;
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a.axis_direction().x, b.axis_direction().x);
    assert_eq!(a.axis_direction().x, -1.0);
}

#[test]
fn orientation_source_has_no_raylib_dependency() {
    let source = include_str!("../src/scene/orientation.rs");
    assert!(!source.to_lowercase().contains("raylib"));
}
