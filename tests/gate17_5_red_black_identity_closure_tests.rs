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
    CAMERA_COLLISION_RADIUS, COLLISION_SUBSTEP, CameraCollisionConfig, CollisionState,
    is_camera_passable, is_camera_solid, resolve_camera_motion,
};
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, angles_for, apply_free_fly_input,
};
use core::math::{IVec3, Vec3};
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::preview::AdaptivePreview;
use renderer::raytracer::{RenderQuality, VoxelScene};
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
use scene::light::Light;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
use scene::red_black_timber::{RED_BLACK_TIMBER, is_red_black_timber, timber_contract};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::collections::{BTreeMap, HashSet};
use std::time::Instant;

/// Gate 17 final voxel count (HEAD `807e490`).
const GATE_17_VOXELS: usize = 9083;
/// Cells the Gate 16 castle and Gate 17 fortress certified.
const GATE_16_CASTLE: usize = 440;
const GATE_17_FORTRESS: usize = 410;
/// Soft ceiling of the Gate 17.5 identity budget.
const IDENTITY_BUDGET_MAX: usize = 1400;
const MIN_FPS: f64 = 15.0;

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
fn _fortress_counts(scene: &WorldScene) -> BTreeMap<String, usize> {
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

/// O: the fortress approach and gatehouse, from along the path.
fn pose_o_approach() -> WorldFreeFlyCameraState {
    red_black(v(24.6, -17.0, 3.6), v(26.0, -13.6, 8.9))
}

/// P: the Red-Black grove on the north shoulder, the fortress behind it.
fn pose_p_grove() -> WorldFreeFlyCameraState {
    red_black(v(21.0, -25.0, -6.0), v(26.0, -15.0, 4.0))
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

fn identity_counts(scene: &WorldScene) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for (_, b) in scene.world().iter() {
        if is_red_black_timber(b.block_type()) {
            *counts.entry(format!("{:?}", b.block_type())).or_insert(0) += 1;
        }
    }
    counts
}

fn interactive_fps(
    scene: &WorldScene,
    manager: &TextureManager,
    materials: &MaterialLibrary,
    state: WorldFreeFlyCameraState,
    warm: usize,
    runs: usize,
) -> (usize, usize, f64, f64) {
    let lights = world_lights();
    let threads = ParallelRenderConfig::detect();
    let camera = state.camera(4.0 / 3.0);
    let vs = VoxelScene {
        world: scene.world(),
        materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
        background: state.background(),
        texture_manager: manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    let mut preview = AdaptivePreview::new();
    let mut fb = Framebuffer::new(400, 300);
    for _ in 0..warm {
        let (w, h) = preview.resolution(800, 600);
        fb.resize(w, h);
        let a = Instant::now();
        render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
        preview.record(a.elapsed());
    }
    let (w, h) = preview.resolution(800, 600);
    let mut render = Vec::new();
    let mut total = Vec::new();
    let mut rgba = Vec::new();
    for _ in 0..runs {
        fb.resize(w, h);
        let a = Instant::now();
        render_parallel(&mut fb, &camera, &vs, RenderQuality::Interactive, &threads);
        render.push(a.elapsed().as_secs_f64() * 1e3);
        fb.write_rgba8(&mut rgba);
        total.push(a.elapsed().as_secs_f64() * 1e3);
    }
    render.sort_by(|a, b| a.partial_cmp(b).unwrap());
    total.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (w, h, render[render.len() / 2], total[total.len() / 2])
}

fn load() -> (TextureManager, MaterialLibrary) {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    (manager, materials)
}

// 1
#[test]
fn the_catalog_closes_at_forty_four() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(BlockType::ALL.len(), 44);
    assert_eq!(official_entries().len(), 44);
    assert_eq!(CatalogScene::new().entries().len(), 46);
    let unique: HashSet<BlockType> = BlockType::ALL.iter().copied().collect();
    assert_eq!(unique.len(), 44);
    for t in RED_BLACK_TIMBER {
        assert!(BlockType::ALL.contains(&t));
        assert_eq!(t.family(), BlockFamily::RedBlackTimber);
        let c = timber_contract(t).unwrap();
        assert!(!c.emissive);
        for p in &c.texture_paths {
            assert!(std::path::Path::new(p).exists(), "{p}");
        }
    }
}

// 2
#[test]
fn the_five_new_blocks_are_in_the_world() {
    let scene = WorldScene::new();
    let counts = identity_counts(&scene);
    for t in RED_BLACK_TIMBER {
        assert!(
            counts.get(&format!("{t:?}")).copied().unwrap_or(0) > 0,
            "{t:?} unused: {counts:?}"
        );
    }
    for (k, n) in &counts {
        println!("GATE17_5 world block {k}={n}");
    }
    // Only in the lower realm.
    for (c, b) in scene.world().iter() {
        if is_red_black_timber(b.block_type()) {
            assert!(c.y < 0, "{c:?} is in the Overworld");
            assert_eq!(
                b.orientation() == Orientation::West,
                b.block_type() == BlockType::RedBlackWoodDoor,
                "{c:?}: only the door keeps a horizontal facing"
            );
        }
    }
}

// 3
#[test]
fn the_fortress_is_complete_with_four_towers_and_a_gatehouse() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let id = scene.red_black_identity();
    assert_eq!(scene.fortress_towers().len() + 1, 4);
    let mut bounds: Vec<_> = l.towers().iter().map(|t| t.bounds).collect();
    bounds.push(id.fourth_tower.bounds);
    for (i, a) in bounds.iter().enumerate() {
        for b in &bounds[i + 1..] {
            assert!(
                a.max_x < b.min_x || b.max_x < a.min_x || a.max_z < b.min_z || b.max_z < a.min_z
            );
        }
    }
    assert_eq!(id.gatehouse.door.len(), 4);
    for c in &id.gatehouse.door {
        let b = scene.world().get(*c).unwrap();
        assert_eq!(b.block_type(), BlockType::RedBlackWoodDoor);
        assert!(is_camera_passable(b.block_type()));
    }
    assert!(!id.gatehouse.hood.is_empty());
    for f in [Family::Crimson, Family::Orange, Family::Violet] {
        assert!(scene.fortress_tower(f).is_some());
    }
    println!(
        "GATE17_5 towers: {:?} | gate door {:?} | gatehouse block {:?}",
        bounds, id.gatehouse.door, l.gatehouse
    );
}

// 4
#[test]
fn the_path_trees_and_fences_are_in_place() {
    let scene = WorldScene::new();
    let id = scene.red_black_identity();
    assert!(id.path.planks.len() >= 20);
    for c in &id.path.planks {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
    }
    let trees = id.grove.trees.len();
    assert!((4..=7).contains(&trees), "{trees}");
    let fences = id.path.fences.len()
        + id.composition.fences.len()
        + id.timber.rails.len()
        + id.gatehouse.rails.len();
    assert!(fences >= 6, "{fences}");
    let first = id.path.planks.first().unwrap();
    let last = *scene.red_black_approach().main.last().unwrap();
    println!(
        "GATE17_5 path: planks={} from={first:?} to={last:?} width=2 | trees={trees} bases={:?} | fences={fences}",
        id.path.planks.len(),
        id.grove.trees.iter().map(|t| t.base).collect::<Vec<_>>()
    );
}

// 5
#[test]
fn the_whole_lower_route_needs_no_noclip() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = lower_route(&scene);
    walk_both_ways(&scene, &route);
    let l = scene.fortress_layout();
    for r in &l.interior_routes[1..] {
        walk_both_ways(&scene, &r.iter().map(|c| eye_under(*c)).collect::<Vec<_>>());
    }
    let t4 = &scene.red_black_identity().fourth_tower;
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
    println!("GATE17_5 lower route: {} eye points", route.len());
}

// 6
#[test]
fn the_collision_contract_is_exactly_extended() {
    let passable: Vec<BlockType> = BlockType::ALL
        .iter()
        .copied()
        .filter(|t| is_camera_passable(*t))
        .collect();
    assert_eq!(
        passable,
        vec![
            BlockType::WoodDoor,
            BlockType::PortalCoreDarkCrimson,
            BlockType::RedBlackWoodDoor
        ]
    );
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert_eq!(COLLISION_SUBSTEP, 0.2);
    let out = std::process::Command::new("git")
        .args(["diff", "807e490", "HEAD", "--", "src/camera"])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let diff = String::from_utf8_lossy(&out.stdout);
            let changed: Vec<&str> = diff
                .lines()
                .filter(|l| {
                    (l.starts_with('+') || l.starts_with('-'))
                        && !l.starts_with("+++")
                        && !l.starts_with("---")
                })
                .collect();
            assert!(
                changed
                    .iter()
                    .all(|l| !l.contains("fn ") || l.contains("is_camera_passable")),
                "{changed:#?}"
            );
            assert!(diff.is_empty() || diff.contains("RedBlackWoodDoor"));
        }
    }
}

// 7
#[test]
fn the_portal_crosses_both_ways() {
    let mut rig = Rig::new();
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    assert_eq!(
        rig.fly_until(400),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
    assert_eq!(rig.state.local_up, v(0.0, -1.0, 0.0));
    rig.teleport(v(16.5, -7.5, 13.0), v(16.5, -7.5, 16.0));
    assert_eq!(
        rig.fly_until(400),
        Some(PortalCrossingEvent::RedBlackToOverworld)
    );
    assert_eq!(rig.state.realm, WorldRealm::Overworld);
    let scene = &rig.scene;
    assert_eq!(scene.portal().frame.len(), 18);
    assert_eq!(scene.portal().core.len(), 12);
    assert_eq!(scene.inverted_route().treads.len(), 10);
}

// 8
#[test]
fn the_overworld_is_untouched() {
    let scene = WorldScene::new();
    assert_eq!(scene.castle_voxels(), GATE_16_CASTLE);
    assert_eq!(scene.fortress_voxels(), GATE_17_FORTRESS);
    assert_eq!(scene.trees().len(), 4);
    for t in scene.trees() {
        assert!(
            t.trunk
                .iter()
                .all(|c| scene.world().get(*c).unwrap().block_type() == BlockType::Log)
        );
    }
    let id = scene.red_black_identity();
    for c in id
        .added_cells()
        .iter()
        .chain(id.balance.cells().iter())
        .chain(id.path.planks.iter())
    {
        assert!(c.y < 0, "{c:?} touches the Overworld");
    }
    walk_both_ways(&scene, &castle_route(&scene));
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "807e490",
            "HEAD",
            "--",
            "src/scene/overworld.rs",
            "src/scene/castle.rs",
            "src/scene/terrain",
            "src/scene/portal.rs",
            "src/scene/descent.rs",
            "src/renderer",
            "src/scene/fortress.rs",
            "src/scene/expansion.rs",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let diff = String::from_utf8_lossy(&out.stdout);
            assert!(
                diff.trim().is_empty(),
                "protected code changed in Gate 17.5:\n{diff}"
            );
        }
    }
}

// 9
#[test]
fn budgets_lights_and_crates_hold() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    let identity = scene.identity_voxels();
    assert_eq!(n, GATE_17_VOXELS + identity);
    assert!(identity <= IDENTITY_BUDGET_MAX, "{identity}");
    assert_eq!(scene.world().bounds().unwrap().min, IVec3::new(0, -22, 0));
    assert_eq!(
        scene.world().bounds().unwrap().max_exclusive,
        IVec3::new(37, 13, 24)
    );
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = &manifest[manifest.find("[dependencies]").unwrap()..];
    let deps = &deps[..deps.find("\n[").map_or(deps.len(), |i| i)];
    assert_eq!(
        deps.lines().filter(|l| l.contains('=')).collect::<Vec<_>>(),
        vec!["raylib = \"5.5\""]
    );
    for p in [
        "src/scene/red_black_identity.rs",
        "src/scene/red_black_timber.rs",
    ] {
        let src = std::fs::read_to_string(p).unwrap();
        for token in [
            "Camera3D",
            "BeginMode3D",
            "DrawCube",
            "DrawModel",
            "PointLight",
            "raylib::",
            "PortalCore",
        ] {
            assert!(!src.contains(token), "{token} in {p}");
        }
    }
    println!("GATE17_5 voxels: gate17={GATE_17_VOXELS} final={n} identity={identity}");
}

// 10
#[test]
fn the_material_balance_holds() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let id = scene.red_black_identity();
    let mut cells: HashSet<IVec3> = scene.fortress_cells().into_iter().collect();
    for z in l.footprint.min_z..=l.footprint.max_z {
        for x in l.footprint.min_x..=l.footprint.max_x {
            cells.insert(IVec3::new(x, l.levels.ground_y, z));
        }
    }
    cells.extend(id.timber.cells());
    cells.extend(id.gatehouse.added_cells());
    cells.extend(id.gatehouse.corridor_floor.iter().copied());
    cells.extend(id.fourth_tower.cells());
    cells.extend(id.balance.cells());
    let kind = |c: &IVec3| scene.world().get(*c).unwrap().block_type();
    let n = cells.len() as f32;
    let dark = cells.iter().filter(|c| is_dark_structure(kind(c))).count() as f32;
    let timber = cells
        .iter()
        .filter(|c| is_red_black_timber(kind(c)))
        .count() as f32;
    let magic = cells
        .iter()
        .filter(|c| {
            is_family_brick(kind(c))
                || is_symbol(kind(c))
                || format!("{:?}", kind(c)).contains("Crying")
                || format!("{:?}", kind(c)).contains("Amethyst")
        })
        .count() as f32;
    println!(
        "GATE17_5 fortress balance: cells={} dark={:.0}% timber={:.0}% magic={:.0}%",
        cells.len(),
        dark / n * 100.0,
        timber / n * 100.0,
        magic / n * 100.0
    );
    assert!(dark / n >= 0.55 && timber / n >= 0.15 && magic / n <= 0.25);
}

// 11
#[test]
fn interactive_performance_meets_the_floor() {
    let (manager, materials) = load();
    let scene = WorldScene::new();
    for (name, state) in [
        ("A_full_diorama_home", world_free_fly_camera()),
        ("K_expanded_red_black", pose_k()),
        ("M_fortress_exterior", pose_m_exterior()),
        ("N_fortress_interior", pose_n_interior()),
        ("O_fortress_approach", pose_o_approach()),
        ("P_red_black_grove", pose_p_grove()),
    ] {
        let (w, h, _, total) = interactive_fps(&scene, &manager, &materials, state, 8, 5);
        let fps = 1000.0 / total;
        println!("GATE17_5 interactive {name} {w}x{h} fps={fps:.1}");
        assert!(fps >= MIN_FPS, "{name}: {fps:.1} FPS");
    }
}

/// Release benchmark of the gate: A, C, F, J, K, L, L2, M, N, O, P.
///
/// `cargo test --release --test gate17_5_red_black_identity_closure_tests -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_identity() {
    let (manager, materials) = load();
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad_x = (l.overworld_castle_pad.min_x + l.overworld_castle_pad.max_x) as f32 / 2.0;
    let threads = ParallelRenderConfig::detect();
    println!(
        "BENCH workers={} voxels={}",
        threads.workers,
        scene.world().len()
    );
    for (name, state) in [
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
        ("O_fortress_approach", pose_o_approach()),
        ("P_red_black_grove", pose_p_grove()),
    ] {
        let (w, h, render, total) = interactive_fps(&scene, &manager, &materials, state, 12, 9);
        println!(
            "BENCH {name} interactive resolution={w}x{h} render_ms={render:.1} total_ms={total:.1} fps={:.1}",
            1000.0 / total
        );
    }
}
