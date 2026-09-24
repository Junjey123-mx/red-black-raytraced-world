#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::hit::Face;
use core::material::MaterialId;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{MAX_VOXEL_STEPS, cell_cube, nearest_voxel_hit};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn stone(id: u32) -> BlockInstance {
    BlockInstance::new(BlockType::Stone, MaterialId::new(id), Orientation::Up)
}

fn world_of(cells: &[(IVec3, BlockInstance)]) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    for (position, block) in cells {
        world.insert(*position, *block);
    }
    world
}

fn make_ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

const FAR: f32 = 1000.0;

// ---------------------------------------------------------------------
// Basic hits and misses
// ---------------------------------------------------------------------

#[test]
fn empty_world_returns_none() {
    let world = VoxelWorld::new();
    let ray = make_ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0));
    assert!(nearest_voxel_hit(&world, &ray, FAR).is_none());
}

#[test]
fn a_frontal_voxel_is_hit() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(0, 0, 0));
    assert_eq!(voxel.hit.face, Face::PositiveZ);
    assert!(approx(voxel.hit.distance, 4.0));
    assert!(approx(voxel.hit.point.z, 1.0));
}

#[test]
fn several_empty_cells_before_the_voxel_are_skipped_without_false_hits() {
    let world = world_of(&[(cell(0, 0, 3), stone(1))]);
    let ray = make_ray((0.5, 0.5, 10.0), (0.0, 0.0, -1.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(0, 0, 3));
    assert_eq!(voxel.hit.face, Face::PositiveZ);
    assert!(approx(voxel.hit.distance, 6.0));
}

#[test]
fn the_nearest_of_two_aligned_voxels_wins_regardless_of_insertion_order() {
    let near = (cell(0, 0, 2), stone(1));
    let far = (cell(0, 0, -2), stone(2));
    let ray = make_ray((0.5, 0.5, 10.0), (0.0, 0.0, -1.0));

    for cells in [[near, far], [far, near]] {
        let world = world_of(&cells);
        let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");
        assert_eq!(voxel.cell, cell(0, 0, 2));
        assert_eq!(voxel.block.material_id(), MaterialId::new(1));
        assert!(approx(voxel.hit.distance, 7.0));
    }
}

#[test]
fn a_ray_that_misses_every_voxel_returns_none() {
    let world = world_of(&[
        (cell(0, 0, 0), stone(1)),
        (cell(1, 0, 0), stone(1)),
        (cell(0, 1, 0), stone(1)),
    ]);
    let ray = make_ray((5.5, 5.5, 10.0), (0.0, 0.0, -1.0));
    assert!(nearest_voxel_hit(&world, &ray, FAR).is_none());
}

// ---------------------------------------------------------------------
// Negative coordinates and negative directions
// ---------------------------------------------------------------------

#[test]
fn a_voxel_in_negative_coordinates_is_hit() {
    let world = world_of(&[(cell(-3, -2, -4), stone(7))]);
    let ray = make_ray((-2.5, -1.5, 5.0), (0.0, 0.0, -1.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(-3, -2, -4));
    assert_eq!(voxel.hit.face, Face::PositiveZ);
    assert!(approx(voxel.hit.distance, 8.0)); // top of cell z = -4 is z = -3
    assert!(approx(voxel.hit.uv.x, 0.5));
    assert!(approx(voxel.hit.uv.y, 0.5));
}

#[test]
fn a_ray_with_negative_direction_hits_the_facing_side() {
    let world = world_of(&[(cell(2, 0, 0), stone(1))]);
    let ray = make_ray((10.5, 0.5, 0.5), (-1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(2, 0, 0));
    assert_eq!(voxel.hit.face, Face::PositiveX);
    assert!(approx(voxel.hit.distance, 7.5));
}

#[test]
fn a_ray_from_negative_space_toward_positive_hits_the_negative_face() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((-6.5, 0.5, 0.5), (1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");
    assert_eq!(voxel.hit.face, Face::NegativeX);
    assert!(approx(voxel.hit.distance, 6.5));
}

// ---------------------------------------------------------------------
// Origin inside a voxel
// ---------------------------------------------------------------------

#[test]
fn a_ray_starting_inside_a_voxel_returns_its_exit_hit() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((0.5, 0.5, 0.5), (1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(0, 0, 0));
    assert_eq!(voxel.hit.face, Face::PositiveX);
    assert!(approx(voxel.hit.distance, 0.5));
    assert!(voxel.hit.distance > 0.0);
}

#[test]
fn an_inside_origin_exit_hit_wins_over_a_touching_neighbor() {
    let world = world_of(&[(cell(0, 0, 0), stone(1)), (cell(1, 0, 0), stone(2))]);
    let ray = make_ray((0.5, 0.5, 0.5), (1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");
    assert_eq!(voxel.cell, cell(0, 0, 0));
    assert!(approx(voxel.hit.distance, 0.5));
}

// ---------------------------------------------------------------------
// An occupied cell is a candidate, not an automatic hit
// ---------------------------------------------------------------------

#[test]
fn an_occupied_start_cell_only_touched_at_its_boundary_is_not_a_hit() {
    // The ray starts exactly on x = 1 moving toward -X: DDA nominates cell
    // (1,0,0) first, but the ray never enters it, so Cube::intersect
    // rejects it and no hit is invented.
    let world = world_of(&[(cell(1, 0, 0), stone(1))]);
    let ray = make_ray((1.0, 0.5, 0.5), (-1.0, 0.0, 0.0));
    assert!(nearest_voxel_hit(&world, &ray, FAR).is_none());
}

#[test]
fn traversal_continues_past_a_rejected_candidate_to_the_real_hit() {
    let world = world_of(&[(cell(1, 0, 0), stone(1)), (cell(0, 0, 0), stone(2))]);
    let ray = make_ray((1.0, 0.5, 0.5), (-1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");
    assert_eq!(voxel.cell, cell(0, 0, 0));
    assert_eq!(voxel.hit.face, Face::NegativeX);
    assert!(approx(voxel.hit.distance, 1.0));
}

// ---------------------------------------------------------------------
// The hit preserves block identity and the geometric HitRecord
// ---------------------------------------------------------------------

fn tagged_world() -> VoxelWorld {
    // A full-cube block type: WoodStairs became a partial shape in Gate 06,
    // and this fixture only needs *some* recognizable non-Stone type.
    let block = BlockInstance::new(
        BlockType::DoubleWoodSlab,
        MaterialId::new(42),
        Orientation::Down,
    );
    world_of(&[(cell(1, 2, 3), block)])
}

fn tagged_ray() -> Ray {
    make_ray((1.25, 2.75, 10.0), (0.0, 0.0, -1.0))
}

#[test]
fn the_hit_preserves_block_type() {
    let voxel = nearest_voxel_hit(&tagged_world(), &tagged_ray(), FAR).unwrap();
    assert_eq!(voxel.block.block_type(), BlockType::DoubleWoodSlab);
}

#[test]
fn the_hit_preserves_material_id() {
    let voxel = nearest_voxel_hit(&tagged_world(), &tagged_ray(), FAR).unwrap();
    assert_eq!(voxel.block.material_id(), MaterialId::new(42));
}

#[test]
fn the_hit_preserves_orientation() {
    let voxel = nearest_voxel_hit(&tagged_world(), &tagged_ray(), FAR).unwrap();
    assert_eq!(voxel.block.orientation(), Orientation::Down);
}

#[test]
fn the_hit_preserves_face_and_normal() {
    let voxel = nearest_voxel_hit(&tagged_world(), &tagged_ray(), FAR).unwrap();
    assert_eq!(voxel.hit.face, Face::PositiveZ);
    assert!(approx(voxel.hit.normal.x, 0.0));
    assert!(approx(voxel.hit.normal.y, 0.0));
    assert!(approx(voxel.hit.normal.z, 1.0));
}

#[test]
fn the_hit_uv_is_valid_and_matches_the_frozen_cube_mapping() {
    let voxel = nearest_voxel_hit(&tagged_world(), &tagged_ray(), FAR).unwrap();

    assert!((0.0..=1.0).contains(&voxel.hit.uv.x));
    assert!((0.0..=1.0).contains(&voxel.hit.uv.y));
    // +Z: u = lx = 0.25, v = 1 - ly = 1 - 0.75.
    assert!(approx(voxel.hit.uv.x, 0.25));
    assert!(approx(voxel.hit.uv.y, 0.25));
}

#[test]
fn distance_and_point_are_coherent_with_the_ray() {
    let ray = tagged_ray();
    let voxel = nearest_voxel_hit(&tagged_world(), &ray, FAR).unwrap();

    assert!(approx(voxel.hit.distance, 6.0));
    let along_ray = ray.at(voxel.hit.distance);
    assert!(approx(voxel.hit.point.x, along_ray.x));
    assert!(approx(voxel.hit.point.y, along_ray.y));
    assert!(approx(voxel.hit.point.z, along_ray.z));
}

// ---------------------------------------------------------------------
// Bounded traversal
// ---------------------------------------------------------------------

#[test]
fn max_distance_excludes_a_voxel_that_lies_beyond_it() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((0.5, 0.5, 10.0), (0.0, 0.0, -1.0));

    assert!(nearest_voxel_hit(&world, &ray, 5.0).is_none());
    assert!(nearest_voxel_hit(&world, &ray, 9.5).is_some());
}

#[test]
fn a_ray_leaving_the_world_terminates_even_with_unbounded_distance() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((0.5, 0.5, 10.0), (0.0, 0.0, 1.0)); // pointing away

    assert!(nearest_voxel_hit(&world, &ray, f32::INFINITY).is_none());
    assert!(MAX_VOXEL_STEPS > 0);
}

#[test]
fn a_far_voxel_beyond_the_step_cap_is_not_reached_but_the_call_still_returns() {
    let world = world_of(&[(cell(0, 0, 100_000), stone(1))]);
    let ray = make_ray((0.5, 0.5, 200_000.0), (0.0, 0.0, -1.0));

    assert!(nearest_voxel_hit(&world, &ray, f32::INFINITY).is_none());
}

#[test]
fn a_non_positive_or_nan_max_distance_returns_none() {
    let world = world_of(&[(cell(0, 0, 0), stone(1))]);
    let ray = make_ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0));

    assert!(nearest_voxel_hit(&world, &ray, 0.0).is_none());
    assert!(nearest_voxel_hit(&world, &ray, -3.0).is_none());
    assert!(nearest_voxel_hit(&world, &ray, f32::NAN).is_none());
}

// ---------------------------------------------------------------------
// Diagonal traversal and deterministic edge/corner behavior
// ---------------------------------------------------------------------

#[test]
fn a_diagonal_ray_finds_the_first_voxel_along_its_path() {
    // The ray x = -0.6 + s, y = -0.4 + s, z = -0.2 + s passes through
    // cells (1,1,1) and (4,4,4); (1,1,1) is nearer. Decoys sit off-path.
    let world = world_of(&[
        (cell(4, 4, 4), stone(4)),
        (cell(1, 1, 1), stone(1)),
        (cell(0, 3, 0), stone(9)),
        (cell(3, 0, 3), stone(9)),
    ]);
    let ray = make_ray((-0.6, -0.4, -0.2), (1.0, 1.0, 1.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");

    assert_eq!(voxel.cell, cell(1, 1, 1));
    assert_eq!(voxel.block.material_id(), MaterialId::new(1));
    assert_eq!(voxel.hit.face, Face::NegativeX);
    assert!(approx(voxel.hit.distance, 1.6 * 3.0_f32.sqrt()));
}

#[test]
fn removing_the_nearer_diagonal_voxel_reveals_the_farther_one() {
    let world = world_of(&[(cell(4, 4, 4), stone(4))]);
    let ray = make_ray((-0.6, -0.4, -0.2), (1.0, 1.0, 1.0));

    let voxel = nearest_voxel_hit(&world, &ray, FAR).expect("must hit");
    assert_eq!(voxel.cell, cell(4, 4, 4));
    assert_eq!(voxel.hit.face, Face::NegativeX);
    assert!(approx(voxel.hit.distance, 4.6 * 3.0_f32.sqrt()));
}

#[test]
fn a_corner_grazing_ray_does_not_hit_cells_it_only_touches_at_a_point() {
    // The ray runs exactly along the main diagonal through the corner
    // (2,2,2). These six cells touch that corner only at a single point;
    // the DDA's simultaneous-crossing policy never nominates them.
    let world = world_of(&[
        (cell(2, 1, 1), stone(1)),
        (cell(1, 2, 1), stone(1)),
        (cell(1, 1, 2), stone(1)),
        (cell(2, 2, 1), stone(1)),
        (cell(2, 1, 2), stone(1)),
        (cell(1, 2, 2), stone(1)),
    ]);
    let ray = make_ray((-0.5, -0.5, -0.5), (1.0, 1.0, 1.0));

    assert!(nearest_voxel_hit(&world, &ray, 50.0).is_none());
}

#[test]
fn a_corner_hit_is_deterministic_and_keeps_a_valid_uv() {
    let world = world_of(&[(cell(2, 2, 2), stone(1))]);
    let ray = make_ray((-0.5, -0.5, -0.5), (1.0, 1.0, 1.0));

    let first = nearest_voxel_hit(&world, &ray, FAR).expect("must hit the corner cell");
    let second = nearest_voxel_hit(&world, &ray, FAR).expect("must hit the corner cell");

    assert_eq!(first, second);
    assert_eq!(first.cell, cell(2, 2, 2));
    assert!(matches!(
        first.hit.face,
        Face::NegativeX | Face::NegativeY | Face::NegativeZ
    ));
    assert!((0.0..=1.0).contains(&first.hit.uv.x));
    assert!((0.0..=1.0).contains(&first.hit.uv.y));
}

#[test]
fn an_edge_grazing_ray_is_deterministic_across_calls() {
    let world = world_of(&[(cell(3, 0, 0), stone(1))]);
    // Travels exactly along the y = 1 / z = 0 edge of cell (3,0,0).
    let ray = make_ray((-2.0, 1.0, 0.0), (1.0, 0.0, 0.0));

    let first = nearest_voxel_hit(&world, &ray, FAR);
    let second = nearest_voxel_hit(&world, &ray, FAR);
    assert_eq!(first, second);
}

// ---------------------------------------------------------------------
// The per-cell cube covers exactly [x, x+1) x [y, y+1) x [z, z+1)
// ---------------------------------------------------------------------

#[test]
fn cell_cube_spans_exactly_one_unit_from_the_cell_coordinate() {
    let cube = cell_cube(cell(-2, 3, 5));

    let from_low = make_ray((-10.0, 3.5, 5.5), (1.0, 0.0, 0.0));
    let entry = cube.intersect(&from_low, 0.0, FAR).unwrap();
    assert_eq!(entry.face, Face::NegativeX);
    assert!(approx(entry.point.x, -2.0));

    let from_high = make_ray((10.0, 3.5, 5.5), (-1.0, 0.0, 0.0));
    let other = cube.intersect(&from_high, 0.0, FAR).unwrap();
    assert_eq!(other.face, Face::PositiveX);
    assert!(approx(other.point.x, -1.0));

    let from_top = make_ray((-1.5, 10.0, 5.5), (0.0, -1.0, 0.0));
    assert!(approx(
        cube.intersect(&from_top, 0.0, FAR).unwrap().point.y,
        4.0
    ));
    let from_front = make_ray((-1.5, 3.5, 10.0), (0.0, 0.0, -1.0));
    assert!(approx(
        cube.intersect(&from_front, 0.0, FAR).unwrap().point.z,
        6.0
    ));
}

#[test]
fn adjacent_cell_cubes_leave_no_gap() {
    let left = cell_cube(cell(0, 0, 0));
    let right = cell_cube(cell(1, 0, 0));

    // A ray crossing x = 1 hits `left` from inside at x = 1 and `right`
    // from outside at x = 1: the shared plane is exactly the same.
    let ray = make_ray((0.5, 0.5, 0.5), (1.0, 0.0, 0.0));
    let exit = left.intersect(&ray, 0.0, FAR).unwrap();
    let entry_ray = make_ray((-5.0, 0.5, 0.5), (1.0, 0.0, 0.0));
    let entry = right.intersect(&entry_ray, 0.0, FAR).unwrap();

    assert!(approx(exit.point.x, 1.0));
    assert!(approx(entry.point.x, 1.0));
}
