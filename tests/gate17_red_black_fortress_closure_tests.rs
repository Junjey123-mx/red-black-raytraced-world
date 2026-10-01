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
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
use scene::light::Light;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::time::Instant;

/// Voxel count certified at the end of Gate 16 (HEAD `0729365`).
const GATE_16_VOXELS: usize = 8673;
/// Cells the Gate 16 castle adds (certified).
const GATE_16_CASTLE: usize = 440;
/// Soft architectural budget of the gate.
const FORTRESS_BUDGET_MAX: usize = 1800;
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

fn pose_l_exterior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(18.0, 13.0, 24.0), v(30.5, 7.0, 8.5))
}

fn pose_l2_interior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(31.5, 6.0, 8.0), v(34.5, 6.0, 9.5))
}

fn pose_k() -> WorldFreeFlyCameraState {
    red_black(v(30.5, -22.0, 20.0), v(30.5, -12.5, 8.5))
}

fn pose_m_exterior() -> WorldFreeFlyCameraState {
    red_black(v(16.0, -30.0, 22.0), v(30.5, -16.0, 8.5))
}

fn pose_n_interior() -> WorldFreeFlyCameraState {
    red_black(v(31.5, -13.5, 9.5), v(33.5, -14.0, 7.0))
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

/// Eye points from the Gate 14 landing through the approach, gate,
/// courtyard, keep and up to the upper room.
fn fortress_route(scene: &WorldScene) -> Vec<Vec3> {
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

/// Block counts of the fortress: its cells above the pad plus the pad row
/// itself (floors, core floors and patches replace that row).
fn fortress_counts(scene: &WorldScene) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let l = scene.fortress_layout();
    let pad_row = (l.footprint.min_z..=l.footprint.max_z).flat_map(|z| {
        (l.footprint.min_x..=l.footprint.max_x).map(move |x| IVec3::new(x, l.levels.ground_y, z))
    });
    for cell in scene.fortress_cells().into_iter().chain(pad_row) {
        let t = scene.world().get(cell).unwrap().block_type();
        *counts.entry(format!("{t:?}")).or_insert(0) += 1;
    }
    counts
}

// 1: one connected world; the fortress hangs from the lower shell.
#[test]
fn the_fortress_is_connected_to_the_lower_world() {
    let scene = WorldScene::new();
    let cells: HashSet<IVec3> = scene.world().iter().map(|(c, _)| *c).collect();
    let start = *scene.house().floor.first().unwrap();
    let mut seen = HashSet::from([start]);
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
    let fortress = scene.fortress_cells();
    assert!(!fortress.is_empty());
    let l = scene.fortress_layout();
    for c in &fortress {
        assert!(seen.contains(c));
        assert!(l.footprint.contains(c.x, c.z), "{c:?} off the pad");
    }
    let bounds = scene.world().bounds().unwrap();
    assert_eq!(bounds.min, IVec3::new(0, -22, 0));
    assert_eq!(bounds.max_exclusive, IVec3::new(37, 13, 24));
    let deepest = fortress.iter().map(|c| c.y).min().unwrap();
    assert_eq!(deepest, l.levels.max_y);
    assert!(deepest > scene.rhombus().lower_tip_y);
    println!(
        "GATE17 fortress: cells={} deepest_y={deepest} bounds={bounds:?} footprint={:?}",
        fortress.len(),
        l.footprint
    );
}

// 2: functional Down orientation everywhere.
#[test]
fn every_fortress_block_faces_the_inverted_up() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    for cell in scene.fortress_cells() {
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down, "{cell:?}");
        assert!(cell.y < l.levels.ground_y);
    }
    for z in l.footprint.min_z..=l.footprint.max_z {
        for x in l.footprint.min_x..=l.footprint.max_x {
            let g = scene
                .world()
                .get(IVec3::new(x, l.levels.ground_y, z))
                .unwrap();
            assert_eq!(g.orientation(), Orientation::Down);
        }
    }
    // Stairs climb north (Down stairs are raised to the north) and the
    // amethyst clusters hang toward -Y from their budding blocks.
    for (tread, o) in &scene.fortress_hall().treads {
        assert_eq!(*o, Orientation::Down);
        assert_eq!(
            scene.world().get(*tread).unwrap().block_type(),
            BlockType::WoodStairs
        );
    }
    let t = &scene.fortress_hall().treads;
    assert!(t[1].0.z < t[0].0.z && t[1].0.y < t[0].0.y);
    for cell in scene.fortress_cells() {
        if scene.world().get(cell).unwrap().block_type() == BlockType::AmethystCluster {
            let base = IVec3::new(cell.x, cell.y + 1, cell.z);
            assert_eq!(
                scene.world().get(base).unwrap().block_type(),
                BlockType::BuddingAmethyst,
                "{cell:?}"
            );
        }
    }
}

// 3-6: towers, keep, symbols and coloured bricks are all present.
#[test]
fn towers_keep_symbols_and_glowing_bricks_are_complete() {
    let scene = WorldScene::new();
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        let t = scene
            .fortress_tower(f)
            .unwrap_or_else(|| panic!("{f:?} tower"));
        assert_eq!(t.band.len(), 8);
        assert_eq!(t.crests.len(), 4);
        assert_eq!(t.finials.len(), 4);
    }
    let k = scene.fortress_keep();
    assert!(k.shell.len() >= 60 && k.roof.len() == 16 && k.symbols().len() == 6);
    let counts = fortress_counts(&scene);
    for want in [
        "CrimsonHeart",
        "CrimsonDiamond",
        "OrangeClub",
        "OrangeSpade",
        "PurpleHeart",
        "PurpleDiamond",
        "PurpleClub",
        "PurpleSpade",
        "RedBlackDeepslateBricksCrimson",
        "RedBlackDeepslateBricksOrange",
        "RedBlackDeepslateBricksViolet",
        "CryingObsidianCrimson",
        "CryingObsidianOrange",
        "CryingObsidianViolet",
        "BuddingAmethyst",
        "AmethystCluster",
        "Mycelium",
        "NetherWartBlock",
        "PolishedBlackstoneBricks",
        "SmoothBasalt",
    ] {
        assert!(
            counts.get(want).copied().unwrap_or(0) >= 1,
            "{want} missing: {counts:?}"
        );
    }
    let cells = scene.fortress_cells();
    let symbols = cells
        .iter()
        .filter(|c| is_symbol(scene.world().get(**c).unwrap().block_type()))
        .count();
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    let coloured = cells
        .iter()
        .filter(|c| is_family_brick(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(symbols <= 24, "symbols as wall spam: {symbols}");
    assert!(dark * 3 > cells.len() * 2, "dark {dark} of {}", cells.len());
    assert!(coloured >= 40);
    for (k, n) in &counts {
        println!("GATE17 material {k}={n}");
    }
    let l = scene.fortress_layout();
    println!(
        "GATE17 geometry: towers=3x{}x{} keep={}x{}x{} hall={}x{} courtyard={} gate_width={} rooms=2 symbols={symbols} coloured={coloured} dark={dark}",
        l.crimson_tower.bounds.width(),
        l.levels.ground_y - l.levels.max_y,
        l.central_keep.width(),
        l.central_keep.depth(),
        l.levels.ground_y - l.levels.keep_roof_y,
        l.hall.width(),
        l.hall.depth(),
        l.courtyard.len(),
        l.gate.width
    );
}

// 7-9: gate, interior and towers reachable, both ways, no noclip.
#[test]
fn the_fortress_route_needs_no_noclip() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = fortress_route(&scene);
    walk_both_ways(&scene, &route);
    let l = scene.fortress_layout();
    for tower_route in &l.interior_routes[1..] {
        let pts: Vec<Vec3> = tower_route.iter().map(|c| eye_under(*c)).collect();
        walk_both_ways(&scene, &pts);
    }
    for cell in &scene.fortress_gate().opening {
        assert!(!scene.world().contains(*cell));
    }
    for cell in scene.fortress_cells() {
        assert!(
            is_camera_solid(scene.world().get(cell).unwrap()),
            "{cell:?}"
        );
    }
    println!("GATE17 route: {} eye points", route.len());
}

// 10: the portal route, both directions, through the real detector.
#[test]
fn the_portal_route_is_preserved_both_ways() {
    let mut rig = Rig::new();
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    let e = rig.fly_until(400);
    assert_eq!(e, Some(PortalCrossingEvent::OverworldToRedBlack));
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, v(0.0, -1.0, 0.0));
    assert!(clear(&rig.scene, rig.state.position));
    // Back: turn around north of the membrane and cross again.
    rig.teleport(v(16.5, -7.5, 13.0), v(16.5, -7.5, 16.0));
    let back = rig.fly_until(400);
    assert_eq!(back, Some(PortalCrossingEvent::RedBlackToOverworld));
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    assert_eq!(rig.state.local_up, WorldRealm::Overworld.up());
    let scene = &rig.scene;
    let portal = scene.portal();
    assert_eq!(portal.frame.len(), 18);
    assert_eq!(portal.core.len(), 12);
    assert!(
        portal.core.iter().all(
            |c| scene.world().get(*c).unwrap().block_type() == BlockType::PortalCoreDarkCrimson
        )
    );
    assert_eq!(scene.inverted_route().treads.len(), 10);
    let pad = scene.fortress_layout().footprint;
    assert!(
        scene
            .inverted_route()
            .treads
            .iter()
            .all(|(t, _)| !pad.contains(t.x, t.z))
    );
}

// 11: the Gate 16 castle is untouched.
#[test]
fn the_overworld_castle_is_preserved() {
    let scene = WorldScene::new();
    assert_eq!(scene.castle_voxels(), GATE_16_CASTLE);
    walk_both_ways(&scene, &castle_route(&scene));
    for cell in scene
        .castle_gatehouse()
        .door
        .iter()
        .chain(scene.castle_keep().door.iter())
    {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodDoor);
        assert!(is_camera_passable(b.block_type()));
    }
    assert_eq!(scene.castle_windows().glass.len(), 17);
    assert_eq!(scene.castle_furniture().chairs.len(), 6);
    assert_eq!(scene.castle_stairs().treads.len(), 2);
    assert_eq!(scene.castle_details().lamps.len(), 7);
    // Castle and fortress never share a cell; the castle stays above the pad.
    let castle: HashSet<IVec3> = scene.castle_cells().into_iter().collect();
    for c in scene.fortress_cells() {
        assert!(!castle.contains(&c));
        assert!(c.y < 0);
    }
    assert!(castle.iter().all(|c| c.y > 0));
}

// 12: budget and lighting.
#[test]
fn voxel_and_light_budgets_are_respected() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    let fortress = scene.fortress_voxels();
    assert_eq!(
        n,
        GATE_16_VOXELS + fortress,
        "cells outside the fortress changed"
    );
    assert!(
        fortress > 0 && fortress <= FORTRESS_BUDGET_MAX,
        "{fortress}"
    );
    assert_eq!(fortress, scene.fortress_cells().len());
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    println!("GATE17 voxels: gate16={GATE_16_VOXELS} final={n} delta={fortress}");
}

// 13: Catalog, crates, Raylib 3D, protected code.
#[test]
fn catalog_crates_and_protected_code_are_unchanged() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(official_entries().len(), 44);
    assert_eq!(CatalogScene::new().entries().len(), 46);
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = &manifest[manifest.find("[dependencies]").unwrap()..];
    let deps = &deps[..deps.find("\n[").map_or(deps.len(), |i| i)];
    assert_eq!(
        deps.lines().filter(|l| l.contains('=')).collect::<Vec<_>>(),
        vec!["raylib = \"5.5\""]
    );
    let src = std::fs::read_to_string("src/scene/fortress.rs").unwrap();
    for token in [
        "Camera3D",
        "BeginMode3D",
        "DrawCube",
        "DrawModel",
        "PointLight",
        "raylib::",
        "PortalCore",
    ] {
        assert!(!src.contains(token), "{token} in fortress.rs");
    }
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert_eq!(COLLISION_SUBSTEP, 0.2);
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "0729365",
            "HEAD",
            "--",
            "src/camera",
            "src/renderer",
            "src/scene/castle.rs",
            "src/scene/material_library.rs",
            // Gate 17.5 extends the block registry, the catalog and the
            // door passability contract (certified by its own tests); the
            // rest of the protected code must stay untouched.
            ":(exclude)src/camera/world_collision.rs",
            "src/scene/portal.rs",
            "src/scene/descent.rs",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let diff = String::from_utf8_lossy(&out.stdout);
            assert!(
                diff.trim().is_empty(),
                "protected code changed since Gate 16:\n{diff}"
            );
        }
    }
}

// 14: interactive performance over both expanded worlds.
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
    for (name, state) in [
        ("A_full_diorama_home", world_free_fly_camera()),
        ("K_expanded_red_black", pose_k()),
        ("L_castle_exterior", pose_l_exterior()),
        ("M_fortress_exterior", pose_m_exterior()),
        ("N_fortress_interior", pose_n_interior()),
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
        println!("GATE17 interactive {name} {w}x{h} fps={fps:.1}");
        assert!(fps >= MIN_FPS, "{name}: {fps:.1} FPS");
    }
}

/// Release benchmark of the gate: A, C, F, J, K, L, L2, M, N.
///
/// `cargo test --release --test gate17_red_black_fortress_closure_tests -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_fortress() {
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
        ("K_expanded_red_black", pose_k()),
        ("L_castle_exterior", pose_l_exterior()),
        ("L2_castle_interior", pose_l2_interior()),
        ("M_fortress_exterior", pose_m_exterior()),
        ("N_fortress_interior", pose_n_interior()),
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
        let mut render = Vec::new();
        let mut total = Vec::new();
        for _ in 0..9 {
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
            render.push(a.elapsed().as_secs_f64() * 1e3);
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
            "BENCH {name} interactive resolution={w}x{h} render_ms={:.1} total_ms={t:.1} fps={:.1} full_ms={:.1}",
            p50(&mut render),
            1000.0 / t,
            p50(&mut full)
        );
    }
}
