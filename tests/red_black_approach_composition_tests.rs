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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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

use camera::projection::primary_ray;
use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm, angles_for};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, nearest_visible_hit};
use scene::block_type::BlockType;
use scene::fortress::is_symbol;
use scene::light::Light;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_lights,
    world_materials,
};
use std::collections::HashSet;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn cfg() -> CameraCollisionConfig {
    CameraCollisionConfig::default()
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(scene.world(), p, &cfg(), &is_camera_solid)
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(scene.world(), from, delta, &cfg(), &is_camera_solid)
}

/// The inverted viewer's eye under a ground cell.
fn eye(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 - 1.5, c.z as f32 + 0.5)
}

fn step(scene: &WorldScene, from: Vec3, target: Vec3) -> Vec3 {
    let direct = resolve(scene, from, target - from);
    if (direct - target).length() < 1e-3 {
        return direct;
    }
    let (f, t) = (from, target);
    for corner in [
        v(t.x, f.y, t.z),
        v(f.x, t.y, f.z),
        v(t.x, f.y, f.z),
        v(f.x, f.y, t.z),
        v(t.x, t.y, f.z),
        v(f.x, t.y, t.z),
    ] {
        let mid = resolve(scene, from, corner - from);
        if (mid - corner).length() < 1e-3 {
            let end = resolve(scene, mid, target - mid);
            if (end - target).length() < 1e-3 {
                return end;
            }
        }
    }
    direct
}

fn walk(scene: &WorldScene, points: &[Vec3]) {
    let mut position = points[0];
    for (i, target) in points.iter().enumerate().skip(1) {
        let reached = step(scene, position, *target);
        assert!(
            (reached - *target).length() < 1e-3,
            "segment {i}: stuck at {reached:?} short of {target:?}"
        );
        position = reached;
    }
}

fn walk_both_ways(scene: &WorldScene, points: &[Vec3]) {
    walk(scene, points);
    let mut back = points.to_vec();
    back.reverse();
    walk(scene, &back);
}

fn red_black(position: Vec3, target: Vec3) -> WorldFreeFlyCameraState {
    let mut s = WorldFreeFlyCameraState::looking_at(position, target);
    s.realm = WorldRealm::RedBlack;
    s.local_up = WorldRealm::RedBlack.up();
    let (yaw, pitch) = angles_for(target - position, s.local_up);
    s.yaw = yaw;
    s.pitch = pitch;
    s
}

fn tree_cells(scene: &WorldScene) -> HashSet<IVec3> {
    scene
        .red_black_identity()
        .grove
        .cells()
        .into_iter()
        .collect()
}

fn gate_facing(scene: &WorldScene) -> Vec<IVec3> {
    let l = scene.fortress_layout();
    scene
        .red_black_approach()
        .main
        .iter()
        .copied()
        .filter(|c| {
            c.z >= l.gate.base.z - 3
                && (c.x == l.gate.base.x - 2
                    || (c.x == l.gate.base.x - 1 && c.z >= l.gate.base.z - 1))
        })
        .collect()
}

// 1
#[test]
fn the_gate_sightline_is_open() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let trees = tree_cells(&scene);
    let fences: HashSet<IVec3> = scene
        .red_black_identity()
        .composition
        .fences
        .iter()
        .copied()
        .collect();
    for g in &scene.red_black_approach().main {
        let from = eye(*g);
        for target in l.gate.cells() {
            let to = v(
                target.x as f32 + 0.5,
                target.y as f32 + 0.5,
                target.z as f32 + 0.5,
            );
            for i in 0..=40 {
                let p = from + (to - from) * (i as f32 / 40.0);
                let cell = IVec3::new(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
                assert!(
                    !trees.contains(&cell) && !fences.contains(&cell),
                    "{cell:?} hides the gate from {g:?}"
                );
            }
        }
    }
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
    for c in gate_facing(&scene) {
        let from = eye(c);
        let view = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: from,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR,
            background: scene::world::world_background(),
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        let sees = scene.red_black_identity().gatehouse.door.iter().any(|d| {
            let target = v(d.x as f32 + 0.5, d.y as f32 + 0.5, d.z as f32 + 0.5);
            nearest_visible_hit(
                &view,
                &Ray::new(from, (target - from).normalize()),
                WORLD_MAX_DISTANCE,
            )
            .is_some_and(|h| h.block.block_type() == BlockType::RedBlackWoodDoor)
        });
        assert!(sees, "door hidden from {c:?}");
    }
    let cp = &scene.red_black_identity().composition;
    println!(
        "GATE17_5 composition: edge={} fences={} shoulder_trees={:?} trees={} identity_voxels={} voxels={}",
        cp.edge.len(),
        cp.fences.len(),
        cp.tree_sites,
        scene.red_black_identity().grove.trees.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn the_path_remains_clear() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let l = scene.fortress_layout();
    let r = scene.inverted_route().layout;
    let mut points = vec![eye(IVec3::new(r.x, r.platform_y, r.platform_z_max))];
    points.extend(scene.red_black_approach().main.iter().map(|c| eye(*c)));
    points.extend(l.main_route().iter().map(|c| eye(*c)));
    walk_both_ways(&scene, &points);
    for c in &scene.red_black_approach().main {
        for dy in 1..=2 {
            assert!(
                !scene.world().contains(IVec3::new(c.x, c.y - dy, c.z)),
                "{c:?}"
            );
        }
    }
}

// 3
#[test]
fn fences_do_not_trap_the_camera() {
    let scene = WorldScene::new();
    let cp = &scene.red_black_identity().composition;
    assert!(
        !cp.fences.is_empty() && cp.fences.len() < cp.edge.len(),
        "fragments, not a wall"
    );
    let path: HashSet<(i32, i32)> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| (c.x, c.z))
        .collect();
    for f in &cp.fences {
        assert_eq!(
            scene.world().get(*f).unwrap().block_type(),
            BlockType::RedBlackFence
        );
        assert!(!path.contains(&(f.x, f.z)), "{f:?} on the path");
        assert!(
            cp.edge.contains(&IVec3::new(f.x, f.y + 1, f.z)),
            "{f:?} stands on the edge"
        );
    }
    // Every path eye is free, and a gap in the fragments opens the shoulder.
    for c in &scene.red_black_approach().main {
        assert!(clear(&scene, eye(*c)), "{c:?}");
    }
    let gap = cp
        .edge
        .iter()
        .find(|g| !cp.fences.contains(&IVec3::new(g.x, g.y - 1, g.z)))
        .expect("a gap");
    let path_cell = scene
        .red_black_approach()
        .main
        .iter()
        .find(|p| p.x == gap.x && p.z == gap.z + 1)
        .unwrap();
    if (path_cell.y - gap.y).abs() <= 1 {
        walk_both_ways(&scene, &[eye(*path_cell), eye(*gap)]);
    }
}

// 4
#[test]
fn trees_do_not_block_the_entrance() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let trees = tree_cells(&scene);
    let mut keep_out: HashSet<(i32, i32)> = l.gate_corridor().into_iter().collect();
    for (x, z) in l.gate.columns() {
        keep_out.insert((x - 1, z));
        keep_out.insert((x - 2, z));
    }
    for c in &trees {
        assert!(
            !keep_out.contains(&(c.x, c.z)),
            "{c:?} in front of the gate"
        );
    }
    let n = scene.red_black_identity().grove.trees.len();
    assert!((4..=7).contains(&n), "{n} trees");
}

// 5
#[test]
fn the_fortress_silhouette_stays_visible() {
    let scene = WorldScene::new();
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
    let pad = scene.fortress_layout().footprint;
    let ground = scene.fortress_layout().levels.ground_y;
    // The same world without its trees, to measure what they screen.
    let trees = tree_cells(&scene);
    let mut bare = VoxelWorld::new();
    for (c, b) in scene.world().iter() {
        if !trees.contains(c) {
            bare.insert(*c, *b);
        }
    }
    let fortress_hit = |h: &renderer::raytracer::VoxelHit| {
        pad.contains(h.cell.x, h.cell.z)
            && h.cell.y < ground
            && !matches!(
                h.block.block_type(),
                BlockType::RedBlackLeaves | BlockType::RedBlackLog
            )
    };
    for (name, state) in [
        ("K", red_black(v(30.5, -22.0, 20.0), v(30.5, -12.5, 8.5))),
        ("M", red_black(v(16.0, -30.0, 22.0), v(30.5, -16.0, 8.5))),
        ("O", red_black(v(24.6, -17.0, 3.6), v(26.0, -13.6, 8.9))),
    ] {
        let camera = state.camera(4.0 / 3.0);
        let with = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: camera.position,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR,
            background: state.background(),
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        let without = VoxelScene {
            world: &bare,
            ..with
        };
        let (w, h) = (96, 72);
        let (mut kept, mut total) = (0, 0);
        for y in 0..h {
            for x in 0..w {
                let ray = primary_ray(&camera, x, y, w, h);
                if nearest_visible_hit(&without, &ray, WORLD_MAX_DISTANCE)
                    .is_some_and(|h| fortress_hit(&h))
                {
                    total += 1;
                    if nearest_visible_hit(&with, &ray, WORLD_MAX_DISTANCE)
                        .is_some_and(|h| fortress_hit(&h))
                    {
                        kept += 1;
                    }
                }
            }
        }
        println!("GATE17_5 silhouette {name}: fortress kept {kept} of {total}");
        assert!(
            total > 300,
            "{name}: the fortress is barely in frame ({total})"
        );
        assert!(
            kept * 10 >= total * 9,
            "{name}: trees screen the fortress ({kept} of {total})"
        );
    }
}

// 6
#[test]
fn no_family_focal_scene_is_obstructed() {
    let scene = WorldScene::new();
    let added: Vec<IVec3> = tree_cells(&scene)
        .into_iter()
        .chain(
            scene
                .red_black_identity()
                .composition
                .fences
                .iter()
                .copied(),
        )
        .collect();
    let focal: Vec<IVec3> = scene
        .families()
        .iter()
        .flat_map(|f| f.symbols.iter().chain(f.accents.iter()).copied())
        .collect();
    for c in &added {
        for f in &focal {
            assert!(
                (f.x - c.x).abs() >= 2 || (f.z - c.z).abs() >= 2,
                "{c:?} crowds {f:?}"
            );
        }
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            assert!(
                !scene
                    .world()
                    .get(n)
                    .is_some_and(|b| is_symbol(b.block_type())),
                "{c:?} touches a symbol"
            );
        }
    }
}

// 7
#[test]
fn no_light_is_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
}
