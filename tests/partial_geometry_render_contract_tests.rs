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
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;

    pub use camera::Camera;
    pub use projection::primary_ray;
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
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
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

use std::collections::HashMap;

use camera::{Camera, primary_ray};
use core::color::Color;
use core::face_textures::FaceTextures;
use core::hit::Face;
use core::material::{Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use renderer::raytracer::{
    MISSING_MATERIAL_COLOR, VoxelHit, cast_ray_voxel_lit, cell_cube, is_voxel_occluded,
    nearest_voxel_hit,
};
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light, PointLight};
use scene::orientation::Orientation;
use scene::scene::{
    diagnostic_partial_materials, diagnostic_partial_voxel_world, stone_material_id,
    wood_material_id,
};
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 24.0;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn stone() -> BlockInstance {
    BlockInstance::new(BlockType::Stone, stone_material_id(), Orientation::Up)
}

fn wood(block_type: BlockType, orientation: Orientation) -> BlockInstance {
    BlockInstance::new(block_type, wood_material_id(), orientation)
}

fn world_of(cells: &[(IVec3, BlockInstance)]) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    for (position, block) in cells {
        world.insert(*position, *block);
    }
    world
}

/// The same camera parameters `app.rs` uses for the mixed scene.
fn app_camera() -> Camera {
    Camera::new(
        Vec3::new(3.4, 3.4, 7.0),
        Vec3::new(3.0, 1.3, 1.9),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        800.0 / 600.0,
    )
}

fn grass_face_textures() -> (TextureManager, FaceTextures) {
    let dir = format!(
        "{}/assets/textures/overworld/grass",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut manager = TextureManager::new();
    let top = manager.load(format!("{dir}/top.png")).unwrap();
    let side = manager.load(format!("{dir}/side.png")).unwrap();
    let bottom = manager.load(format!("{dir}/bottom.png")).unwrap();
    (
        manager,
        FaceTextures::new(side, side, top, bottom, side, side),
    )
}

/// Independent oracle: intersect the ray with the resolved local geometry of
/// *every* cell in a box and keep the nearest. Tests only; the renderer
/// never does this.
fn brute_force_nearest(world: &VoxelWorld, ray: &Ray) -> Option<VoxelHit> {
    let mut best: Option<VoxelHit> = None;
    for x in -2..9 {
        for y in -2..5 {
            for z in -2..7 {
                let position = cell(x, y, z);
                let Some(block) = world.get(position) else {
                    continue;
                };
                let origin = Vec3::new(x as f32, y as f32, z as f32);
                let geometry =
                    block_geometry(block.block_type(), block.orientation()).translated(origin);
                if let Some(hit) = geometry.intersect_local(ray, 0.0, RANGE) {
                    let closer = best
                        .as_ref()
                        .is_none_or(|current| hit.distance < current.hit.distance);
                    if closer {
                        best = Some(VoxelHit {
                            cell: position,
                            block: *block,
                            hit,
                        });
                    }
                }
            }
        }
    }
    best
}

// ---------------------------------------------------------------------
// 1. BlockType + Orientation resolve the expected local geometry.
// ---------------------------------------------------------------------

#[test]
fn block_type_and_orientation_resolve_the_expected_geometry() {
    assert!(block_geometry(BlockType::Stone, Orientation::Up).is_full_cube());
    assert_eq!(
        block_geometry(BlockType::WoodStairs, Orientation::South).part_count(),
        2
    );
    assert_eq!(
        block_geometry(BlockType::Fence, Orientation::South).part_count(),
        3
    );
    assert_eq!(
        block_geometry(BlockType::WoodDoor, Orientation::South).part_count(),
        1
    );
    assert_eq!(
        block_geometry(BlockType::PortalCoreDarkCrimson, Orientation::South).part_count(),
        1
    );
    assert_eq!(
        block_geometry(BlockType::AmethystCluster, Orientation::Up).part_count(),
        5
    );

    assert_ne!(
        block_geometry(BlockType::WoodStairs, Orientation::North),
        block_geometry(BlockType::WoodStairs, Orientation::East)
    );
}

// ---------------------------------------------------------------------
// 2-6. Empty space inside an occupied cell lets the ray through.
// ---------------------------------------------------------------------

#[test]
fn ray_crosses_the_empty_notch_of_a_stair_and_hits_the_cube_behind() {
    let world = world_of(&[
        (
            cell(1, 1, 2),
            wood(BlockType::WoodStairs, Orientation::South),
        ),
        (cell(2, 1, 2), stone()),
    ]);

    // y = 1.75, z = 2.75: above the lower step and in front of the upper one.
    let voxel =
        nearest_voxel_hit(&world, &ray((-1.0, 1.75, 2.75), (1.0, 0.0, 0.0)), RANGE).unwrap();

    assert_eq!(voxel.cell, cell(2, 1, 2));
    assert_eq!(voxel.hit.face, Face::NegativeX);
    assert!(approx(voxel.hit.point.x, 2.0));
}

#[test]
fn stair_geometry_is_hit_where_it_has_volume() {
    let world = world_of(&[
        (
            cell(1, 1, 2),
            wood(BlockType::WoodStairs, Orientation::South),
        ),
        (cell(2, 1, 2), stone()),
    ]);

    // Same direction, but low enough to strike the lower step's side.
    let voxel =
        nearest_voxel_hit(&world, &ray((-1.0, 1.25, 2.75), (1.0, 0.0, 0.0)), RANGE).unwrap();
    assert_eq!(voxel.cell, cell(1, 1, 2));
    assert_eq!(voxel.hit.face, Face::NegativeX);
}

#[test]
fn ray_slips_through_the_gaps_of_a_fence() {
    let world = world_of(&[
        (cell(2, 1, 2), wood(BlockType::Fence, Orientation::South)),
        (cell(2, 1, 1), stone()),
    ]);

    // y = 1.65 lies between the two rails, x = 2.15 beside the post.
    let through =
        nearest_voxel_hit(&world, &ray((2.15, 1.65, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(through.cell, cell(2, 1, 1));
    assert_eq!(through.hit.face, Face::PositiveZ);
    assert!(approx(through.hit.point.z, 2.0));

    // At rail height the same ray stops on the fence itself.
    let rail = nearest_voxel_hit(&world, &ray((2.15, 1.45, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(rail.cell, cell(2, 1, 2));

    // And the post is hit dead center.
    let post = nearest_voxel_hit(&world, &ray((2.5, 1.2, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(post.cell, cell(2, 1, 2));
}

#[test]
fn a_door_only_occupies_a_thin_slab_of_its_cell() {
    let world = world_of(&[
        (cell(3, 1, 2), wood(BlockType::WoodDoor, Orientation::South)),
        (cell(3, 1, 1), stone()),
    ]);

    // Front: stops on the leaf at z = 3.
    let front = nearest_voxel_hit(&world, &ray((3.5, 1.5, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(front.cell, cell(3, 1, 2));
    assert!(approx(front.hit.point.z, 3.0));

    // Coming from the empty north half of the cell the leaf is still hit
    // after crossing the empty part of the cell.
    let door_only = world_of(&[(cell(3, 1, 2), wood(BlockType::WoodDoor, Orientation::South))]);
    let side =
        nearest_voxel_hit(&door_only, &ray((3.5, 1.5, 1.2), (0.0, 0.0, 1.0)), RANGE).unwrap();
    assert_eq!(side.cell, cell(3, 1, 2));
    assert!(approx(side.hit.point.z, 2.8125));

    // Straight down through the empty part of the cell reaches the floor.
    let down = nearest_voxel_hit(&world, &ray((3.5, 5.0, 2.4), (0.0, -1.0, 0.0)), RANGE);
    assert!(down.is_none());
}

#[test]
fn a_portal_core_only_occupies_a_thin_plane() {
    let world = world_of(&[
        (
            cell(4, 1, 2),
            BlockInstance::new(
                BlockType::PortalCoreDarkCrimson,
                wood_material_id(),
                Orientation::South,
            ),
        ),
        (cell(4, 1, 1), stone()),
    ]);

    let hit = nearest_voxel_hit(&world, &ray((4.5, 1.5, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(hit.cell, cell(4, 1, 2));
    assert!(approx(hit.hit.point.z, 2.5625));

    // Skimming the cell in front of the membrane finds nothing.
    assert!(nearest_voxel_hit(&world, &ray((3.0, 1.5, 2.2), (1.0, 0.0, 0.0)), RANGE).is_none());
}

#[test]
fn an_amethyst_cluster_is_not_a_full_cube() {
    let world = world_of(&[
        (
            cell(5, 1, 2),
            BlockInstance::new(
                BlockType::AmethystCluster,
                wood_material_id(),
                Orientation::Up,
            ),
        ),
        (cell(5, 1, 1), stone()),
    ]);

    // Down the empty corner of the cluster cell.
    assert!(nearest_voxel_hit(&world, &ray((5.05, 5.0, 2.95), (0.0, -1.0, 0.0)), RANGE).is_none());

    // Central crystal tip, at 0.8125 of the cell height.
    let tip = nearest_voxel_hit(&world, &ray((5.5, 5.0, 2.5), (0.0, -1.0, 0.0)), RANGE).unwrap();
    assert_eq!(tip.cell, cell(5, 1, 2));
    assert!(approx(tip.hit.point.y, 1.8125));

    // Along Z through the gap beside the crystals, straight to the cube behind.
    let behind =
        nearest_voxel_hit(&world, &ray((5.95, 1.3, 6.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(behind.cell, cell(5, 1, 1));
}

// ---------------------------------------------------------------------
// 7-9. DDA continues after a local miss; the first real hit wins.
// ---------------------------------------------------------------------

#[test]
fn an_occupied_cell_without_a_local_hit_lets_the_dda_continue() {
    let fence_cell = cell(2, 1, 2);
    let world = world_of(&[
        (fence_cell, wood(BlockType::Fence, Orientation::South)),
        (cell(2, 1, 1), stone()),
        (cell(2, 1, 0), stone()),
    ]);
    let gap_ray = ray((2.15, 1.65, 6.0), (0.0, 0.0, -1.0));

    assert!(world.contains(fence_cell));
    let voxel = nearest_voxel_hit(&world, &gap_ray, RANGE).unwrap();
    assert_ne!(voxel.cell, fence_cell);
    assert_eq!(voxel.cell, cell(2, 1, 1), "the nearer cube behind wins");
}

#[test]
fn the_first_real_hit_wins_between_two_partial_cells() {
    let world = world_of(&[
        (cell(0, 1, 3), wood(BlockType::WoodDoor, Orientation::South)),
        (cell(0, 1, 2), wood(BlockType::WoodDoor, Orientation::South)),
    ]);

    let voxel = nearest_voxel_hit(&world, &ray((0.5, 1.5, 8.0), (0.0, 0.0, -1.0)), RANGE).unwrap();
    assert_eq!(voxel.cell, cell(0, 1, 3));
    assert!(approx(voxel.hit.point.z, 4.0));
}

#[test]
fn full_cubes_keep_the_previous_cell_cube_behavior() {
    let world = world_of(&[(cell(1, 1, 1), stone())]);
    let rays = [
        ray((1.5, 1.5, 6.0), (0.0, 0.0, -1.0)),
        ray((-2.0, 1.4, 1.6), (1.0, 0.05, 0.0)),
        ray((1.5, 6.0, 1.5), (0.1, -1.0, 0.05)),
    ];

    for r in rays {
        let voxel = nearest_voxel_hit(&world, &r, RANGE).unwrap();
        assert_eq!(
            Some(voxel.hit),
            cell_cube(cell(1, 1, 1)).intersect(&r, 0.0, RANGE)
        );
    }
}

// ---------------------------------------------------------------------
// 10. Face, UV, point and distance survive the local -> world transform.
// ---------------------------------------------------------------------

#[test]
fn face_uv_point_and_distance_are_world_space_and_consistent() {
    let world = diagnostic_partial_voxel_world();
    let camera = app_camera();
    let (width, height) = (80, 60);
    let mut partial_hits = 0;

    for y in 0..height {
        for x in 0..width {
            let r = primary_ray(&camera, x, y, width, height);
            let Some(voxel) = nearest_voxel_hit(&world, &r, RANGE) else {
                continue;
            };

            assert!((0.0..=1.0).contains(&voxel.hit.uv.x));
            assert!((0.0..=1.0).contains(&voxel.hit.uv.y));
            assert!(approx(voxel.hit.normal.length(), 1.0));
            assert_eq!(voxel.hit.normal, voxel.hit.face.normal());

            let along = r.at(voxel.hit.distance);
            assert!(approx(along.x, voxel.hit.point.x));
            assert!(approx(along.y, voxel.hit.point.y));
            assert!(approx(along.z, voxel.hit.point.z));

            // The hit lies inside the cell that reported it.
            let c = voxel.cell;
            assert!(
                voxel.hit.point.x >= c.x as f32 - EPS
                    && voxel.hit.point.x <= c.x as f32 + 1.0 + EPS
            );
            assert!(
                voxel.hit.point.y >= c.y as f32 - EPS
                    && voxel.hit.point.y <= c.y as f32 + 1.0 + EPS
            );
            assert!(
                voxel.hit.point.z >= c.z as f32 - EPS
                    && voxel.hit.point.z <= c.z as f32 + 1.0 + EPS
            );

            if !block_geometry(voxel.block.block_type(), voxel.block.orientation()).is_full_cube() {
                partial_hits += 1;
            }
        }
    }

    assert!(
        partial_hits > 50,
        "partial shapes must be visible: {partial_hits}"
    );
}

// ---------------------------------------------------------------------
// 11. Shadow rays respect partial geometry.
// ---------------------------------------------------------------------

#[test]
fn shadow_rays_pass_through_fence_gaps_and_stop_at_the_post() {
    let world = world_of(&[(cell(2, 1, 2), wood(BlockType::Fence, Orientation::South))]);

    // From the front toward the back through the rail gap: no occlusion.
    let gap = ray((2.15, 1.65, 6.0), (0.0, 0.0, -1.0));
    assert!(!is_voxel_occluded(&world, &gap, RANGE));

    // Through the post: occluded.
    let post = ray((2.5, 1.65, 6.0), (0.0, 0.0, -1.0));
    assert!(is_voxel_occluded(&world, &post, RANGE));

    // Through a rail: occluded.
    let rail = ray((2.15, 1.45, 6.0), (0.0, 0.0, -1.0));
    assert!(is_voxel_occluded(&world, &rail, RANGE));

    // A blocker beyond the queried distance does not count.
    assert!(!is_voxel_occluded(&world, &post, 1.0));
}

// ---------------------------------------------------------------------
// 12. The scene is built from a VoxelWorld and matches an independent oracle.
// ---------------------------------------------------------------------

#[test]
fn the_diagnostic_scene_is_a_voxel_world_with_every_shape() {
    let world = diagnostic_partial_voxel_world();
    let present = |target: BlockType| {
        (0..8).any(|x| {
            (0..3).any(|y| {
                (0..5).any(|z| {
                    world
                        .get(cell(x, y, z))
                        .is_some_and(|b| b.block_type() == target)
                })
            })
        })
    };

    assert!(present(BlockType::Stone));
    assert!(present(BlockType::WoodStairs));
    assert!(present(BlockType::Fence));
    assert!(present(BlockType::WoodDoor));
    assert!(present(BlockType::PortalCoreDarkCrimson));
    assert!(present(BlockType::AmethystCluster));
    assert!(present(BlockType::Grass));
}

#[test]
fn dda_matches_the_brute_force_oracle_on_the_mixed_scene() {
    let world = diagnostic_partial_voxel_world();
    let camera = app_camera();
    let (width, height) = (96, 72);
    let mut hits = 0;

    for y in 0..height {
        for x in 0..width {
            let r = primary_ray(&camera, x, y, width, height);
            match (
                nearest_voxel_hit(&world, &r, RANGE),
                brute_force_nearest(&world, &r),
            ) {
                (None, None) => {}
                (Some(a), Some(b)) => {
                    hits += 1;
                    assert_eq!(a.cell, b.cell, "pixel ({x},{y})");
                    assert_eq!(a.hit.face, b.hit.face, "pixel ({x},{y})");
                    assert!(approx(a.hit.distance, b.hit.distance), "pixel ({x},{y})");
                }
                (a, b) => panic!("pixel ({x},{y}): dda={a:?} oracle={b:?}"),
            }
        }
    }

    assert!(hits > 500);
    assert!(hits < width * height);
}

// ---------------------------------------------------------------------
// 13. The lit render still works: lighting, hard shadows, no corruption.
// ---------------------------------------------------------------------

fn mixed_scene_lights() -> [Light; 2] {
    [
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            Color::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(-1.5, 6.5, 6.5),
            Color::white(),
            1.5,
        )),
    ]
}

#[test]
fn the_mixed_scene_renders_lit_without_missing_materials() {
    let world = diagnostic_partial_voxel_world();
    let (textures, face_textures) = grass_face_textures();
    let materials: HashMap<MaterialId, Material> = diagnostic_partial_materials(face_textures);
    let lights = mixed_scene_lights();
    let camera = app_camera();
    let background = Color::new(0.05, 0.05, 0.08, 1.0);
    let (width, height) = (96, 72);
    let mut framebuffer = Framebuffer::new(width, height);

    let mut background_pixels = 0;
    let mut lit_pixels = 0;
    for y in 0..height {
        for x in 0..width {
            let r = primary_ray(&camera, x, y, width, height);
            let color = cast_ray_voxel_lit(
                &world,
                &materials,
                &r,
                camera.position,
                &lights,
                0.15,
                background,
                &textures,
                RANGE,
            );
            assert_ne!(color, MISSING_MATERIAL_COLOR, "pixel ({x},{y})");
            assert!(color.r.is_finite() && color.g.is_finite() && color.b.is_finite());
            assert!((0.0..=1.0).contains(&color.r));
            if color == background {
                background_pixels += 1;
            } else {
                lit_pixels += 1;
            }
            framebuffer.set_pixel(x, y, color);
        }
    }

    assert!(background_pixels > 0);
    assert!(lit_pixels > width * height / 4);
}

#[test]
fn a_fence_post_casts_a_hard_shadow_on_the_ground() {
    let (textures, face_textures) = grass_face_textures();
    let materials = diagnostic_partial_materials(face_textures);
    let mut world = VoxelWorld::new();
    for x in 0..5 {
        for z in 0..5 {
            world.insert(cell(x, 0, z), stone());
        }
    }
    world.insert(cell(2, 1, 2), wood(BlockType::Fence, Orientation::South));

    let light_position = Vec3::new(5.0, 3.0, 2.5);
    let lights = [Light::Point(PointLight::new(
        light_position,
        Color::white(),
        3.0,
    ))];
    let background = Color::new(0.0, 0.0, 0.0, 1.0);
    let ground_color = |x: f32, z: f32| {
        cast_ray_voxel_lit(
            &world,
            &materials,
            &ray((x, 4.0, z), (0.0, -1.0, 0.0)),
            Vec3::new(x, 4.0, z),
            &lights,
            0.15,
            background,
            &textures,
            RANGE,
        )
    };
    let luminance = |c: Color| c.r + c.g + c.b;

    // Ground point west of the post, on the light's line through the post
    // center (z = 2.5): the post blocks the light.
    let shadowed_point = Vec3::new(1.3, 1.0001, 2.5);
    let to_light = light_position - shadowed_point;
    let blocked = ray((1.3, 1.0001, 2.5), (to_light.x, to_light.y, to_light.z));
    assert!(is_voxel_occluded(&world, &blocked, to_light.length()));

    // Same ground column shifted in z: the shadow ray misses every part.
    let lit_point = Vec3::new(1.3, 1.0001, 3.4);
    let to_light = light_position - lit_point;
    let clear = ray((1.3, 1.0001, 3.4), (to_light.x, to_light.y, to_light.z));
    assert!(!is_voxel_occluded(&world, &clear, to_light.length()));

    // The shaded image agrees: the shadowed ground is darker than the lit one.
    assert!(luminance(ground_color(1.3, 2.5)) < luminance(ground_color(1.3, 3.4)));
}

// ---------------------------------------------------------------------
// 14. Guard: block shapes are resolved through BlockGeometry, not FullCube.
// ---------------------------------------------------------------------

#[test]
fn traversal_never_falls_back_to_a_full_cube_for_partial_blocks() {
    let source = std::fs::read_to_string(format!(
        "{}/src/renderer/raytracer.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let start = source.find("pub fn nearest_voxel_hit").unwrap();
    let end = source[start..].find("\n}\n").unwrap() + start;
    let body = &source[start..end];

    assert!(body.contains("block_geometry("));
    assert!(body.contains(".translated("));
    assert!(!body.contains("cell_cube("));
    assert!(matches!(
        block_geometry(BlockType::Fence, Orientation::South),
        BlockGeometry::Composite(_)
    ));
}
