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
use scene::castle::seat_faces;
use scene::catalog::{CatalogScene, official_entries};
use scene::light::Light;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::time::Instant;

/// Voxel count certified at the end of Gate 15 (HEAD `2fa6c03`).
const GATE_15_VOXELS: usize = 8233;
/// Soft architectural budget of the gate.
const CASTLE_BUDGET_MAX: usize = 1500;
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

/// Castle exterior from the south-west, above the approach.
fn pose_l_exterior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(18.0, 13.0, 24.0), v(30.5, 7.0, 8.5))
}

/// Inside the great hall, looking east across the room.
fn pose_l2_interior() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(v(31.5, 6.0, 8.0), v(34.5, 6.0, 9.5))
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

/// The mandatory route as eye points: path end, Gate 15 approach, forecourt,
/// gatehouse, door, courtyard, keep door, great hall, stairs, upper level.
fn full_route(scene: &WorldScene) -> Vec<Vec3> {
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

fn castle_counts(scene: &WorldScene) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for cell in scene.castle_cells() {
        let t = scene.world().get(cell).unwrap().block_type();
        *counts.entry(format!("{t:?}")).or_insert(0) += 1;
    }
    counts
}

// 1: the castle stands on the terrain as part of the one connected world.
#[test]
fn the_castle_is_connected_to_the_terrain() {
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
    let castle = scene.castle_cells();
    assert!(!castle.is_empty());
    for c in &castle {
        assert!(seen.contains(c));
    }
    // Every castle ground cell rests on Gate 15 strata; the box is unchanged.
    let pad = scene.castle_layout().footprint;
    let g = scene.castle_layout().levels.ground_y;
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            assert!(scene.world().contains(IVec3::new(x, g, z)));
            assert!(scene.world().contains(IVec3::new(x, g - 1, z)));
        }
    }
    let bounds = scene.world().bounds().unwrap();
    assert_eq!(bounds.min, IVec3::new(0, -22, 0));
    assert_eq!(bounds.max_exclusive, IVec3::new(37, 13, 24));
    let top = castle.iter().map(|c| c.y).max().unwrap();
    println!(
        "GATE16 castle: cells={} top_y={top} bounds={:?} footprint={:?}",
        castle.len(),
        bounds,
        pad
    );
}

// 2-7: the whole mandatory route with collision on, forward and back.
#[test]
fn the_full_castle_route_needs_no_noclip() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = full_route(&scene);
    walk_both_ways(&scene, &route);
    let c = scene.castle_layout();
    // Gate reached, door passed, courtyard, keep door, hall, stairs, upper.
    let cols: Vec<(i32, i32)> = c.interior_route.iter().map(|r| (r.x, r.z)).collect();
    assert!(
        c.gate
            .columns()
            .iter()
            .all(|g| cols.contains(g) || cols.contains(&(g.0, g.1 - 1)))
    );
    assert!(cols.contains(&(c.keep_door.base.x, c.keep_door.base.z)));
    assert!(c.courtyard.iter().any(|col| cols.contains(col)));
    for (tread, _) in &scene.castle_stairs().treads {
        assert!(c.interior_route.contains(tread));
    }
    assert_eq!(c.interior_route.last().unwrap().y, c.levels.upper_floor_y);
    println!("GATE16 route: {} eye points", route.len());
}

// 3, 4: doors pass, walls and glass block.
#[test]
fn doors_pass_and_walls_glass_fences_block() {
    let scene = WorldScene::new();
    for cell in scene
        .castle_gatehouse()
        .door
        .iter()
        .chain(scene.castle_keep().door.iter())
    {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodDoor);
        assert!(is_camera_passable(b.block_type()) && !is_camera_solid(b));
    }
    let mut kinds = HashSet::new();
    for cell in scene.castle_cells() {
        let b = scene.world().get(cell).unwrap();
        kinds.insert(b.block_type());
        if b.block_type() != BlockType::WoodDoor {
            assert!(is_camera_solid(b), "{cell:?} {:?}", b.block_type());
        }
    }
    for wanted in [
        BlockType::Stone,
        BlockType::Cobblestone,
        BlockType::DeepslateBricks,
        BlockType::Log,
        BlockType::WoodPlanks,
        BlockType::Glass,
        BlockType::Fence,
        BlockType::WoodStairs,
        BlockType::DoubleWoodSlab,
        BlockType::RedstoneLampLit,
        BlockType::WoodDoor,
    ] {
        assert!(
            kinds.contains(&wanted),
            "{wanted:?} missing from the castle"
        );
    }
    // Nothing outside the certified list.
    for k in &kinds {
        assert!(
            matches!(
                k,
                BlockType::Stone
                    | BlockType::Cobblestone
                    | BlockType::DeepslateBricks
                    | BlockType::Log
                    | BlockType::WoodPlanks
                    | BlockType::Glass
                    | BlockType::Fence
                    | BlockType::WoodStairs
                    | BlockType::DoubleWoodSlab
                    | BlockType::RedstoneLampLit
                    | BlockType::WoodDoor
            ),
            "{k:?} in the castle"
        );
    }
    // Glass: a ray-visible window that still stops the camera.
    let g = scene.castle_windows().glass[0];
    assert!(!clear(
        &scene,
        v(g.x as f32 + 0.5, g.y as f32 + 0.5, g.z as f32 + 0.5)
    ));
}

// 8: the architecture is complete.
#[test]
fn the_architecture_is_complete() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    assert!(!scene.castle_foundation().footings.is_empty());
    assert!(scene.castle_foundation().walls.len() >= 100);
    assert!(c.courtyard.len() >= 8);
    assert!(!scene.castle_gatehouse().roof.is_empty() && scene.castle_gatehouse().door.len() == 4);
    assert_eq!(c.towers.len(), 4);
    assert_eq!(scene.castle_towers().cores.len(), 4);
    assert_eq!(scene.castle_towers().tower_merlons.len(), 16);
    assert!(
        scene.castle_towers().wall_merlons.len() + scene.castle_details().keep_merlons.len() >= 8
    );
    assert!(scene.castle_keep().shell.len() >= 60 && scene.castle_keep().roof.len() == 16);
    assert!(scene.castle_windows().glass.len() >= 12);
    assert_eq!(scene.castle_stairs().treads.len(), 2);
    let f = scene.castle_furniture();
    assert!(f.chairs.len() >= 4 && f.tables.len() >= 2 && !f.benches.is_empty());
    for (cell, o) in &f.chairs {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::WoodStairs
        );
        let (dx, dz) = seat_faces(*o);
        let faces = IVec3::new(cell.x + dx, cell.y, cell.z + dz);
        assert!(f.tables.contains(&faces) || f.benches.contains(&faces));
    }
    assert!(scene.castle_details().lamps.len() >= 5);
    // Hollow: the two rooms and four tower cores are mostly air.
    let lv = c.levels;
    for (x, z) in &scene.castle_towers().cores {
        for y in lv.base_y..=lv.tower_top_y {
            assert!(!scene.world().contains(IVec3::new(*x, y, *z)));
        }
    }
    let counts = castle_counts(&scene);
    for (k, n) in &counts {
        println!("GATE16 material {k}={n}");
    }
    println!(
        "GATE16 geometry: towers=4x{}x{} keep={}x{}x{} hall={}x{} courtyard={} gate_width={} windows={} chairs={} treads={} lamps={} floors=2",
        c.towers[0].width(),
        lv.tower_top_y + 1 - lv.ground_y,
        c.keep.width(),
        c.keep.depth(),
        lv.keep_roof_y - lv.ground_y,
        c.hall.width(),
        c.hall.depth(),
        c.courtyard.len(),
        c.gate.width,
        scene.castle_windows().glass.len(),
        f.chairs.len(),
        scene.castle_stairs().treads.len(),
        scene.castle_details().lamps.len()
    );
}

// 9-11: house, pond and portal are preserved.
#[test]
fn house_pond_trees_and_portal_are_preserved() {
    let scene = WorldScene::new();
    let w = scene.world();
    let has = |c: &IVec3, t: BlockType| w.get(*c).map(|b| b.block_type()) == Some(t);
    let house = scene.house_exterior();
    assert!(house.door.iter().all(|c| has(c, BlockType::WoodDoor)));
    assert!(house.glass.iter().all(|c| has(c, BlockType::Glass)));
    assert!(house.fence.iter().all(|c| has(c, BlockType::Fence)));
    assert!(
        house
            .lamps
            .iter()
            .all(|c| has(c, BlockType::RedstoneLampLit))
    );
    assert!(house.roof.iter().all(|(c, t, _)| has(c, *t)));
    assert!(
        scene
            .house()
            .walls
            .iter()
            .all(|c| is_camera_solid(w.get(*c).unwrap()))
    );
    let lower = house.door[0];
    walk_both_ways(
        &scene,
        &[
            eye_over(scene.path().start().unwrap()),
            v(
                lower.x as f32 + 0.5,
                lower.y as f32 + 0.5,
                lower.z as f32 + 1.5,
            ),
            v(
                lower.x as f32 + 0.5,
                lower.y as f32 + 0.5,
                lower.z as f32 + 0.5,
            ),
            v(
                lower.x as f32 + 0.5,
                lower.y as f32 + 0.5,
                lower.z as f32 - 1.5,
            ),
        ],
    );
    let pond = scene.pond();
    assert!(pond.water.iter().all(|c| has(c, BlockType::Water)));
    assert!(pond.sand.iter().all(|c| has(c, BlockType::Sand)));
    let pad = scene.castle_layout().footprint;
    for (x, z) in &pond.basin {
        assert!(!pad.contains(*x, *z));
    }
    // Around the pond: the ring one column out is still walkable at head height.
    for (x, z) in &pond.basin {
        let top = scene.column_top(*x, *z).unwrap();
        assert!(clear(
            &scene,
            v(*x as f32 + 0.5, top as f32 + 2.5, *z as f32 + 0.5)
        ));
    }
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
    let route = scene.inverted_route();
    assert_eq!(route.treads.len(), 10);
    assert!(
        route
            .treads
            .iter()
            .all(|(c, o)| has(c, BlockType::WoodStairs) && *o == Orientation::Down)
    );
    // Red-Black untouched by the castle: no castle cell below the ground.
    let g = scene.castle_layout().levels.ground_y;
    assert!(scene.castle_cells().iter().all(|c| c.y > g));
    let counts: Vec<(usize, usize)> = scene
        .families()
        .iter()
        .map(|f| (f.symbols.len(), f.accents.len()))
        .collect();
    assert_eq!(counts, vec![(25, 12), (21, 6), (23, 10)]);
}

// 12: the portal traversal of Gate 14 still passes end to end.
#[test]
fn the_portal_traversal_still_passes() {
    let mut rig = Rig::new();
    rig.teleport(v(16.5, -7.5, 16.5), v(16.5, -7.5, 14.5));
    let e = rig.fly_until(400);
    assert_eq!(e, Some(PortalCrossingEvent::OverworldToRedBlack));
    assert_eq!(rig.state.realm, WorldRealm::RedBlack);
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
    assert!(rig.state.position.z < 5.0 && deepest < -14.0);
    assert!(clear(&rig.scene, rig.state.position));
    // Canonical route both ways, house to landing.
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
    // The castle never stands between the house and the portal.
    let pad = scene.castle_layout().footprint;
    assert!(
        scene
            .upper_descent()
            .treads
            .iter()
            .all(|(t, _)| !pad.contains(t.x, t.z))
    );
}

// 13: budget.
#[test]
fn the_voxel_budget_is_respected() {
    let scene = WorldScene::new();
    let n = scene.world().len();
    let castle = scene.castle_voxels();
    assert_eq!(
        n,
        GATE_15_VOXELS + castle,
        "cells outside the castle changed"
    );
    assert!(castle > 0 && castle <= CASTLE_BUDGET_MAX, "{castle}");
    assert_eq!(castle, scene.castle_cells().len());
    println!("GATE16 voxels: gate15={GATE_15_VOXELS} final={n} delta={castle}");
}

// 14: Catalog, lights, crates, Raylib 3D.
#[test]
fn catalog_lights_crates_and_raylib_are_unchanged() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    assert_eq!(official_entries().len(), 39);
    assert_eq!(CatalogScene::new().entries().len(), 41);
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
    let castle = std::fs::read_to_string("src/scene/castle.rs").unwrap();
    for token in [
        "Camera3D",
        "BeginMode3D",
        "DrawCube",
        "DrawModel",
        "PointLight",
        "raylib::",
    ] {
        assert!(!castle.contains(token), "{token} in castle.rs");
    }
    assert_eq!(CAMERA_COLLISION_RADIUS, 0.25);
    assert_eq!(COLLISION_SUBSTEP, 0.2);
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            "2fa6c03",
            "HEAD",
            "--",
            "src/camera",
            "src/renderer",
            "src/scene/block_type.rs",
            "src/scene/catalog.rs",
        ])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            let diff = String::from_utf8_lossy(&out.stdout);
            assert!(
                diff.trim().is_empty(),
                "protected code changed since Gate 15:\n{diff}"
            );
        }
    }
}

// 15: interactive performance on the original and castle viewpoints.
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
        assert!(fps >= MIN_FPS, "{name}: {fps:.1} FPS");
    }
}

/// Release benchmark of the gate: A, C, F, J and the two castle viewpoints.
///
/// `cargo test --release --test gate16_overworld_castle_closure_tests -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_castle() {
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
        ("L_castle_exterior", pose_l_exterior()),
        ("L2_castle_interior", pose_l2_interior()),
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
