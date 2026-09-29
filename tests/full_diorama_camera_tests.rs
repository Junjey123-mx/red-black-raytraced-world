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
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
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
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
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
use scene::block_type::{BlockFamily, BlockType};
use scene::material_gallery::gallery_camera;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_CAMERA_OFFSET, WORLD_MAX_DISTANCE, WorldScene, WorldTextures,
    world_background, world_camera, world_focus, world_lights, world_materials,
};
use std::collections::HashSet;

const ASPECT: f32 = 4.0 / 3.0;

#[test]
fn the_camera_is_finite_and_far_enough_for_the_whole_diamond() {
    let camera = world_camera(ASPECT);
    for v in [camera.position, camera.target] {
        assert!(v.x.is_finite() && v.y.is_finite() && v.z.is_finite());
    }
    let distance = (camera.position - camera.target).length();
    assert!(distance > MIN_DISTANCE && distance < MAX_DISTANCE);
    // The diamond spans roughly 35 blocks top to tip: at this distance the
    // 60-degree vertical field holds it with margin.
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let height = (scene
        .house_exterior()
        .roof
        .iter()
        .map(|(c, _, _)| c.y)
        .max()
        .unwrap()
        - r.lower_tip_y) as f32;
    let visible = 2.0 * distance * (30.0f32).to_radians().tan();
    assert!(
        visible > height * 1.15,
        "visible {visible} vs height {height}"
    );
    assert_eq!(camera.position, world_focus() + WORLD_CAMERA_OFFSET);
}

#[test]
fn the_target_lies_inside_the_full_bounds() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let target = world_camera(ASPECT).target;
    assert!(target.y > r.lower_tip_y as f32 && target.y < r.upper_surface_reference as f32);
    assert!((target.x - r.center_x as f32).abs() <= 2.0);
    assert!((target.z - r.center_z as f32).abs() <= 2.0);
    // Halfway between the surface and the tip: not the upper scene alone.
    let mid = (r.upper_surface_reference + r.lower_tip_y) as f32 / 2.0;
    assert!(
        (target.y - mid).abs() <= 2.0,
        "target y {} vs mid {mid}",
        target.y
    );
}

#[test]
fn the_full_bounds_are_visible_in_the_initial_frame() {
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
    let mut families: HashSet<BlockFamily> = HashSet::new();
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut sky = 0;
    let mut edge_hits = 0;
    for y in 0..h {
        for x in 0..w {
            let ray = primary_ray(&camera, x, y, w, h);
            match nearest_visible_hit(&view, &ray, WORLD_MAX_DISTANCE) {
                Some(hit) => {
                    seen.insert(hit.block.block_type());
                    families.insert(hit.block.block_type().family());
                    min_y = min_y.min(hit.cell.y);
                    max_y = max_y.max(hit.cell.y);
                    if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                        edge_hits += 1;
                    }
                }
                None => sky += 1,
            }
        }
    }
    // Nothing is clipped by the frame edges; the sky surrounds the diamond.
    assert_eq!(edge_hits, 0, "the diamond touches the frame edge");
    assert!(sky > (w * h) / 3);
    // From the tree crowns to the lower tip.
    let r = scene.rhombus();
    assert!(min_y <= r.lower_tip_y + 1, "lowest visible row {min_y}");
    assert!(max_y >= 10, "highest visible row {max_y}");
    for wanted in [
        BlockType::Grass,
        BlockType::WoodPlanks,
        BlockType::Leaves,
        BlockType::Water,
        BlockType::PortalFrameRedObsidian,
        BlockType::PortalCoreDarkCrimson,
        BlockType::WoodStairs,
        BlockType::Deepslate,
        BlockType::Stone,
        BlockType::Dirt,
    ] {
        assert!(
            seen.contains(&wanted),
            "{wanted:?} is not visible from the official framing"
        );
    }
    // The cutaway shows the interior and the underside shows Red-Black.
    assert!(families.contains(&BlockFamily::Portal));
    assert!(
        families.iter().any(|f| matches!(
            f,
            BlockFamily::Crimson
                | BlockFamily::Orange
                | BlockFamily::Violet
                | BlockFamily::RedBlackBase
                | BlockFamily::StructuralRedBlack
        )),
        "{families:?}"
    );
}

#[test]
fn reset_orbit_elevation_and_zoom_work_from_the_official_view() {
    let home = world_camera(ASPECT);
    let mut view = DiagnosticCameraState::from_pose(home.position, home.target);
    let start = view;
    view.set_angles(view.yaw + 1.0, view.pitch);
    assert_ne!(view.position(), start.position());
    view.set_angles(view.yaw, view.pitch + 0.4);
    assert!(view.pitch > start.pitch);
    view.set_distance(view.distance * 0.6);
    assert!(view.distance < start.distance);
    view.reset();
    assert_eq!(view, start);
    let reset_camera = view.build_camera(ASPECT);
    assert!((reset_camera.position - home.position).length() < 1e-3);
    assert!((reset_camera.target - home.target).length() < 1e-3);
}

#[test]
fn the_catalog_camera_is_untouched() {
    let camera = gallery_camera(ASPECT);
    assert_eq!(camera.position, Vec3::new(6.0, 5.6, 14.0));
    assert_eq!(camera.target, Vec3::new(6.0, 0.6, 1.8));
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("gallery_camera(aspect_ratio)"));
    assert!(app.contains("world_camera(aspect_ratio)"));
}
