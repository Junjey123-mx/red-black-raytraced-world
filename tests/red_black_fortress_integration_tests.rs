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

use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm, angles_for};
use core::math::{IVec3, Vec3};
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::preview::AdaptivePreview;
use renderer::raytracer::{RenderQuality, VoxelScene};
use scene::block_type::BlockType;
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
use scene::light::Light;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::time::Instant;

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

/// K: the expanded lower world from below and south (Gate 15 pose).
pub fn pose_k() -> WorldFreeFlyCameraState {
    red_black(v(30.5, -22.0, 20.0), v(30.5, -12.5, 8.5))
}

/// M: the fortress exterior from below the south-west.
pub fn pose_m_exterior() -> WorldFreeFlyCameraState {
    red_black(v(16.0, -30.0, 22.0), v(30.5, -16.0, 8.5))
}

/// N: inside the council hall looking at the throne and crest.
pub fn pose_n_interior() -> WorldFreeFlyCameraState {
    red_black(v(31.5, -13.5, 9.5), v(33.5, -14.0, 7.0))
}

fn is_glow(t: BlockType) -> bool {
    is_family_brick(t)
        || is_symbol(t)
        || matches!(
            t,
            BlockType::CryingObsidianCrimson
                | BlockType::CryingObsidianOrange
                | BlockType::CryingObsidianViolet
                | BlockType::BuddingAmethyst
                | BlockType::AmethystCluster
        )
}

/// The complete inverted route from the Gate 14 landing to the fortress
/// upper room, then each tower core.
fn grand_route(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    points.push(eye(scene.expansion_layout().red_black_path_anchor));
    for c in &scene.red_black_approach().main {
        points.push(eye(*c));
    }
    let l = scene.fortress_layout();
    for c in l.main_route() {
        points.push(eye(*c));
    }
    points
}

#[test]
fn accents_are_present_and_restrained() {
    let scene = WorldScene::new();
    let c = scene.fortress_composition();
    assert_eq!(c.finials.len(), 3, "{}", c.finials.len());
    assert!(
        c.parapet.len() >= 4 && c.parapet.len() <= 10,
        "{}",
        c.parapet.len()
    );
    assert_eq!(c.crystal.len(), 2);
    assert_eq!(c.patches.len(), 4);
    let l = scene.fortress_layout();
    for cell in &c.finials {
        let t = scene.world().get(*cell).unwrap().block_type();
        assert!(matches!(
            t,
            BlockType::CryingObsidianCrimson
                | BlockType::CryingObsidianOrange
                | BlockType::CryingObsidianViolet
        ));
        assert!(l.is_keep_corner(cell.x, cell.z));
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "finial floats"
        );
    }
    let kinds: Vec<BlockType> = c
        .finials
        .iter()
        .map(|f| scene.world().get(*f).unwrap().block_type())
        .collect();
    assert!(
        kinds.contains(&BlockType::CryingObsidianCrimson)
            && kinds.contains(&BlockType::CryingObsidianViolet)
            && kinds.contains(&BlockType::CryingObsidianOrange),
        "{kinds:?}"
    );
    for cell in &c.parapet {
        assert!(is_family_brick(
            scene.world().get(*cell).unwrap().block_type()
        ));
        assert_eq!(cell.y, l.levels.keep_roof_y - 1);
    }
    let budding = c.crystal[0];
    let cluster = c.crystal[1];
    assert_eq!(
        scene.world().get(budding).unwrap().block_type(),
        BlockType::BuddingAmethyst
    );
    assert_eq!(
        scene.world().get(cluster).unwrap().block_type(),
        BlockType::AmethystCluster
    );
    assert_eq!(
        cluster,
        IVec3::new(budding.x, budding.y - 1, budding.z),
        "the cluster grows toward -Y"
    );
    assert_eq!(
        scene.world().get(cluster).unwrap().orientation(),
        Orientation::Down
    );
    for (cell, t) in &c.patches {
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *t);
        assert_eq!(cell.y, l.levels.ground_y);
        assert!(l.courtyard.contains(&(cell.x, cell.z)));
    }
    for cell in c.added_cells() {
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down
        );
    }
    println!(
        "GATE17 composition: finials={} parapet={} crystal={} patches={} fortress_voxels={} voxels={}",
        c.finials.len(),
        c.parapet.len(),
        c.crystal.len(),
        c.patches.len(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn emission_is_localized_and_the_body_stays_dark() {
    let scene = WorldScene::new();
    let cells = scene.fortress_cells();
    let glow: Vec<IVec3> = cells
        .iter()
        .copied()
        .filter(|c| is_glow(scene.world().get(*c).unwrap().block_type()))
        .collect();
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    // Dark stone stays the structural majority (the Gate 17.5 balance rule
    // keeps it at 60-70 % once corinto timber joins the body).
    assert!(
        dark * 5 >= cells.len() * 3,
        "dark {dark} of {}",
        cells.len()
    );
    assert!(
        glow.len() * 4 < cells.len(),
        "glow {} of {}",
        glow.len(),
        cells.len()
    );
    // No glowing cell sits in a glowing patch: lines (bands, edges) and a
    // motif touching a line are allowed, a solid glowing wall is not.
    for c in &glow {
        let n = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ]
        .iter()
        .filter(|(dx, dy, dz)| glow.contains(&IVec3::new(c.x + dx, c.y + dy, c.z + dz)))
        .count();
        assert!(n <= 3, "{c:?} has {n} glowing neighbours");
    }
    // Every family glows somewhere.
    for want in [
        BlockType::RedBlackDeepslateBricksCrimson,
        BlockType::RedBlackDeepslateBricksOrange,
        BlockType::RedBlackDeepslateBricksViolet,
        BlockType::CryingObsidianCrimson,
        BlockType::CryingObsidianOrange,
        BlockType::CryingObsidianViolet,
    ] {
        assert!(
            glow.iter()
                .any(|c| scene.world().get(*c).unwrap().block_type() == want),
            "{want:?} missing"
        );
    }
}

#[test]
fn no_point_light_is_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    let src = std::fs::read_to_string("src/scene/fortress.rs").unwrap();
    assert!(!src.contains("PointLight") && !src.contains("Light::") && !src.contains("emission"));
    let world = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(world.matches("PointLight::new(").count(), 0);
}

#[test]
fn the_grand_route_runs_both_ways_without_noclip() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = grand_route(&scene);
    walk_both_ways(&scene, &route);
    // Each tower from its courtyard cell and back.
    let l = scene.fortress_layout();
    for tower_route in &l.interior_routes[1..] {
        let pts: Vec<Vec3> = tower_route.iter().map(|c| eye(*c)).collect();
        walk_both_ways(&scene, &pts);
        // And reached from the gate across the courtyard: east to the
        // courtyard column before the keep, along it, then to the tower.
        let g = l.levels.ground_y;
        let yard_x = l.central_keep.min_x - 1;
        let gate_cell = eye(IVec3::new(l.gatehouse.max_x + 1, g, l.gate.base.z));
        let outside = pts[0];
        let turn = eye(IVec3::new(yard_x, g, l.gate.base.z));
        let via = eye(IVec3::new(yard_x, g, tower_route[0].z));
        walk_both_ways(&scene, &[gate_cell, turn, via, outside]);
    }
    println!("GATE17 grand route: {} eye points", route.len());
}

#[test]
fn the_courtyard_and_patches_keep_the_routes_clear() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = l.levels.ground_y;
    for (x, z) in &l.courtyard {
        assert!(clear(&scene, eye(IVec3::new(*x, g, *z))), "({x},{z})");
    }
    for cell in scene.fortress_composition().added_cells() {
        assert!(
            !l.courtyard.contains(&(cell.x, cell.z)),
            "{cell:?} in the courtyard"
        );
        assert!(
            !l.interior_routes
                .iter()
                .flatten()
                .any(|r| r.x == cell.x && r.z == cell.z && cell.y <= r.y - 1 && cell.y >= r.y - 2),
            "{cell:?} on a route"
        );
    }
}

#[test]
fn the_lower_viewpoints_keep_interactive_performance() {
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
    let _ = world_free_fly_camera();
    for (name, state) in [
        ("K_expanded_red_black", pose_k()),
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
        assert!(fps >= 15.0, "{name}: {fps:.1} FPS");
    }
}
