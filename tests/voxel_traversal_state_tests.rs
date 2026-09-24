#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::math::{IVec3, Vec3};
use renderer::voxel_traversal::{
    AXIS_EPSILON, DdaState, axis_step, cell_from_point, safe_reciprocal_magnitude,
};

fn code_of(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_token(code: &str, token: &str) -> bool {
    code.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == token)
}

#[test]
fn state_represents_cell_step_and_boundary_distances() {
    let state = DdaState::new(
        IVec3::new(3, -2, 5),
        IVec3::new(1, -1, 0),
        Vec3::new(0.25, 0.5, f32::INFINITY),
        Vec3::new(1.0, 2.0, f32::INFINITY),
    );

    assert_eq!(state.cell, IVec3::new(3, -2, 5));
    assert_eq!(state.step, IVec3::new(1, -1, 0));
    assert_eq!(state.t_max.x, 0.25);
    assert_eq!(state.t_max.y, 0.5);
    assert_eq!(state.t_delta.x, 1.0);
    assert_eq!(state.t_delta.y, 2.0);
}

#[test]
fn positive_direction_steps_plus_one() {
    assert_eq!(axis_step(1.0), 1);
    assert_eq!(axis_step(0.3), 1);
    assert_eq!(axis_step(AXIS_EPSILON * 2.0), 1);
}

#[test]
fn negative_direction_steps_minus_one() {
    assert_eq!(axis_step(-1.0), -1);
    assert_eq!(axis_step(-0.3), -1);
    assert_eq!(axis_step(-AXIS_EPSILON * 2.0), -1);
}

#[test]
fn near_zero_direction_steps_zero() {
    assert_eq!(axis_step(0.0), 0);
    assert_eq!(axis_step(-0.0), 0);
    assert_eq!(axis_step(AXIS_EPSILON), 0);
    assert_eq!(axis_step(-AXIS_EPSILON), 0);
    assert_eq!(axis_step(1e-9), 0);
}

#[test]
fn non_finite_direction_never_selects_an_axis() {
    assert_eq!(axis_step(f32::NAN), 0);
}

#[test]
fn inactive_axes_produce_infinity_not_nan() {
    for component in [0.0_f32, -0.0, 1e-9, -1e-9, AXIS_EPSILON, f32::NAN] {
        let value = safe_reciprocal_magnitude(component);
        assert!(!value.is_nan(), "{component} produced NaN");
        assert!(value.is_infinite() && value > 0.0, "{component} -> {value}");
    }
}

#[test]
fn active_axes_have_finite_positive_reciprocal_magnitude() {
    assert_eq!(safe_reciprocal_magnitude(1.0), 1.0);
    assert_eq!(safe_reciprocal_magnitude(-1.0), 1.0);
    assert_eq!(safe_reciprocal_magnitude(0.5), 2.0);
    assert_eq!(safe_reciprocal_magnitude(-0.25), 4.0);
    assert!(safe_reciprocal_magnitude(0.001).is_finite());
}

#[test]
fn cell_from_point_uses_floor_for_positive_coordinates() {
    assert_eq!(
        cell_from_point(Vec3::new(0.2, 0.7, 0.1)),
        IVec3::new(0, 0, 0)
    );
    assert_eq!(
        cell_from_point(Vec3::new(0.999, 0.0, 0.5)),
        IVec3::new(0, 0, 0)
    );
    assert_eq!(
        cell_from_point(Vec3::new(1.0, 2.5, 3.9)),
        IVec3::new(1, 2, 3)
    );
}

#[test]
fn cell_from_point_uses_floor_for_negative_coordinates() {
    assert_eq!(
        cell_from_point(Vec3::new(-0.1, 0.0, 0.0)),
        IVec3::new(-1, 0, 0)
    );
    assert_eq!(
        cell_from_point(Vec3::new(-1.0, -1.0, -1.0)),
        IVec3::new(-1, -1, -1)
    );
    assert_eq!(
        cell_from_point(Vec3::new(-1.2, -0.5, -2.0)),
        IVec3::new(-2, -1, -2)
    );
}

#[test]
fn cell_from_point_does_not_truncate_toward_zero() {
    // Truncation would give 0 for all of these; floor must not.
    for value in [-0.001_f32, -0.5, -0.999] {
        assert_eq!(
            cell_from_point(Vec3::new(value, value, value)),
            IVec3::new(-1, -1, -1)
        );
    }
}

#[test]
fn state_source_does_not_reference_world_materials_or_textures() {
    let code = code_of(include_str!("../src/renderer/voxel_traversal.rs"));
    for forbidden in [
        "VoxelWorld",
        "BlockInstance",
        "BlockType",
        "Material",
        "MaterialId",
        "CpuTexture",
        "TextureManager",
        "Cube",
    ] {
        assert!(
            !has_token(&code, forbidden),
            "DDA state mentions {forbidden}"
        );
    }
}
