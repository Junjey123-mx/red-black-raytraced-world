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
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::fortress::{FORTRESS_WALL_COURSES, is_dark_structure};
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

#[test]
fn the_foundation_replaces_the_pad_and_is_connected() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let f = scene.fortress_foundation();
    let g = l.levels.ground_y;
    let pad = l.footprint;
    assert_eq!(
        f.footings.len() + f.floors.len(),
        (pad.width() * pad.depth()) as usize
    );
    for cell in f.footings.iter().chain(f.floors.iter()) {
        assert_eq!(cell.y, g);
        assert!(pad.contains(cell.x, cell.z));
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        // Dark, or the organic core floor a family tower lays later.
        assert!(
            is_dark_structure(b.block_type())
                || matches!(
                    b.block_type(),
                    BlockType::NetherWartBlock | BlockType::Mycelium
                ),
            "{cell:?}"
        );
        // Each pad column is a Gate 15 lower-shell column (the one connected
        // world is certified by the closure tests).
        assert!(
            scene.red_black_expansion().column(cell.x, cell.z).is_some(),
            "{cell:?}"
        );
    }
    for cell in &f.footings {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks
        );
    }
    println!(
        "GATE17 foundation: footings={} floors={} walls={} gate_opening={} fortress_voxels={} voxels={}",
        f.footings.len(),
        f.floors.len(),
        f.walls.len(),
        f.gate_opening.len(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_courtyard_is_paved_and_clear() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = l.levels.ground_y;
    for (x, z) in l.courtyard.iter().chain(l.gate_corridor().iter()) {
        let ground = IVec3::new(*x, g, *z);
        // Basalt paving, or the blackstone threshold / organic patches later
        // stages lay over it.
        let t = scene.world().get(ground).unwrap().block_type();
        assert!(
            matches!(
                t,
                BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::Mycelium
                    | BlockType::NetherWartBlock
            ),
            "({x},{z}) is {t:?}"
        );
        // Head room functionally above the floor: the two cells below in world y.
        for dy in 1..=2 {
            let open = scene
                .world()
                .get(IVec3::new(*x, g - dy, *z))
                .is_none_or(|b| !is_camera_solid(b));
            assert!(open, "({x},{z}) - {dy}");
        }
        assert!(clear(&scene, eye(ground)), "({x},{z})");
    }
}

#[test]
fn the_curtain_is_anchored_and_hollow() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let f = scene.fortress_foundation();
    let pad = l.footprint;
    let perimeter = (2 * (pad.width() + pad.depth()) - 4) as usize;
    assert_eq!(
        f.walls.len(),
        perimeter * FORTRESS_WALL_COURSES as usize - f.gate_opening.len()
    );
    let mut basalt = 0;
    for cell in &f.walls {
        assert!(l.is_curtain_column(cell.x, cell.z));
        assert!(cell.y <= l.levels.base_y && cell.y >= l.levels.wall_top_y);
        let b = scene.world().get(*cell).unwrap();
        assert!(is_camera_solid(b));
        assert_eq!(b.orientation(), Orientation::Down);
        if b.block_type() == BlockType::SmoothBasalt {
            basalt += 1;
        }
        // Anchored toward the ground (world +y), through the opening's lintel too.
        let support = IVec3::new(cell.x, cell.y + 1, cell.z);
        assert!(
            scene.world().contains(support) || f.gate_opening.contains(&support),
            "{cell:?} floats"
        );
    }
    assert!(
        basalt > f.walls.len() / 6 && basalt < f.walls.len() / 2,
        "{basalt}"
    );
    for (x, z) in &l.courtyard {
        for y in l.levels.wall_top_y..=l.levels.base_y {
            let open = scene
                .world()
                .get(IVec3::new(*x, y, *z))
                .is_none_or(|b| !is_camera_solid(b));
            assert!(open, "({x},{y},{z}) fills the courtyard");
        }
    }
}

#[test]
fn the_gate_opening_is_valid() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let f = scene.fortress_foundation();
    assert_eq!(f.gate_opening, l.gate.cells());
    for cell in &f.gate_opening {
        let open = scene.world().get(*cell).is_none_or(|b| !is_camera_solid(b));
        assert!(open, "{cell:?} blocked");
    }
    let g = l.gate;
    // Lintel functionally above (world y - 2), jambs beside.
    let lintel = IVec3::new(g.base.x, g.base.y - 2, g.base.z);
    assert!(
        scene
            .world()
            .get(lintel)
            .is_some_and(|b| is_camera_solid(b))
    );
    for j in [
        IVec3::new(g.base.x, g.base.y, g.base.z - 1),
        IVec3::new(g.base.x, g.base.y, g.base.z + g.width),
    ] {
        assert!(
            scene.world().get(j).is_some_and(|b| is_camera_solid(b)),
            "{j:?}"
        );
    }
    assert!(clear(
        &scene,
        v(
            g.base.x as f32 + 0.5,
            g.base.y as f32 - 0.5,
            g.base.z as f32 + 1.0
        )
    ));
}

#[test]
fn everything_is_down_oriented() {
    let scene = WorldScene::new();
    for cell in scene.fortress_foundation().cells() {
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down,
            "{cell:?}"
        );
    }
    for cell in scene.fortress_cells() {
        assert!(cell.y < scene.fortress_layout().levels.ground_y);
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down
        );
    }
}

#[test]
fn the_approach_is_unobstructed() {
    let scene = WorldScene::new();
    let a = scene.red_black_approach();
    for cell in &a.main {
        assert!(clear(&scene, eye(*cell)), "{cell:?}");
    }
    let l = scene.fortress_layout();
    for cell in scene.fortress_foundation().cells() {
        assert!(l.footprint.contains(cell.x, cell.z));
    }
    // From the approach through the gate into the courtyard and back.
    let mut points: Vec<Vec3> = a.main.iter().map(|c| eye(*c)).collect();
    let end = l.central_keep.min_x;
    points.extend(
        l.main_route()
            .iter()
            .take_while(|r| r.x < end)
            .map(|r| eye(*r)),
    );
    walk_both_ways(&scene, &points);
}
