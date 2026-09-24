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
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
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
use renderer::shadows::shadow_ray_to_point_light;
use renderer::texture_sampling::sample_nearest;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light, PointLight};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::scene::{
    diagnostic_materials, diagnostic_voxel_world, grass_material_id, stone_material_id,
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

fn make_ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

/// The same camera parameters `app.rs` uses for the diagnostic scene.
fn app_camera() -> Camera {
    Camera::new(
        Vec3::new(6.0, 5.0, 7.5),
        Vec3::new(1.8, 1.0, 1.8),
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

/// Independent oracle: intersect the ray with *every* occupied cell and keep
/// the nearest. Used only by tests to validate the DDA result; the renderer
/// never does this.
fn brute_force_nearest(world_cells: &[IVec3], world: &VoxelWorld, ray: &Ray) -> Option<VoxelHit> {
    let mut best: Option<VoxelHit> = None;
    for position in world_cells {
        let Some(block) = world.get(*position) else {
            continue;
        };
        if let Some(hit) = cell_cube(*position).intersect(ray, 0.0, RANGE) {
            let closer = best
                .as_ref()
                .is_none_or(|current| hit.distance < current.hit.distance);
            if closer {
                best = Some(VoxelHit {
                    cell: *position,
                    block: *block,
                    hit,
                });
            }
        }
    }
    best
}

fn all_terrain_cells() -> Vec<IVec3> {
    let mut cells = Vec::new();
    for x in -2..8 {
        for y in -2..6 {
            for z in -2..8 {
                cells.push(cell(x, y, z));
            }
        }
    }
    cells
}

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

// ---------------------------------------------------------------------
// 1 / 14. The diagnostic scene is a real multi-voxel VoxelWorld.
// ---------------------------------------------------------------------

#[test]
fn diagnostic_scene_contains_multiple_voxels() {
    let world = diagnostic_voxel_world();
    assert!((8..=30).contains(&world.len()), "len = {}", world.len());
    assert_eq!(world.len(), 22);
}

#[test]
fn diagnostic_scene_lives_in_the_sparse_voxel_world_with_height_variation_and_air() {
    let world = diagnostic_voxel_world();

    // Two grass tops at different heights and a stone layer beneath.
    assert_eq!(
        world.get(cell(0, 2, 0)).unwrap().block_type(),
        BlockType::Grass
    );
    assert_eq!(
        world.get(cell(3, 0, 3)).unwrap().block_type(),
        BlockType::Grass
    );
    assert_eq!(
        world.get(cell(0, 1, 1)).unwrap().block_type(),
        BlockType::Stone
    );
    // Air above the lower columns and outside the footprint.
    assert!(!world.contains(cell(3, 1, 3)));
    assert!(!world.contains(cell(5, 0, 5)));
    assert!(!world.contains(cell(-1, 0, 0)));
}

// ---------------------------------------------------------------------
// 2 / 3 / 4. Primary rays.
// ---------------------------------------------------------------------

#[test]
fn a_ray_from_above_hits_the_expected_top_voxel() {
    let world = diagnostic_voxel_world();
    let ray = make_ray((0.5, 12.0, 0.5), (0.0, -1.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, RANGE).expect("must hit");

    assert_eq!(voxel.cell, cell(0, 2, 0));
    assert_eq!(voxel.block.block_type(), BlockType::Grass);
    assert_eq!(voxel.hit.face, Face::PositiveY);
    assert!(approx(voxel.hit.distance, 9.0));
}

#[test]
fn a_background_ray_produces_no_hit() {
    let world = diagnostic_voxel_world();
    let upward = make_ray((2.0, 10.0, 2.0), (0.0, 1.0, 0.0));
    let sideways = make_ray((20.0, 1.5, 20.0), (1.0, 0.0, 0.0));

    assert!(nearest_voxel_hit(&world, &upward, RANGE).is_none());
    assert!(nearest_voxel_hit(&world, &sideways, RANGE).is_none());
}

#[test]
fn a_near_voxel_hides_a_far_voxel_in_the_same_row() {
    let world = diagnostic_voxel_world();
    // Row z = 0, y = 0 is occupied for x = 0..3; from +X the nearest is x = 3.
    let ray = make_ray((10.5, 0.5, 0.5), (-1.0, 0.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, RANGE).expect("must hit");

    assert_eq!(voxel.cell, cell(3, 0, 0));
    assert_eq!(voxel.hit.face, Face::PositiveX);
    assert!(approx(voxel.hit.distance, 6.5));
}

#[test]
fn dda_matches_a_brute_force_oracle_for_every_pixel_of_the_camera_frame() {
    let world = diagnostic_voxel_world();
    let camera = app_camera();
    let cells = all_terrain_cells();
    let (width, height) = (64, 48);

    let mut hits = 0;
    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            let dda = nearest_voxel_hit(&world, &ray, RANGE);
            let oracle = brute_force_nearest(&cells, &world, &ray);

            match (dda, oracle) {
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

    // The frame must really contain both hits and background.
    assert!(hits > 100);
    assert!(hits < width * height);
}

// ---------------------------------------------------------------------
// 5. UV stays in range for every visible hit.
// ---------------------------------------------------------------------

#[test]
fn every_visible_hit_has_uv_in_range_a_unit_normal_and_a_coherent_point() {
    let world = diagnostic_voxel_world();
    let camera = app_camera();
    let (width, height) = (64, 48);

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            if let Some(voxel) = nearest_voxel_hit(&world, &ray, RANGE) {
                assert!((0.0..=1.0).contains(&voxel.hit.uv.x));
                assert!((0.0..=1.0).contains(&voxel.hit.uv.y));
                assert!(approx(voxel.hit.normal.length(), 1.0));
                let along = ray.at(voxel.hit.distance);
                assert!(approx(along.x, voxel.hit.point.x));
                assert!(approx(along.y, voxel.hit.point.y));
                assert!(approx(along.z, voxel.hit.point.z));
            }
        }
    }
}

// ---------------------------------------------------------------------
// 6. MaterialId resolves the expected material on the diagnostic path.
// ---------------------------------------------------------------------

#[test]
fn material_ids_resolve_to_the_expected_diagnostic_materials() {
    let (_manager, face_textures) = grass_face_textures();
    let materials = diagnostic_materials(face_textures);
    let world = diagnostic_voxel_world();

    let grass_block = world.get(cell(0, 2, 0)).unwrap();
    let stone_block = world.get(cell(0, 1, 1)).unwrap();
    assert_eq!(grass_block.material_id(), grass_material_id());
    assert_eq!(stone_block.material_id(), stone_material_id());

    let grass = materials.get(grass_block.material_id()).unwrap();
    assert!(grass.face_textures.is_some());
    assert!(approx(grass.specular, 0.04));
    assert!(approx(grass.shininess, 8.0));

    let stone = materials.get(stone_block.material_id()).unwrap();
    assert!(stone.face_textures.is_none());
}

#[test]
fn every_voxel_in_the_scene_has_a_material_in_the_map() {
    let (_manager, face_textures) = grass_face_textures();
    let materials = diagnostic_materials(face_textures);
    let world = diagnostic_voxel_world();

    for position in all_terrain_cells() {
        if let Some(block) = world.get(position) {
            assert!(
                materials.contains(block.material_id()),
                "missing material at {position:?}"
            );
        }
    }
}

#[test]
fn a_voxel_with_an_unknown_material_renders_the_loud_missing_color() {
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        BlockInstance::new(BlockType::Stone, MaterialId::new(999), Orientation::Up),
    );
    let materials: MaterialLibrary = MaterialLibrary::new();
    let manager = TextureManager::new();
    let ray = make_ray((0.5, 5.0, 0.5), (0.0, -1.0, 0.0));

    let color = cast_ray_voxel_lit(
        &world,
        &materials,
        &ray,
        Vec3::new(0.5, 5.0, 0.5),
        &[],
        0.1,
        Color::black(),
        &manager,
        RANGE,
    );

    assert_eq!(color, MISSING_MATERIAL_COLOR);
}

// ---------------------------------------------------------------------
// 7. The texture is sampled from the real hit.
// ---------------------------------------------------------------------

#[test]
fn grass_top_color_comes_from_sampling_the_top_texture_at_the_real_hit_uv() {
    let (manager, face_textures) = grass_face_textures();
    let materials = diagnostic_materials(face_textures);
    let world = diagnostic_voxel_world();
    let ray = make_ray((0.3, 12.0, 0.8), (0.0, -1.0, 0.0));

    let voxel = nearest_voxel_hit(&world, &ray, RANGE).expect("must hit");
    assert_eq!(voxel.hit.face, Face::PositiveY);

    let no_lights: [Light; 0] = [];
    let color = cast_ray_voxel_lit(
        &world,
        &materials,
        &ray,
        ray.origin,
        &no_lights,
        1.0,
        Color::black(),
        &manager,
        RANGE,
    );

    let top_id = face_textures.texture_for_face(Face::PositiveY);
    let expected = sample_nearest(manager.get(top_id).unwrap(), voxel.hit.uv);
    assert!(approx(color.r, expected.r));
    assert!(approx(color.g, expected.g));
    assert!(approx(color.b, expected.b));
}

#[test]
fn a_lateral_grass_face_samples_the_side_texture_and_stone_stays_uniform() {
    let (manager, face_textures) = grass_face_textures();
    let materials = diagnostic_materials(face_textures);
    let world = diagnostic_voxel_world();
    let no_lights: [Light; 0] = [];

    // +X face of the low front platform: a grass cell's side texture.
    let side_ray = make_ray((10.0, 0.5, 3.5), (-1.0, 0.0, 0.0));
    let side_hit = nearest_voxel_hit(&world, &side_ray, RANGE).unwrap();
    assert_eq!(side_hit.hit.face, Face::PositiveX);
    let side_color = cast_ray_voxel_lit(
        &world,
        &materials,
        &side_ray,
        side_ray.origin,
        &no_lights,
        1.0,
        Color::black(),
        &manager,
        RANGE,
    );
    let side_id = face_textures.texture_for_face(Face::PositiveX);
    let expected_side = sample_nearest(manager.get(side_id).unwrap(), side_hit.hit.uv);
    assert!(approx(side_color.r, expected_side.r));
    assert!(approx(side_color.g, expected_side.g));

    // The exposed stone cliff (+Z face of the cell below the wall's top).
    let stone_ray = make_ray((0.5, 1.5, 10.0), (0.0, 0.0, -1.0));
    let stone_hit = nearest_voxel_hit(&world, &stone_ray, RANGE).unwrap();
    assert_eq!(stone_hit.block.block_type(), BlockType::Stone);
    let stone_color = cast_ray_voxel_lit(
        &world,
        &materials,
        &stone_ray,
        stone_ray.origin,
        &no_lights,
        1.0,
        Color::black(),
        &manager,
        RANGE,
    );
    assert!(approx(stone_color.r, 0.5));
    assert!(approx(stone_color.g, 0.5));
    assert!(approx(stone_color.b, 0.52));
}

// ---------------------------------------------------------------------
// 8 / 9 / 10. Shadows query the same VoxelWorld.
// ---------------------------------------------------------------------

#[test]
fn a_shadow_ray_detects_a_blocking_voxel() {
    let world = diagnostic_voxel_world();
    // From above the low platform toward -X, the raised column at x = 1 blocks it.
    let ray = make_ray((3.5, 1.6, 0.5), (-1.0, 0.0, 0.0));

    assert!(is_voxel_occluded(&world, &ray, 10.0));
    // The same ray is unobstructed if the light is closer than the blocker.
    assert!(!is_voxel_occluded(&world, &ray, 0.4));
}

fn floor_and_blocker_world(blocker_y: i32) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    let stone = BlockInstance::new(BlockType::Stone, MaterialId::new(9), Orientation::Up);
    world.insert(cell(0, 0, 0), stone);
    world.insert(cell(0, blocker_y, 0), stone);
    world
}

#[test]
fn a_voxel_behind_a_point_light_does_not_block_it() {
    let light = PointLight::new(Vec3::new(0.5, 4.0, 0.5), Color::white(), 1.0);
    let surface_point = Vec3::new(0.5, 1.0, 0.5);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let (ray, distance) = shadow_ray_to_point_light(surface_point, normal, &light);

    let between = floor_and_blocker_world(2); // y in [2,3], below the light
    let behind = floor_and_blocker_world(5); // y in [5,6], beyond the light

    assert!(is_voxel_occluded(&between, &ray, distance));
    assert!(!is_voxel_occluded(&behind, &ray, distance));
}

fn matte_materials() -> MaterialLibrary {
    let mut materials = MaterialLibrary::new();
    materials.insert(
        MaterialId::new(9),
        Material::matte(Color::new(0.6, 0.6, 0.6, 1.0)),
    );
    materials
}

#[test]
fn ambient_persists_under_a_voxel_shadow_and_direct_light_returns_outside_it() {
    // A stone strip along x with a floating blocker above x = 1.
    let mut world = VoxelWorld::new();
    let stone = BlockInstance::new(BlockType::Stone, MaterialId::new(9), Orientation::Up);
    for x in 0..4 {
        world.insert(cell(x, 0, 0), stone);
    }
    world.insert(cell(1, 2, 0), stone);

    let materials = matte_materials();
    let manager = TextureManager::new();
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::white(),
        1.0,
    ))];
    let ambient_factor = 0.2;

    let render = |ray: &Ray| {
        cast_ray_voxel_lit(
            &world,
            &materials,
            ray,
            ray.origin,
            &lights,
            ambient_factor,
            Color::black(),
            &manager,
            50.0,
        )
    };

    // Aim at the floor top under the blocker (x = 1.5) and at an open spot.
    let shadowed = render(&make_ray((1.5, 1.5, 6.0), (0.0, -0.5, -5.5)));
    let lit = render(&make_ray((3.5, 1.5, 6.0), (0.0, -0.5, -5.5)));

    let ambient_only = 0.6 * ambient_factor;
    assert!(approx(shadowed.r, ambient_only), "shadowed = {shadowed:?}");
    assert!(approx(shadowed.g, ambient_only));
    assert!(approx(shadowed.b, ambient_only));
    assert!(lit.r > ambient_only + 0.3, "lit = {lit:?}");
}

// ---------------------------------------------------------------------
// 11 / 13. Architecture guards on the visible path.
// ---------------------------------------------------------------------

#[test]
fn no_raylib_3d_api_is_used_by_the_render_sources() {
    let sources = [
        include_str!("../src/app.rs"),
        include_str!("../src/renderer/raytracer.rs"),
        include_str!("../src/renderer/voxel_traversal.rs"),
        include_str!("../src/scene/scene.rs"),
        include_str!("../src/scene/voxel_world.rs"),
    ];
    for source in sources {
        let code = code_of(source);
        for forbidden in [
            "Camera3D",
            "BeginMode3D",
            "DrawCube",
            "DrawCubeV",
            "DrawModel",
            "Mesh",
            "Shader",
        ] {
            assert!(!has_token(&code, forbidden), "found {forbidden}");
        }
    }
}

#[test]
fn the_app_renders_through_the_voxel_world_instead_of_a_manual_cube_list() {
    let app = code_of(include_str!("../src/app.rs"));

    assert!(has_token(&app, "cast_ray_voxel_lit"));
    // Gate 06: the visible scene is the mixed partial-geometry VoxelWorld.
    assert!(has_token(&app, "diagnostic_partial_voxel_world"));
    assert!(has_token(&app, "VoxelWorld"));
    // No hand-built cube list and no legacy explicit-object raycast.
    assert!(!has_token(&app, "Cube"));
    assert!(!has_token(&app, "cast_ray_lit"));
    assert!(!has_token(&app, "objects"));
}

// ---------------------------------------------------------------------
// 12. Framebuffer contract with the full voxel pipeline.
// ---------------------------------------------------------------------

#[test]
fn a_full_voxel_frame_keeps_framebuffer_dimensions_and_produces_finite_colors() {
    let (manager, face_textures) = grass_face_textures();
    let materials = diagnostic_materials(face_textures);
    let world = diagnostic_voxel_world();
    let camera = app_camera();
    let lights = [
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            Color::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(-3.0, 7.0, 6.0),
            Color::white(),
            1.5,
        )),
    ];
    let background = Color::new(0.05, 0.05, 0.08, 1.0);

    let (width, height) = (80, 60);
    let mut framebuffer = Framebuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            let color = cast_ray_voxel_lit(
                &world,
                &materials,
                &ray,
                camera.position,
                &lights,
                0.1,
                background,
                &manager,
                RANGE,
            );
            framebuffer.set_pixel(x, y, color);
        }
    }

    assert_eq!(framebuffer.width(), width);
    assert_eq!(framebuffer.height(), height);
    assert_eq!(framebuffer.pixels().len(), width * height);

    let mut background_pixels = 0;
    let mut lit_pixels = 0;
    for pixel in framebuffer.pixels() {
        assert!(pixel.r.is_finite() && pixel.g.is_finite() && pixel.b.is_finite());
        assert!((0.0..=1.0).contains(&pixel.r));
        assert!(*pixel != MISSING_MATERIAL_COLOR);
        if (pixel.r - background.r).abs() < 1e-6 && (pixel.b - background.b).abs() < 1e-6 {
            background_pixels += 1;
        } else {
            lit_pixels += 1;
        }
    }
    assert!(background_pixels > 0, "no background pixels");
    assert!(lit_pixels > 200, "too few voxel pixels: {lit_pixels}");
}

// ---------------------------------------------------------------------
// Face orientation of the visible faces is coherent with the frozen UV.
// ---------------------------------------------------------------------

#[test]
fn visible_top_faces_are_positive_y_and_side_faces_are_positive_x_or_z() {
    let world = diagnostic_voxel_world();
    let camera = app_camera();
    let (width, height) = (64, 48);
    let mut seen = std::collections::HashSet::new();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(&camera, x, y, width, height);
            if let Some(voxel) = nearest_voxel_hit(&world, &ray, RANGE) {
                seen.insert(format!("{:?}", voxel.hit.face));
            }
        }
    }

    // The camera sits at +X/+Y/+Z of the terrain: only those faces show.
    assert!(seen.contains("PositiveY"));
    assert!(seen.contains("PositiveX"));
    assert!(seen.contains("PositiveZ"));
    assert!(!seen.contains("NegativeY"));
}
