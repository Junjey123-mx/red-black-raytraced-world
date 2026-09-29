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
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
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
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_gallery.rs"]
    pub mod material_gallery;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::diagnostic::{DiagnosticCameraState, MAX_DISTANCE, MIN_DISTANCE};
use camera::projection::primary_ray;
use core::math::Vec3;
use renderer::raytracer::{VoxelScene, nearest_visible_hit};
use scene::block_type::BlockType;
use scene::material_gallery::gallery_camera;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_CAMERA_OFFSET, WORLD_MAX_DISTANCE, WorldScene, WorldTextures,
    world_background, world_camera, world_focus, world_lights, world_materials,
};
use std::collections::HashSet;

const ASPECT: f32 = 4.0 / 3.0;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[test]
fn the_official_camera_is_finite_with_a_valid_distance_and_fov() {
    let camera = world_camera(ASPECT);
    assert!(finite(camera.position) && finite(camera.target));
    let distance = (camera.position - camera.target).length();
    assert!(
        distance > MIN_DISTANCE && distance < MAX_DISTANCE,
        "{distance}"
    );
    assert!((20.0..40.0).contains(&distance), "{distance}");
    assert!(camera.fov > 0.0 && camera.fov < 180.0);
    assert_eq!(camera.fov, 60.0);
    assert_eq!(camera.aspect_ratio, ASPECT);
    assert_eq!(camera.position, world_focus() + WORLD_CAMERA_OFFSET);
}

#[test]
fn the_target_is_the_volumetric_center_of_the_scene() {
    let scene = WorldScene::new();
    let target = world_camera(ASPECT).target;
    let b = scene.bounds();
    assert!(target.x >= b.min.x as f32 && target.x <= b.max.x as f32 + 1.0);
    assert!(target.z >= b.min.z as f32 && target.z <= b.max.z as f32 + 1.0);
    // Between the deepest cell and the roof ridge / tree tops.
    let roof_top = scene
        .house_exterior()
        .roof
        .iter()
        .map(|(c, _, _)| c.y)
        .max()
        .unwrap();
    assert!(target.y >= b.min.y as f32 && target.y <= roof_top as f32);
    // Near the middle of the footprint.
    let center = scene.center();
    assert!((target.x - center.x).abs() <= 2.0 && (target.z - center.z).abs() <= 2.0);
}

#[test]
fn the_framing_is_a_raised_three_quarter_view() {
    let camera = world_camera(ASPECT);
    let offset = camera.position - camera.target;
    let pitch = (offset.y / offset.length()).asin().to_degrees();
    assert!((20.0..=45.0).contains(&pitch), "pitch {pitch}");
    // From the door side (south, +Z), a little to the east: neither a plan
    // view nor a pure frontal elevation.
    assert!(offset.z > 0.0 && offset.x > 0.0);
    assert!(offset.z > offset.x);
}

#[test]
fn reset_orbit_and_zoom_keep_working_around_the_official_framing() {
    let home = world_camera(ASPECT);
    let mut view = DiagnosticCameraState::from_pose(home.position, home.target);
    let start = view;
    let start_camera = view.build_camera(ASPECT);
    assert!((start_camera.position - home.position).length() < 1e-3);
    assert!((start_camera.target - home.target).length() < 1e-3);

    view.set_angles(view.yaw + 0.7, view.pitch + 0.3);
    assert_ne!(view.position(), start.position());
    view.set_distance(view.distance * 0.5);
    assert!((view.distance - start.distance * 0.5).abs() < 1e-4);
    view.set_target(view.target + Vec3::new(1.0, 0.0, 0.0));
    view.reset();
    assert_eq!(view, start);
    let reset_camera = view.build_camera(ASPECT);
    assert!((reset_camera.position - home.position).length() < 1e-3);
}

#[test]
fn the_catalog_camera_is_unchanged() {
    let camera = gallery_camera(ASPECT);
    assert_eq!(camera.position, Vec3::new(6.0, 5.6, 14.0));
    assert_eq!(camera.target, Vec3::new(6.0, 0.6, 1.8));
    assert_eq!(camera.fov, 60.0);
    // The world does not add block selection: only the catalog has it.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let world_mode = &app[app.find("impl ViewerMode for WorldMode").unwrap()
        ..app.find("/// Casts one primary ray").unwrap()];
    assert!(!world_mode.contains("select_next") && !world_mode.contains("select_previous"));
    assert!(!world_mode.contains("focus_camera"));
}

#[test]
fn the_initial_framing_shows_every_landmark_and_the_sky() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let scene = WorldScene::new();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let camera = world_camera(ASPECT);
    let view = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR,
        background: world_background(),
        texture_manager: &manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    let (w, h) = (160, 120);
    let mut seen: HashSet<BlockType> = HashSet::new();
    let mut sky = 0;
    let mut sky_top_row = 0;
    for y in 0..h {
        for x in 0..w {
            let ray = primary_ray(&camera, x, y, w, h);
            match nearest_visible_hit(&view, &ray, WORLD_MAX_DISTANCE) {
                Some(hit) => {
                    seen.insert(hit.block.block_type());
                }
                None => {
                    sky += 1;
                    if y == 0 {
                        sky_top_row += 1;
                    }
                }
            }
        }
    }
    for wanted in [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Deepslate,
        BlockType::Water,
        BlockType::Sand,
        BlockType::Log,
        BlockType::Leaves,
        BlockType::WoodPlanks,
        BlockType::WoodStairs,
        BlockType::Glass,
        BlockType::WoodDoor,
        BlockType::RedstoneLampLit,
        BlockType::Fence,
        BlockType::Cobblestone,
    ] {
        assert!(
            seen.contains(&wanted),
            "{wanted:?} is not visible from the official framing"
        );
    }
    // Sky above, but the mass fills a good part of the frame.
    assert_eq!(sky_top_row, w, "the top row is not all sky");
    let total = (w * h) as f32;
    assert!(
        (sky as f32) > 0.3 * total && (sky as f32) < 0.8 * total,
        "sky {sky}"
    );
}
