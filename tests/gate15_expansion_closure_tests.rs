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

use camera::camera::Camera;
use camera::portal_crossing::{PortalCrossingDetector, PortalCrossingEvent, PortalVolume};
use camera::world_collision::{
    CAMERA_COLLISION_RADIUS, COLLISION_SUBSTEP, CameraCollisionConfig, CollisionState,
    is_camera_passable, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, angles_for, apply_free_fly_input,
};
use core::math::{IVec3, Vec3};
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::preview::AdaptivePreview;
use renderer::raytracer::{RenderQuality, VoxelScene};
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::expansion::{
    PAD_SIDE, VOXEL_BUDGET_SOFT_MAX, VOXEL_BUDGET_TARGET, ground_cell, lower_ground_cell,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

/// Voxel count certified at the end of Gate 14 (HEAD `8f955d4`).
const GATE_14_VOXELS: usize = 6783;
/// Interactive floor of the gate.
const MIN_FPS: f64 = 15.0;

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

fn eye_under(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 - 1.5, t.z as f32 + 0.5)
}

fn eye_over(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 + 2.0, t.z as f32 + 0.5)
}

/// A move with collision that may turn a corner like a walker does.
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

fn screen_right(state: &WorldFreeFlyCameraState) -> Vec3 {
    let camera: Camera = state.camera(4.0 / 3.0);
    camera.basis().right
}

fn moved_by(state: &WorldFreeFlyCameraState, move_right: f32) -> Vec3 {
    let mut moved = *state;
    apply_free_fly_input(
        &mut moved,
        &FreeFlyInput {
            move_right,
            delta_time: 0.1,
            ..Default::default()
        },
    );
    (moved.position - state.position).normalize()
}

/// The World update of one frame as the viewer runs it.
struct Rig {
    scene: WorldScene,
    state: WorldFreeFlyCameraState,
    detector: PortalCrossingDetector,
    collision: CollisionState,
}

impl Rig {
    fn new() -> Self {
        let scene = WorldScene::new();
        let detector =
            PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
        Self {
            scene,
            state: world_free_fly_camera(),
            detector,
            collision: CollisionState::new(),
        }
    }

    fn teleport(&mut self, position: Vec3, target: Vec3) {
        let pose = WorldFreeFlyCameraState::looking_at(position, target);
        self.state.position = pose.position;
        let (yaw, pitch) = angles_for(target - position, self.state.local_up);
        self.state.yaw = yaw;
        self.state.pitch = pitch;
    }

    fn frame(&mut self, input: FreeFlyInput) -> Option<PortalCrossingEvent> {
        let _ = self.state.advance_transition(input.delta_time.max(0.0));
        let mut event = None;
        if input.is_active() {
            let previous = self.state.position;
            apply_free_fly_input(&mut self.state, &input);
            if self.collision.collision_enabled() {
                self.state.position =
                    resolve(&self.scene, previous, self.state.position - previous);
            }
            if self.state.transition().is_none() {
                event = self
                    .detector
                    .detect(previous, self.state.position, self.state.realm);
                if let Some(e) = event {
                    self.state.begin_transition(e);
                }
            }
        }
        event
    }

    fn fly_until(&mut self, frames: usize) -> Option<PortalCrossingEvent> {
        let mut fired = None;
        for _ in 0..frames {
            let e = self.frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 60.0,
                ..Default::default()
            });
            fired = fired.or(e);
            if fired.is_some() && self.state.transition().is_none() {
                break;
            }
        }
        fired
    }
}

/// The eye points of the upper approach: path end, main route, pad centre.
fn upper_route(scene: &WorldScene) -> Vec<Vec3> {
    let l = scene.expansion_layout();
    let mut points = vec![eye_over(*scene.path().cells.last().unwrap())];
    points.push(eye_over(l.overworld_path_anchor));
    for c in &scene.overworld_approach().main {
        points.push(eye_over(*c));
    }
    let pad = l.overworld_castle_pad;
    let y = l.castle_pad_surface_y as f32 + 2.0;
    points.push(v(
        pad.min_x as f32 + 0.5,
        y,
        l.overworld_path_target.z as f32 + 0.5,
    ));
    points.push(v(
        (pad.min_x + pad.max_x) as f32 / 2.0 + 0.5,
        y,
        (pad.min_z + pad.max_z) as f32 / 2.0 + 0.5,
    ));
    points
}

/// The eye points of the lower approach: exit clearance, inverted flight,
/// landing, paved route, pad centre.
fn lower_route(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye_under(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye_under(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    let l = scene.expansion_layout();
    points.push(eye_under(l.red_black_path_anchor));
    for c in &scene.red_black_approach().main {
        points.push(eye_under(*c));
    }
    let pad = l.red_black_fortress_pad;
    let y = l.fortress_pad_bottom_y as f32 - 1.5;
    points.push(v(
        pad.min_x as f32 + 0.5,
        y,
        l.red_black_path_target.z as f32 + 0.5,
    ));
    points.push(v(
        (pad.min_x + pad.max_x) as f32 / 2.0 + 0.5,
        y,
        (pad.min_z + pad.max_z) as f32 / 2.0 + 0.5,
    ));
    points
}

fn decoration_cells(scene: &WorldScene) -> usize {
    let s = scene.expansion_scenery();
    s.upper.cells().len() + s.lower.added_cells().len()
}

// 1: one VoxelWorld in one exact box.
#[test]
fn the_world_is_one_voxel_world_in_one_box() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let bounds = scene.world().bounds().unwrap();
    assert_eq!(bounds.min, IVec3::new(0, -22, 0));
    assert_eq!(
        bounds.max_exclusive,
        IVec3::new(l.overworld_extension.max_x + 1, 13, scene.config().depth)
    );
    let src = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(
        src.matches("VoxelWorld::new()").count(),
        1,
        "the World builds exactly one VoxelWorld"
    );
    let expansion = std::fs::read_to_string("src/scene/expansion.rs").unwrap();
    assert!(!expansion.contains("VoxelWorld::new()"));
    println!("GATE15 bounds {:?}", bounds);
}

// 2, 3: the upper and lower masses form one connected body.
#[test]
fn the_upper_and_lower_masses_are_one_connected_body() {
    let scene = WorldScene::new();
    let cells: HashSet<IVec3> = scene.world().iter().map(|(c, _)| *c).collect();
    let start = *scene.house().floor.first().unwrap();
    let mut seen: HashSet<IVec3> = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(c) = queue.pop_front() {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            if cells.contains(&n) && seen.insert(n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(
        seen.len(),
        cells.len(),
        "the world is not one connected mass"
    );
    // Both expansions and the landing hang off the same body.
    for c in scene
        .overworld_expansion()
        .cells
        .iter()
        .chain(scene.red_black_expansion().cells().iter())
        .chain(scene.inverted_route().layout.platform_cells().iter())
    {
        assert!(seen.contains(c), "{c:?} detached");
    }
    // Nothing floats: the lowest cell is the lower tip, the highest a tree.
    let min_y = cells.iter().map(|c| c.y).min().unwrap();
    assert_eq!(min_y, scene.rhombus().lower_tip_y);
}

// 4: the castle pad is reserved and buildable.
#[test]
fn the_castle_pad_is_reserved() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.overworld_castle_pad;
    assert_eq!(pad.width(), PAD_SIDE);
    assert_eq!(pad.depth(), PAD_SIDE);
    assert_eq!(l.castle_pad_area(), PAD_SIDE * PAD_SIDE);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let top = ground_cell(scene.world(), x, z, 40, -4).unwrap();
            assert_eq!(top.y, l.castle_pad_surface_y, "({x},{z})");
            let b = scene.world().get(top).unwrap();
            assert_eq!(b.block_type(), BlockType::Grass);
            assert_eq!(b.orientation(), Orientation::Up);
            for y in (top.y + 1)..=40 {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x},{y},{z})"
                );
            }
            assert!(clear(&scene, eye_over(top)));
        }
    }
    println!(
        "GATE15 castle pad x {}..={} z {}..={} surface y={}",
        pad.min_x, pad.max_x, pad.min_z, pad.max_z, l.castle_pad_surface_y
    );
}

// 5: the fortress pad is reserved and buildable (downward).
#[test]
fn the_fortress_pad_is_reserved() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    assert_eq!(pad.width(), PAD_SIDE);
    assert_eq!(pad.depth(), PAD_SIDE);
    assert_eq!(l.fortress_pad_area(), PAD_SIDE * PAD_SIDE);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let ground = lower_ground_cell(scene.world(), x, z, -60, 0).unwrap();
            assert_eq!(ground.y, l.fortress_pad_bottom_y, "({x},{z})");
            assert_eq!(
                scene.world().get(ground).unwrap().orientation(),
                Orientation::Down
            );
            for y in -60..ground.y {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x},{y},{z})"
                );
            }
            assert!(clear(&scene, eye_under(ground)));
        }
    }
    println!(
        "GATE15 fortress pad x {}..={} z {}..={} bottom y={}",
        pad.min_x, pad.max_x, pad.min_z, pad.max_z, l.fortress_pad_bottom_y
    );
}

// 6: both approach paths are continuous with collision on, both ways.
#[test]
fn both_approach_paths_are_continuous() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    walk_both_ways(&scene, &upper_route(&scene));
    walk_both_ways(&scene, &lower_route(&scene));
    let upper = scene.overworld_approach();
    let lower = scene.red_black_approach();
    assert!(upper.main.len() >= 30 && lower.main.len() >= 20);
    println!(
        "GATE15 paths: upper main={} forecourt={} accents={} | lower main={} accents={}",
        upper.main.len(),
        upper.forecourt.len(),
        upper.accents.len(),
        lower.main.len(),
        lower.accents.len()
    );
}

// 7: the portal traversal of Gate 14 (route B) still passes.
#[test]
fn the_portal_traversal_still_passes() {
    let mut rig = Rig::new();
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    let e = rig.fly_until(400);
    assert_eq!(e, Some(PortalCrossingEvent::OverworldToRedBlack));
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, v(0.0, -1.0, 0.0));
    assert!(clear(&rig.scene, rig.state.position));
    let mut deepest = rig.state.position.y;
    for _ in 0..600 {
        rig.frame(FreeFlyInput {
            move_forward: 1.0,
            move_up: 1.0,
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
        deepest = deepest.min(rig.state.position.y);
    }
    assert!(rig.state.position.z < 5.0, "{:?}", rig.state.position);
    assert!(deepest < -14.0, "{deepest}");
    assert!(clear(&rig.scene, rig.state.position));
    // The canonical route itself, both ways.
    let scene = &rig.scene;
    let mut points = vec![eye_over(*scene.path().cells.last().unwrap())];
    for (t, _) in &scene.upper_descent().treads {
        points.push(eye_over(*t));
    }
    points.extend([
        v(16.5, -7.5, 16.5),
        v(16.5, -7.5, 14.5),
        v(16.5, -7.5, 13.5),
    ]);
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye_under(*t));
    }
    let l = scene.inverted_route().layout;
    points.push(eye_under(IVec3::new(l.x, l.platform_y, l.platform_z_max)));
    walk_both_ways(scene, &points);
}

// 8: the house is reachable from the path through its door.
#[test]
fn the_house_is_accessible() {
    let scene = WorldScene::new();
    let door = &scene.house_exterior().door;
    assert_eq!(door.len(), 2);
    for d in door {
        let b = scene.world().get(*d).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodDoor);
        assert!(is_camera_passable(b.block_type()));
    }
    for w in &scene.house().walls {
        assert!(is_camera_solid(scene.world().get(*w).unwrap()));
    }
    let start = scene.path().start().unwrap();
    let lower = door[0];
    let outside = v(
        lower.x as f32 + 0.5,
        lower.y as f32 + 0.5,
        lower.z as f32 + 1.5,
    );
    let inside = v(
        lower.x as f32 + 0.5,
        lower.y as f32 + 0.5,
        lower.z as f32 - 1.5,
    );
    let through = v(
        lower.x as f32 + 0.5,
        lower.y as f32 + 0.5,
        lower.z as f32 + 0.5,
    );
    let points = [eye_over(start), outside, through, inside];
    walk_both_ways(&scene, &points);
    // Along the whole path to its end and back.
    let path: Vec<Vec3> = scene.path().cells.iter().map(|c| eye_over(*c)).collect();
    walk_both_ways(&scene, &path);
}

// 9: A/D follow the screen in both realms, over both expansions too.
#[test]
fn a_and_d_follow_the_screen() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad_x = (l.overworld_castle_pad.min_x + l.overworld_castle_pad.max_x) as f32 / 2.0;
    for state in [
        world_free_fly_camera(),
        WorldFreeFlyCameraState::looking_at(v(pad_x, 10.0, 22.0), v(pad_x, 4.5, 8.5)),
        red_black(v(16.5, -16.5, 2.5), v(16.5, -16.5, -5.0)),
        red_black(v(pad_x, -22.0, 20.0), v(pad_x, -12.5, 8.5)),
    ] {
        let right = screen_right(&state);
        assert!(
            moved_by(&state, 1.0).dot(right) > 0.999,
            "{:?}",
            state.realm
        );
        assert!(
            moved_by(&state, -1.0).dot(right) < -0.999,
            "{:?}",
            state.realm
        );
    }
}

// 10: Catalog 39 of 39, single crate, no Raylib 3D.
#[test]
fn catalog_is_39_of_39_and_crates_untouched() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    assert_eq!(official_entries().len(), 39);
    assert_eq!(CatalogScene::new().entries().len(), 41);
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = &manifest[manifest.find("[dependencies]").unwrap()..];
    let deps = &deps[..deps.find("\n[").map_or(deps.len(), |i| i)];
    assert_eq!(
        deps.lines().filter(|l| l.contains('=')).collect::<Vec<_>>(),
        vec!["raylib = \"5.5\""]
    );
    let expansion = std::fs::read_to_string("src/scene/expansion.rs").unwrap();
    for krate in [
        "raylib", "rayon", "rand", "noise", "glam", "nalgebra", "cgmath",
    ] {
        assert!(
            !expansion.contains(&format!("{krate}::"))
                && !expansion.contains(&format!("use {krate}")),
            "{krate} in expansion.rs"
        );
    }
    for token in ["Camera3D", "DrawCube", "BeginMode3D", "DrawModel"] {
        assert!(!expansion.contains(token), "{token} in expansion.rs");
    }
}

// 11: interactive performance over the original and expanded viewpoints.
#[test]
fn interactive_performance_meets_the_floor() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let scene = WorldScene::new();
    let materials: MaterialLibrary = world_materials(&textures);
    let lights = world_lights();
    let threads = ParallelRenderConfig::detect();
    let l = scene.expansion_layout();
    let pad_x = (l.overworld_castle_pad.min_x + l.overworld_castle_pad.max_x) as f32 / 2.0;
    for (name, state) in [
        ("A_full_diorama_home", world_free_fly_camera()),
        (
            "J_expanded_overworld",
            WorldFreeFlyCameraState::looking_at(v(pad_x, 12.0, 22.0), v(pad_x, 4.5, 8.5)),
        ),
        (
            "K_expanded_red_black",
            red_black(v(pad_x, -22.0, 20.0), v(pad_x, -12.5, 8.5)),
        ),
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
        println!("GATE15 interactive {name} {w}x{h} fps={fps:.1}");
        assert!(fps >= MIN_FPS, "{name}: {fps:.1} FPS");
    }
}

// 12: the voxel budget.
#[test]
fn the_voxel_budget_is_respected() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    let decoration = decoration_cells(&scene);
    assert!(n > GATE_14_VOXELS);
    assert!(n - decoration <= VOXEL_BUDGET_TARGET, "{n} - {decoration}");
    assert!(n <= VOXEL_BUDGET_SOFT_MAX, "{n}");
    println!(
        "GATE15 voxels={n} delta={} decoration={decoration} upper={} lower={}",
        n - GATE_14_VOXELS,
        scene.overworld_expansion().cells.len(),
        scene.red_black_expansion().cells().len()
    );
}

// 13: every landmark of the original diorama is preserved.
#[test]
fn landmarks_are_preserved() {
    let scene = WorldScene::new();
    let w = scene.world();
    let has = |c: &IVec3, t: BlockType| w.get(*c).map(|b| b.block_type()) == Some(t);
    let house = scene.house_exterior();
    assert_eq!(house.door.len(), 2);
    assert!(house.door.iter().all(|c| has(c, BlockType::WoodDoor)));
    assert!(house.glass.iter().all(|c| has(c, BlockType::Glass)));
    assert!(house.fence.iter().all(|c| has(c, BlockType::Fence)));
    assert!(
        house
            .lamps
            .iter()
            .all(|c| has(c, BlockType::RedstoneLampLit))
    );
    assert!(house.roof.iter().all(|(c, _, _)| w.contains(*c)));
    assert!(scene.house().floor.iter().all(|c| w.contains(*c)));
    let pond = scene.pond();
    assert!(!pond.water.is_empty() && pond.water.iter().all(|c| has(c, BlockType::Water)));
    assert!(pond.sand.iter().all(|c| has(c, BlockType::Sand)));
    assert_eq!(scene.trees().len(), 4);
    for t in scene.trees() {
        assert!(t.trunk.iter().all(|c| has(c, BlockType::Log)));
        assert!(t.leaves.iter().all(|c| has(c, BlockType::Leaves)));
    }
    let portal = scene.portal();
    assert_eq!(portal.frame.len(), 18);
    assert_eq!(portal.core.len(), 12);
    assert!(
        portal
            .frame
            .iter()
            .all(|c| has(c, BlockType::PortalFrameRedObsidian))
    );
    assert!(
        portal
            .core
            .iter()
            .all(|c| has(c, BlockType::PortalCoreDarkCrimson))
    );
    assert!(
        scene
            .upper_descent()
            .treads
            .iter()
            .all(|(c, _)| has(c, BlockType::WoodStairs))
    );
    let route = scene.inverted_route();
    assert_eq!(route.treads.len(), 10);
    assert!(
        route
            .treads
            .iter()
            .all(|(c, o)| has(c, BlockType::WoodStairs) && *o == Orientation::Down)
    );
    assert!(
        route
            .layout
            .platform_cells()
            .iter()
            .all(|c| has(c, BlockType::DeepslateBricks))
    );
    assert!(
        route
            .layout
            .exit_clearance()
            .iter()
            .all(|c| !w.contains(*c))
    );
    let tip = scene.rhombus().lower_tip_y;
    assert!(w.iter().any(|(c, _)| c.y == tip));
    let counts: Vec<(usize, usize)> = scene
        .families()
        .iter()
        .map(|f| (f.symbols.len(), f.accents.len()))
        .collect();
    assert_eq!(counts, vec![(25, 12), (21, 6), (23, 10)]);
    for f in scene.families() {
        assert!(f.cells().iter().all(|c| w.contains(*c)));
    }
    assert!(scene.lower_surface().cells.iter().all(|c| w.contains(*c)));
}

// 14: Gate 14 navigation and the Gate 13.6 renderer are untouched.
#[test]
fn gate_14_navigation_and_gate_13_6_renderer_are_untouched() {
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert_eq!(COLLISION_SUBSTEP, 0.2);
    let s = CollisionState::new();
    assert!(s.collision_enabled());
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "8f955d4",
            "HEAD",
            "--",
            "src/camera",
            "src/renderer",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let diff = String::from_utf8_lossy(&out.stdout);
            assert!(
                diff.trim().is_empty(),
                "camera/renderer changed since Gate 14:\n{diff}"
            );
        }
    }
}

/// Release benchmark of the gate: A, C, F and the two expanded viewpoints.
///
/// `cargo test --release --test gate15_expansion_closure_tests -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_expansion() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let scene = WorldScene::new();
    let materials: MaterialLibrary = world_materials(&textures);
    let lights = world_lights();
    let threads = ParallelRenderConfig::detect();
    let l = scene.expansion_layout();
    let pad_x = (l.overworld_castle_pad.min_x + l.overworld_castle_pad.max_x) as f32 / 2.0;
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let poses = [
        ("A_full_diorama_home", world_free_fly_camera()),
        (
            "C_portal_cutaway",
            WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
        ),
        (
            "F_just_after_crossing",
            red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
        ),
        (
            "J_expanded_overworld",
            WorldFreeFlyCameraState::looking_at(v(pad_x, 12.0, 22.0), v(pad_x, 4.5, 8.5)),
        ),
        (
            "K_expanded_red_black",
            red_black(v(pad_x, -22.0, 20.0), v(pad_x, -12.5, 8.5)),
        ),
    ];
    println!(
        "BENCH workers={} voxels={}",
        threads.workers,
        scene.world().len()
    );
    let mut rgba = Vec::new();
    for (name, state) in poses {
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
        for _ in 0..12 {
            let (w, h) = preview.resolution(800, 600);
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            preview.record(a.elapsed());
        }
        let (w, h) = preview.resolution(800, 600);
        let mut total = Vec::new();
        for _ in 0..9 {
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            fb.write_rgba8(&mut rgba);
            total.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let mut full = Vec::new();
        let mut full_fb = Framebuffer::new(800, 600);
        for _ in 0..3 {
            let a = Instant::now();
            render_parallel(&mut full_fb, &camera, &vs, RenderQuality::Full, &threads);
            full.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let t = p50(&mut total);
        println!(
            "BENCH {name} interactive resolution={w}x{h} total_ms={t:.1} fps={:.1} full_ms={:.1}",
            1000.0 / t,
            p50(&mut full)
        );
    }
}
