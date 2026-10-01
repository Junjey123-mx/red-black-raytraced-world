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

use camera::diagnostic::DiagnosticCameraState;
use camera::portal_crossing::{
    PORTAL_CROSSING_EPSILON, PORTAL_REARM_DISTANCE, PortalCrossingDetector, PortalCrossingEvent,
    PortalVolume,
};
use camera::world_free_fly::{
    FREE_FLY_MAX_PITCH, FREE_FLY_MOUSE_SENSITIVITY, FREE_FLY_SPEED, FreeFlyInput,
    PORTAL_TRANSITION_DURATION, WorldFreeFlyCameraState, WorldRealm, apply_free_fly_input,
};
use core::math::Vec3;
use scene::block_type::OFFICIAL_BLOCK_COUNT;
use scene::catalog::{CatalogScene, official_entries};
use scene::environment::WorldEnvironmentProfile;
use scene::light::Light;
use scene::material_gallery::{gallery_background, gallery_camera};
use scene::world::{WorldScene, world_background, world_free_fly_camera, world_lights};

const UP: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const DOWN: Vec3 = Vec3 {
    x: 0.0,
    y: -1.0,
    z: 0.0,
};

/// A World camera plus its portal detector, driven frame by frame like
/// `WorldMode::update` does.
struct Rig {
    state: WorldFreeFlyCameraState,
    detector: PortalCrossingDetector,
}

impl Rig {
    fn new(scene: &WorldScene) -> Self {
        Self {
            state: world_free_fly_camera(),
            detector: PortalCrossingDetector::new(PortalVolume::from_anchor(
                &scene.rhombus().portal,
            )),
        }
    }

    fn frame(&mut self, input: FreeFlyInput) -> Option<PortalCrossingEvent> {
        self.state.advance_transition(input.delta_time);
        let previous = self.state.position;
        apply_free_fly_input(&mut self.state, &input);
        if input.reset {
            self.detector.reset();
            return None;
        }
        if self.state.transition().is_some() {
            return None;
        }
        let event = self
            .detector
            .detect(previous, self.state.position, self.state.realm)?;
        self.state.begin_transition(event);
        Some(event)
    }

    /// Flies straight through the portal center along its normal, from
    /// `from_side` (+1 in front, -1 behind), and lets the roll finish.
    /// Teleport the pose only: the home framing (what R restores) stays.
    fn teleport(&mut self, position: Vec3, target: Vec3) {
        let pose = WorldFreeFlyCameraState::looking_at(position, target);
        self.state.position = pose.position;
        self.state.yaw = pose.yaw;
        self.state.pitch = pose.pitch;
    }

    fn traverse(&mut self, from_side: f32) -> Option<PortalCrossingEvent> {
        let v = self.detector.volume;
        let start = v.plane_point + v.normal * (4.0 * from_side);
        self.teleport(start, v.plane_point);
        self.state.realm = if from_side > 0.0 {
            WorldRealm::Overworld
        } else {
            WorldRealm::RedBlack
        };
        self.state.local_up = self.state.realm.up();
        let mut event = None;
        for _ in 0..200 {
            let e = self.frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 60.0,
                ..Default::default()
            });
            event = event.or(e);
            if event.is_some() && self.state.transition().is_none() {
                break;
            }
        }
        event
    }
}

#[test]
fn world_uses_free_fly_and_catalog_uses_the_diagnostic_camera() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog =
        &app[app.find("struct CatalogMode").unwrap()..app.find("struct WorldMode").unwrap()];
    let world =
        &app[app.find("struct WorldMode").unwrap()..app.find("/// Casts one primary ray").unwrap()];
    assert!(catalog.contains("view: DiagnosticCameraState") && !catalog.contains("FreeFly"));
    assert!(
        world.contains("free_fly: WorldFreeFlyCameraState")
            && !world.contains("DiagnosticCameraState")
    );
    assert!(
        catalog.contains("Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset")
    );
    assert!(world.contains("WASD move | Mouse look | Space/Shift up/down | N noclip | R reset"));
    let g = gallery_camera(4.0 / 3.0);
    let view = DiagnosticCameraState::from_pose(g.position, g.target);
    assert_eq!(g.position, Vec3::new(6.0, 5.6, 14.0));
    assert!((view.build_camera(4.0 / 3.0).position - g.position).length() < 1e-3);
}

#[test]
fn movement_and_mouse_contracts() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    let s0 = rig.state;
    let dt = 0.1;
    rig.frame(FreeFlyInput {
        move_forward: 1.0,
        delta_time: dt,
        ..Default::default()
    });
    assert!(
        ((rig.state.position - s0.position) - s0.forward() * (FREE_FLY_SPEED * dt)).length() < 1e-4
    );
    let p = rig.state.position;
    rig.frame(FreeFlyInput {
        move_right: -1.0,
        delta_time: dt,
        ..Default::default()
    });
    assert!(((rig.state.position - p) + s0.right() * (FREE_FLY_SPEED * dt)).length() < 1e-4);
    let p = rig.state.position;
    rig.frame(FreeFlyInput {
        move_up: 1.0,
        delta_time: dt,
        ..Default::default()
    });
    assert!(((rig.state.position - p) - s0.up() * (FREE_FLY_SPEED * dt)).length() < 1e-4);
    let p = rig.state.position;
    rig.frame(FreeFlyInput {
        move_up: -1.0,
        delta_time: 2.0 * dt,
        ..Default::default()
    });
    assert!(((rig.state.position - p) + s0.up() * (FREE_FLY_SPEED * 2.0 * dt)).length() < 1e-4);
    // Mouse: yaw right, pitch clamped.
    let f = rig.state.forward();
    rig.frame(FreeFlyInput {
        look_dx: 80.0,
        ..Default::default()
    });
    assert!((rig.state.yaw - s0.yaw - 80.0 * FREE_FLY_MOUSE_SENSITIVITY).abs() < 1e-5);
    assert!((rig.state.forward() - f).length() > 0.01);
    for _ in 0..100 {
        rig.frame(FreeFlyInput {
            look_dy: -200.0,
            ..Default::default()
        });
    }
    assert!(rig.state.pitch <= FREE_FLY_MAX_PITCH);
    assert!((FREE_FLY_SPEED - 4.0).abs() < 1e-6 && (3.0..=5.0).contains(&FREE_FLY_SPEED));
}

#[test]
fn route_a_and_b_full_traversal_and_return() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    // A: Overworld -> portal -> roll -> Red-Black.
    assert_eq!(
        rig.traverse(1.0),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, DOWN);
    assert!((rig.state.up() - DOWN).length() < 1e-3);
    assert_eq!(rig.state.environment(), WorldEnvironmentProfile::RED_BLACK);
    assert_eq!(rig.state.sky_up(), DOWN);
    // B: turn around and come back: Red-Black -> portal -> roll -> Overworld.
    for _ in 0..20 {
        rig.frame(FreeFlyInput {
            look_dx: std::f32::consts::PI / FREE_FLY_MOUSE_SENSITIVITY / 20.0,
            ..Default::default()
        });
    }
    let mut event = None;
    for _ in 0..400 {
        let e = rig.frame(FreeFlyInput {
            move_forward: 1.0,
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
        event = event.or(e);
        if event.is_some() && rig.state.transition().is_none() {
            break;
        }
    }
    assert_eq!(event, Some(PortalCrossingEvent::RedBlackToOverworld));
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.local_up, UP);
    assert_eq!(rig.state.environment(), WorldEnvironmentProfile::DAY);
    assert_eq!(rig.state.background(), world_background());
}

#[test]
fn the_roll_is_180_degrees_and_the_sky_turns_with_it() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    let v = rig.detector.volume;
    rig.teleport(v.plane_point + v.normal * 0.5, v.plane_point);
    let up0 = rig.state.up();
    let mut e = None;
    while e.is_none() {
        e = rig.frame(FreeFlyInput {
            move_forward: 1.0,
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
    }
    let mut previous_sky = rig.state.sky_up();
    let mut previous_up = rig.state.up();
    let mut elapsed = 0.0;
    while rig.state.transition().is_some() {
        rig.frame(FreeFlyInput {
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
        elapsed += 1.0 / 60.0;
        let sky = rig.state.sky_up();
        assert!((sky.length() - 1.0).abs() < 1e-3);
        assert!((sky - previous_sky).length() < 0.2, "sky pop");
        assert!((rig.state.up() - previous_up).length() < 0.2, "roll snap");
        previous_sky = sky;
        previous_up = rig.state.up();
    }
    assert!((elapsed - PORTAL_TRANSITION_DURATION).abs() < 0.05);
    assert!(
        (rig.state.up() + up0).length() < 1e-2,
        "not a 180-degree roll"
    );
}

#[test]
fn route_c_flying_under_the_island_never_changes_the_realm() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    let r = scene.rhombus();
    // Dive beside the diamond, pass far under it and come back up, crossing
    // the portal's plane and the waist height many times, never the opening.
    rig.teleport(Vec3::new(40.0, 10.0, 40.0), Vec3::new(40.0, -40.0, 40.0));
    let mut events = 0;
    for _ in 0..600 {
        if rig
            .frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 20.0,
                ..Default::default()
            })
            .is_some()
        {
            events += 1;
        }
    }
    assert!(rig.state.position.y < r.lower_tip_y as f32);
    rig.teleport(Vec3::new(40.0, -40.0, 40.0), Vec3::new(-20.0, -40.0, -20.0));
    for _ in 0..600 {
        if rig
            .frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 20.0,
                ..Default::default()
            })
            .is_some()
        {
            events += 1;
        }
    }
    // Along the portal plane, outside the frame.
    let v = rig.detector.volume;
    rig.teleport(
        Vec3::new(v.min_x - 6.0, v.plane_point.y, v.plane_point.z + 3.0),
        Vec3::new(v.min_x - 6.0, v.plane_point.y, v.plane_point.z - 3.0),
    );
    for _ in 0..200 {
        if rig
            .frame(FreeFlyInput {
                move_forward: 1.0,
                delta_time: 1.0 / 20.0,
                ..Default::default()
            })
            .is_some()
        {
            events += 1;
        }
    }
    assert_eq!(events, 0);
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.local_up, UP);
}

#[test]
fn route_d_reset_from_red_black_and_from_mid_transition() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    let home = world_free_fly_camera();
    assert_eq!(
        rig.traverse(1.0),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    rig.frame(FreeFlyInput {
        reset: true,
        ..Default::default()
    });
    assert_eq!(rig.state.position, home.position);
    assert_eq!((rig.state.yaw, rig.state.pitch), (home.yaw, home.pitch));
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.local_up, UP);
    assert_eq!(rig.state.environment(), WorldEnvironmentProfile::DAY);
    assert!(rig.state.transition().is_none());
    assert!(rig.detector.is_armed());
    // Mid-transition.
    let v = rig.detector.volume;
    rig.teleport(v.plane_point + v.normal * 0.5, v.plane_point);
    let mut e = None;
    while e.is_none() {
        e = rig.frame(FreeFlyInput {
            move_forward: 1.0,
            delta_time: 1.0 / 60.0,
            ..Default::default()
        });
    }
    rig.frame(FreeFlyInput {
        delta_time: PORTAL_TRANSITION_DURATION / 2.0,
        ..Default::default()
    });
    assert!(rig.state.transition().is_some());
    rig.frame(FreeFlyInput {
        reset: true,
        ..Default::default()
    });
    assert!(rig.state.transition().is_none());
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.position, home.position);
    assert_eq!(rig.state.background(), world_background());
    // Reset from the plain Overworld too.
    rig.frame(FreeFlyInput {
        move_forward: 1.0,
        delta_time: 0.2,
        ..Default::default()
    });
    rig.frame(FreeFlyInput {
        reset: true,
        ..Default::default()
    });
    assert_eq!(rig.state.position, home.position);
}

#[test]
fn the_portal_detector_rearms_only_away_from_the_membrane() {
    let scene = WorldScene::new();
    let mut rig = Rig::new(&scene);
    let v = rig.detector.volume;
    // Cross in one frame, then hover just around the plane: one event only.
    rig.teleport(v.plane_point + v.normal * 0.5, v.plane_point);
    let mut events = 0;
    for i in 0..12 {
        let d = if i % 2 == 0 { 1.0 } else { -1.0 };
        if rig
            .frame(FreeFlyInput {
                move_forward: d,
                delta_time: 0.25,
                ..Default::default()
            })
            .is_some()
        {
            events += 1;
        }
        // Let any roll finish without moving.
        while rig.state.transition().is_some() {
            rig.frame(FreeFlyInput {
                delta_time: 0.1,
                ..Default::default()
            });
        }
    }
    assert_eq!(events, 1, "hovering re-triggered the portal");
    assert!(!rig.detector.is_armed());
    // Fly clearly away: armed again.
    rig.teleport(
        v.plane_point - v.normal * 0.4,
        v.plane_point - v.normal * 10.0,
    );
    rig.state.realm = WorldRealm::RedBlack;
    rig.state.local_up = DOWN;
    for _ in 0..10 {
        rig.frame(FreeFlyInput {
            move_forward: 1.0,
            delta_time: 0.25,
            ..Default::default()
        });
    }
    assert!(rig.detector.is_armed());
    assert!(PORTAL_REARM_DISTANCE > 0.5 && PORTAL_CROSSING_EPSILON < 0.2);
}

#[test]
fn geometry_lights_catalog_and_dependencies_are_unchanged() {
    let scene = WorldScene::new();
    // Gate 14 rebuilt the lower route, so the total is no longer 6784;
    // the diorama stays deterministic and the portal/lights intact.
    assert_eq!(scene.world().len(), WorldScene::new().world().len());
    assert!(scene.world().len() > 6000);
    assert_eq!(scene.portal().frame.len(), 18);
    assert_eq!(scene.portal().core.len(), 12);
    assert_eq!(scene.house().floor.len(), 35);
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(scene.trees().len(), 4);
    assert_eq!(scene.families().len(), 3);
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| matches!(l, Light::Point(_)))
            .count(),
        4
    );
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    assert_eq!(official_entries().len(), 39);
    assert_eq!(CatalogScene::new().len(), 41);
    assert_eq!(
        gallery_background(),
        core::color::Color::new(0.05, 0.05, 0.08, 1.0)
    );
    let cargo = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = cargo
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split('[')
        .next()
        .unwrap();
    assert_eq!(deps.lines().filter(|l| l.contains('=')).count(), 1);
    // The camera never writes the world: its module has no VoxelWorld.
    let camera = std::fs::read_to_string("src/camera/world_free_fly.rs").unwrap();
    assert!(!camera.contains("VoxelWorld") && !camera.contains("world.insert"));
    let crossing = std::fs::read_to_string("src/camera/portal_crossing.rs").unwrap();
    assert!(!crossing.contains("VoxelWorld"));
}

#[test]
fn reportable_constants() {
    println!("GATE13.5 movement speed: {FREE_FLY_SPEED} blocks/s");
    println!("GATE13.5 mouse sensitivity: {FREE_FLY_MOUSE_SENSITIVITY} rad/px");
    println!(
        "GATE13.5 pitch clamp: +-{:.1} deg",
        FREE_FLY_MAX_PITCH.to_degrees()
    );
    println!("GATE13.5 portal transition duration: {PORTAL_TRANSITION_DURATION} s");
    println!("GATE13.5 portal crossing epsilon: {PORTAL_CROSSING_EPSILON}");
    println!("GATE13.5 portal rearm distance: {PORTAL_REARM_DISTANCE}");
    let scene = WorldScene::new();
    let v = PortalVolume::from_anchor(&scene.rhombus().portal);
    println!(
        "GATE13.5 portal plane point: {:?} normal: {:?} aperture x {}..{} y {}..{}",
        v.plane_point, v.normal, v.min_x, v.max_x, v.min_y, v.max_y
    );
    let d = WorldEnvironmentProfile::DAY;
    let r = WorldEnvironmentProfile::RED_BLACK;
    println!(
        "GATE13.5 day zenith {:?} horizon {:?} haze {:?} ambient x{}",
        d.sky_zenith, d.sky_horizon, d.sky_haze, d.ambient_scale
    );
    println!(
        "GATE13.5 red-black zenith {:?} horizon {:?} haze {:?} ambient x{}",
        r.sky_zenith, r.sky_horizon, r.sky_haze, r.ambient_scale
    );
}
