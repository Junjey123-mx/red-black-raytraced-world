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

use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::fortress::{is_dark_structure, is_family_brick};
use scene::orientation::Orientation;
use scene::world::WorldScene;

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

/// Eye points from the Gate 15 approach, through the gate, into the courtyard.
fn entrance(scene: &WorldScene) -> Vec<Vec3> {
    let l = scene.fortress_layout();
    let mut points: Vec<Vec3> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| eye(*c))
        .collect();
    let end = l.central_keep.min_x;
    points.extend(
        l.main_route()
            .iter()
            .take_while(|r| r.x < end)
            .map(|r| eye(*r)),
    );
    points
}

#[test]
fn the_approach_connects_to_the_gate() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = scene.fortress_gate();
    let last = *scene.red_black_approach().main.last().unwrap();
    assert_eq!(last.x + 1, l.gate.base.x);
    assert!(
        l.gate
            .columns()
            .iter()
            .any(|(_, z)| (z - last.z).abs() <= 1)
    );
    assert_eq!(g.opening, l.gate.cells());
    assert_eq!(g.opening.len(), 4);
    assert!(CollisionState::new().collision_enabled());
    walk(&scene, &entrance(&scene));
    println!(
        "GATE17 gate: pillars={} walls={} vault={} accents={} threshold={} fortress_voxels={} voxels={}",
        g.pillars.len(),
        g.walls.len(),
        g.vault.len(),
        g.accents.len(),
        g.threshold.len(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_corridor_is_clear_under_its_vault() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = scene.fortress_gate();
    let ground = l.levels.ground_y;
    for (x, z) in l.gate_corridor() {
        for dy in 1..=2 {
            let cell = IVec3::new(x, ground - dy, z);
            assert!(
                !scene.world().contains(cell),
                "{cell:?} blocks the corridor"
            );
        }
        assert!(clear(&scene, eye(IVec3::new(x, ground, z))), "({x},{z})");
    }
    for cell in &g.opening {
        assert!(!scene.world().contains(*cell), "{cell:?} is not open");
    }
    assert_eq!(
        g.vault.len(),
        (l.gatehouse.width() * l.gatehouse.depth()) as usize
    );
    for cell in &g.vault {
        assert_eq!(cell.y, l.levels.wall_top_y - 1);
        assert!(is_dark_structure(
            scene.world().get(*cell).unwrap().block_type()
        ));
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z))
                || l.gate_corridor().contains(&(cell.x, cell.z)),
            "{cell:?}"
        );
    }
}

#[test]
fn the_frame_is_solid_dark_architecture_with_colored_accents() {
    let scene = WorldScene::new();
    let g = scene.fortress_gate();
    let l = scene.fortress_layout();
    for cell in g
        .pillars
        .iter()
        .chain(g.walls.iter())
        .chain(g.vault.iter())
        .chain(g.accents.iter())
        .chain(g.threshold.iter())
    {
        let b = scene.world().get(*cell).unwrap();
        assert!(is_camera_solid(b), "{cell:?}");
        assert_eq!(b.orientation(), Orientation::Down);
    }
    for cell in &g.pillars {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks
        );
    }
    assert_eq!(g.accents.len(), 2);
    let kinds: Vec<BlockType> = g
        .accents
        .iter()
        .map(|c| scene.world().get(*c).unwrap().block_type())
        .collect();
    assert!(
        kinds.contains(&BlockType::CryingObsidianCrimson)
            && kinds.contains(&BlockType::CryingObsidianViolet),
        "{kinds:?}"
    );
    // The woven jambs and lintel (C197) are kept around the opening.
    let gate = l.gate;
    for cell in [
        IVec3::new(gate.base.x, gate.base.y, gate.base.z - 1),
        IVec3::new(gate.base.x, gate.base.y, gate.base.z + gate.width),
        IVec3::new(gate.base.x, gate.base.y - 2, gate.base.z),
    ] {
        assert!(
            is_family_brick(scene.world().get(cell).unwrap().block_type()),
            "{cell:?}"
        );
    }
    assert_eq!(g.threshold.len(), 2);
}

#[test]
fn the_courtyard_is_reachable_and_the_route_reverses() {
    let scene = WorldScene::new();
    let points = entrance(&scene);
    walk_both_ways(&scene, &points);
    let l = scene.fortress_layout();
    let g = l.levels.ground_y;
    let last = *points.last().unwrap();
    for (x, z) in &l.courtyard {
        let target = eye(IVec3::new(*x, g, *z));
        assert!(clear(&scene, target), "({x},{z})");
        let via = eye(IVec3::new(l.gatehouse.max_x + 1, g, *z));
        let via2 = eye(IVec3::new(*x, g, l.gate.base.z));
        let a = [last, via, target];
        let b = [last, via2, target];
        let ok = |pts: &[Vec3]| {
            let mut p = pts[0];
            for t in &pts[1..] {
                p = step(&scene, p, *t);
                if (p - *t).length() >= 1e-3 {
                    return false;
                }
            }
            true
        };
        assert!(ok(&a) || ok(&b), "({x},{z}) unreachable from the gate");
    }
}

#[test]
fn no_portal_semantics_are_reused() {
    let scene = WorldScene::new();
    for cell in scene.fortress_cells() {
        let t = scene.world().get(cell).unwrap().block_type();
        assert!(
            !matches!(
                t,
                BlockType::PortalCoreDarkCrimson | BlockType::PortalFrameRedObsidian
            ),
            "{cell:?} is a portal block"
        );
        assert!(t != BlockType::WoodDoor || is_camera_passable(t));
    }
    for cell in &scene.fortress_gate().opening {
        assert!(scene.world().get(*cell).is_none());
    }
    let src = std::fs::read_to_string("src/scene/fortress.rs").unwrap();
    for token in [
        "PortalCore",
        "PortalFrame",
        "portal_crossing",
        "PortalVolume",
        "WoodDoor",
    ] {
        assert!(!src.contains(token), "{token} in fortress.rs");
    }
    // The real portal is untouched.
    let portal = scene.portal();
    assert_eq!(portal.frame.len(), 18);
    assert_eq!(portal.core.len(), 12);
    for c in &portal.core {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PortalCoreDarkCrimson
        );
    }
}
