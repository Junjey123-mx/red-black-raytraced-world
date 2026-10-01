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

use camera::portal_crossing::{PortalCrossingDetector, PortalCrossingEvent, PortalVolume};
use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, angles_for, apply_free_fly_input,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::world::{WorldScene, world_free_fly_camera};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn cfg() -> CameraCollisionConfig {
    CameraCollisionConfig::default()
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

/// The Gate 16 castle route as eye points (path end to the upper room).
fn castle_route(scene: &WorldScene) -> Vec<Vec3> {
    let l = scene.expansion_layout();
    let mut points = vec![eye_over(*scene.path().cells.last().unwrap())];
    points.push(eye_over(l.overworld_path_anchor));
    for c in &scene.overworld_approach().main {
        points.push(eye_over(*c));
    }
    for c in &scene.castle_layout().interior_route {
        points.push(eye_over(*c));
    }
    points
}

/// The full lower route: exit clearance, inverted stairs, landing, corinto
/// path, gatehouse door, courtyard, keep, stair, upper room.
fn lower_route(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye_under(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye_under(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    points.push(eye_under(scene.expansion_layout().red_black_path_anchor));
    for c in &scene.red_black_approach().main {
        points.push(eye_under(*c));
    }
    for c in scene.fortress_layout().main_route() {
        points.push(eye_under(*c));
    }
    points
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

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(scene.world(), p, &cfg(), &is_camera_solid)
}

/// The Overworld half of the journey: the castle, then the house path down
/// the upper stairs to the portal's front.
fn overworld_to_portal(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = castle_route(scene);
    points.reverse();
    points.push(eye_over(*scene.path().cells.last().unwrap()));
    for (t, _) in &scene.upper_descent().treads {
        points.push(eye_over(*t));
    }
    points.extend([v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5)]);
    points
}

// The whole two-realm journey of Gate 17.5, collision on, noclip off:
// Overworld castle -> portal -> roll -> Red-Black -> inverted stairs ->
// corinto path -> Red-Black door -> courtyard -> keep -> towers, and back
// through the portal to the Overworld.
#[test]
fn the_complete_two_realm_journey_runs_both_ways() {
    let mut rig = Rig::new();
    assert!(rig.collision.collision_enabled());
    let scene = WorldScene::new();
    // 1. The Overworld leg, castle to the portal's front, both ways.
    let over = overworld_to_portal(&scene);
    walk_both_ways(&scene, &over);
    // 2. Through the membrane with the real detector: roll into Red-Black.
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    assert_eq!(
        rig.fly_until(400),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, v(0.0, -1.0, 0.0));
    assert!(rig.state.transition().is_none(), "the roll finished");
    assert!(clear(&rig.scene, rig.state.position));
    // 3. The lower leg: stairs, path, door, courtyard, keep, upper room.
    let lower = lower_route(&scene);
    walk_both_ways(&scene, &lower);
    let id = scene.red_black_identity();
    let door_cols: Vec<(i32, i32)> = id.gatehouse.door.iter().map(|c| (c.x, c.z)).collect();
    assert!(
        lower
            .iter()
            .any(|p| door_cols.contains(&(p.x.floor() as i32, p.z.floor() as i32))),
        "the route passes the Red-Black door"
    );
    for c in &id.gatehouse.door {
        assert!(is_camera_passable(
            scene.world().get(*c).unwrap().block_type()
        ));
    }
    // 4. Every tower zone from the courtyard, the fourth one included.
    let l = scene.fortress_layout();
    for r in &l.interior_routes[1..] {
        walk_both_ways(&scene, &r.iter().map(|c| eye_under(*c)).collect::<Vec<_>>());
    }
    let t4 = &id.fourth_tower;
    let g = l.levels.ground_y;
    let e = t4.entrance[0];
    walk_both_ways(
        &scene,
        &[
            eye_under(IVec3::new(e.x - 1, g, e.z)),
            eye_under(IVec3::new(e.x, g, e.z)),
            eye_under(IVec3::new(t4.core.0, g, t4.core.1)),
        ],
    );
    // 5. Back through the membrane from the Red-Black side: reverse roll.
    rig.teleport(v(16.5, -7.5, 13.0), v(16.5, -7.5, 16.0));
    assert_eq!(
        rig.fly_until(400),
        Some(PortalCrossingEvent::RedBlackToOverworld)
    );
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.local_up, WorldRealm::Overworld.up());
    assert!(clear(&rig.scene, rig.state.position));
    // 6. Home again: the Overworld leg runs in reverse to the castle.
    let mut home = over.clone();
    home.reverse();
    walk(&scene, &home);
    // The camera solidity contract: only the portal membrane and the two doors pass.
    let passable: Vec<BlockType> = BlockType::ALL
        .iter()
        .copied()
        .filter(|t| is_camera_passable(*t))
        .collect();
    assert_eq!(passable.len(), 3);
    let _ = world_free_fly_camera();
    println!(
        "GATE17_5 journey: overworld={} lower={} eye points",
        over.len(),
        lower.len()
    );
}
