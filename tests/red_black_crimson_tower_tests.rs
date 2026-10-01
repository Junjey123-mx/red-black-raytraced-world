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
use scene::fortress::{is_dark_structure, is_family_brick};
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
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

fn count(scene: &WorldScene, cells: &[IVec3], t: BlockType) -> usize {
    cells
        .iter()
        .filter(|c| scene.world().get(**c).unwrap().block_type() == t)
        .count()
}

#[test]
fn hearts_and_diamonds_are_heraldic_focal_blocks() {
    let scene = WorldScene::new();
    let t = scene
        .fortress_tower(Family::Crimson)
        .expect("the Crimson tower");
    let cells = t.cells();
    let hearts = count(&scene, &cells, BlockType::CrimsonHeart);
    let diamonds = count(&scene, &cells, BlockType::CrimsonDiamond);
    assert!(
        hearts >= 1 && diamonds >= 1,
        "hearts {hearts} diamonds {diamonds}"
    );
    assert!(
        hearts + diamonds <= 6,
        "symbols as wall fill: {}",
        hearts + diamonds
    );
    let l = scene.fortress_layout();
    let f = l.footprint;
    for (cell, bt) in &t.crests {
        assert!(matches!(
            bt,
            BlockType::CrimsonHeart | BlockType::CrimsonDiamond
        ));
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *bt);
        // On an outward face, surrounded by dark structure or band, never
        // next to another symbol.
        assert!(
            cell.x == f.min_x || cell.z == f.min_z,
            "{cell:?} is not on an outward face"
        );
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz);
            if let Some(b) = scene.world().get(n) {
                assert!(
                    !matches!(
                        b.block_type(),
                        BlockType::CrimsonHeart | BlockType::CrimsonDiamond
                    ),
                    "{cell:?} touches {n:?}"
                );
            }
        }
    }
    println!(
        "GATE17 crimson tower: shell={} band={} crests={} finials={} extras={} fortress_voxels={} voxels={}",
        t.shell.len(),
        t.band.len(),
        t.crests.len(),
        t.finials.len(),
        t.extras.len(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn crimson_obsidian_and_structural_brick_are_present() {
    let scene = WorldScene::new();
    let t = scene.fortress_tower(Family::Crimson).unwrap();
    let cells = t.cells();
    assert_eq!(count(&scene, &cells, BlockType::CryingObsidianCrimson), 4);
    for cell in &t.finials {
        assert_eq!(cell.y, scene.fortress_layout().levels.max_y);
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "finial floats"
        );
    }
    assert!(count(&scene, &cells, BlockType::RedBlackDeepslateBricksCrimson) >= 8);
    for cell in &t.band {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::RedBlackDeepslateBricksCrimson
        );
    }
    // Every coloured cell of the tower is Crimson-family: no other family leaks in.
    for cell in &cells {
        let bt = scene.world().get(*cell).unwrap().block_type();
        if is_family_brick(bt) {
            assert_eq!(bt, BlockType::RedBlackDeepslateBricksCrimson, "{cell:?}");
        }
    }
}

#[test]
fn nether_wart_is_used_with_restraint() {
    let scene = WorldScene::new();
    let t = scene.fortress_tower(Family::Crimson).unwrap();
    let wart: Vec<IVec3> = t
        .extras
        .iter()
        .filter(|(_, bt)| *bt == BlockType::NetherWartBlock)
        .map(|(c, _)| *c)
        .collect();
    assert!(!wart.is_empty() && wart.len() <= 4, "{}", wart.len());
    for cell in &wart {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::NetherWartBlock
        );
    }
    // The core floor is wart: organic ground inside the tower.
    let (cx, cz) = t.core;
    assert!(wart.contains(&IVec3::new(cx, scene.fortress_layout().levels.ground_y, cz)));
    // Dark structure still dominates the tower.
    let cells = t.cells();
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(dark * 2 > cells.len(), "dark {dark} of {}", cells.len());
}

#[test]
fn the_tower_is_hollow_down_oriented_and_navigable() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t = scene.fortress_tower(Family::Crimson).unwrap();
    let (cx, cz) = t.core;
    for y in (l.levels.tower_top_y..=l.levels.base_y).rev() {
        assert!(
            !scene.world().contains(IVec3::new(cx, y, cz)),
            "({cx},{y},{cz}) fills the core"
        );
    }
    for cell in t.cells() {
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down,
            "{cell:?}"
        );
        assert!(l.crimson_tower.bounds.contains(cell.x, cell.z));
    }
    for cell in &t.entrance {
        assert!(
            !scene.world().contains(*cell),
            "{cell:?} blocks the entrance"
        );
    }
    // Courtyard -> entrance -> core, and back, with collision on.
    let route: Vec<Vec3> = l.interior_routes[1].iter().map(|c| eye(*c)).collect();
    assert_eq!(l.interior_routes[1].last().unwrap().x, cx);
    walk_both_ways(&scene, &route);
    assert!(clear(&scene, eye(IVec3::new(cx, l.levels.ground_y, cz))));
}
