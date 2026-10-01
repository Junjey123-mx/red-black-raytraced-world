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
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
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
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_collision::{CameraCollisionConfig, is_camera_solid, is_position_clear};
use camera::world_free_fly::WorldFreeFlyCameraState;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::preview::AdaptivePreview;
use renderer::raytracer::{RenderQuality, VoxelScene, nearest_visible_hit};
use scene::block_type::BlockType;
use scene::material_gallery::glass_material_id;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_lights,
    world_materials,
};
use std::time::Instant;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Castle exterior from the south-west, above the approach.
pub fn pose_l_exterior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(18.0, 13.0, 24.0), v(30.5, 7.0, 8.5))
}

/// Inside the great hall, looking east across the room.
pub fn pose_l2_interior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(31.5, 6.0, 8.0), v(34.5, 6.0, 9.5))
}

#[test]
fn windows_exist_and_are_glass() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let w = scene.castle_windows();
    assert!(!w.glass.is_empty());
    assert!(
        w.glass.len() >= 12 && w.glass.len() <= c.windows.len(),
        "{}",
        w.glass.len()
    );
    for cell in &w.glass {
        assert!(c.windows.contains(cell));
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Glass);
        assert_eq!(b.material_id(), glass_material_id());
        assert_eq!(b.orientation(), Orientation::Up);
    }
    let on_keep = w.glass.iter().filter(|g| c.is_keep_shell(g.x, g.z)).count();
    let on_tower = w
        .glass
        .iter()
        .filter(|g| c.is_tower_shell(g.x, g.z))
        .count();
    assert!(
        on_keep >= 8 && on_tower >= 4,
        "keep {on_keep} tower {on_tower}"
    );
    println!(
        "GATE16 windows: glass={} keep={on_keep} towers={on_tower} castle_voxels={} voxels={}",
        w.glass.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn walls_stay_supported_around_every_window() {
    let scene = WorldScene::new();
    let w = scene.castle_windows();
    let masonry = |cell: IVec3| {
        scene.world().get(cell).is_some_and(|b| {
            matches!(
                b.block_type(),
                BlockType::Stone | BlockType::Cobblestone | BlockType::DeepslateBricks
            )
        })
    };
    for g in &w.glass {
        assert!(masonry(IVec3::new(g.x, g.y - 1, g.z)), "{g:?} sill");
        assert!(
            scene.world().contains(IVec3::new(g.x, g.y + 1, g.z)),
            "{g:?} lintel"
        );
        // Narrow: no glass neighbour along the wall in the same course.
        let side = [
            IVec3::new(g.x + 1, g.y, g.z),
            IVec3::new(g.x - 1, g.y, g.z),
            IVec3::new(g.x, g.y, g.z + 1),
            IVec3::new(g.x, g.y, g.z - 1),
        ];
        assert!(
            side.iter().all(|s| !w.glass.contains(s)),
            "{g:?} is part of a glass façade"
        );
    }
    // Predominantly stone: glass is a small share of the castle.
    let castle = scene.castle_voxels();
    assert!(
        w.glass.len() * 10 < castle,
        "{} glass of {castle}",
        w.glass.len()
    );
}

#[test]
fn glass_is_solid_to_the_camera() {
    let scene = WorldScene::new();
    for g in &scene.castle_windows().glass {
        let b = scene.world().get(*g).unwrap();
        assert!(is_camera_solid(b));
        let center = v(g.x as f32 + 0.5, g.y as f32 + 0.5, g.z as f32 + 0.5);
        assert!(!is_position_clear(
            scene.world(),
            center,
            &CameraCollisionConfig::default(),
            &is_camera_solid
        ));
    }
}

#[test]
fn the_interior_sees_outside_through_the_glass() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let eye = v(
        c.hall.min_x as f32 + 1.5,
        c.levels.base_y as f32 + 1.5,
        c.hall.max_z as f32 - 0.5,
    );
    let view = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: eye,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR,
        background: scene::world::world_background(),
        texture_manager: &manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    // The low west window of the hall.
    let window = *scene
        .castle_windows()
        .glass
        .iter()
        .find(|g| g.x == c.keep.min_x && g.y == c.levels.base_y + 1)
        .expect("a low west window");
    let target = v(
        window.x as f32 + 0.5,
        window.y as f32 + 0.5,
        window.z as f32 + 0.5,
    );
    let ray = Ray::new(eye, (target - eye).normalize());
    let hit = nearest_visible_hit(&view, &ray, WORLD_MAX_DISTANCE).expect("the window");
    assert_eq!(hit.cell, window, "first hit {:?}", hit.cell);
    assert_eq!(hit.block.block_type(), BlockType::Glass);
    // Beyond the glass the ray is in the open courtyard.
    let beyond = v(window.x as f32 - 0.5, target.y, target.z);
    let out = Ray::new(beyond, ray.direction);
    let next = nearest_visible_hit(&view, &out, WORLD_MAX_DISTANCE);
    assert!(
        next.is_none_or(|h| (h.hit.point - beyond).length() > 1.5),
        "{:?}",
        next.map(|h| h.cell)
    );
}

#[test]
fn the_castle_viewpoints_keep_interactive_performance() {
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
    let threads = ParallelRenderConfig::detect();
    for (name, state) in [
        ("L_castle_exterior", pose_l_exterior()),
        ("L2_castle_interior", pose_l2_interior()),
    ] {
        let camera = state.camera(4.0 / 3.0);
        let vs = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: camera.position,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        let mut preview = AdaptivePreview::new();
        let mut fb = Framebuffer::new(400, 300);
        for _ in 0..8 {
            let (w, h) = preview.resolution(800, 600);
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            preview.record(a.elapsed());
        }
        let (w, h) = preview.resolution(800, 600);
        let mut ms = Vec::new();
        for _ in 0..5 {
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            ms.push(a.elapsed().as_secs_f64() * 1e3);
        }
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let fps = 1000.0 / ms[ms.len() / 2];
        println!("GATE16 interactive {name} {w}x{h} fps={fps:.1}");
        assert!(fps >= 15.0, "{name}: {fps:.1} FPS");
    }
}
