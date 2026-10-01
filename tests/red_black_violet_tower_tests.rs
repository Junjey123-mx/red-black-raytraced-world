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
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_shape_factory::block_geometry;
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
fn the_four_purple_suits_are_present_as_heraldry() {
    let scene = WorldScene::new();
    let t = scene
        .fortress_tower(Family::Violet)
        .expect("the Violet tower");
    let cells = t.cells();
    for suit in [
        BlockType::PurpleHeart,
        BlockType::PurpleDiamond,
        BlockType::PurpleClub,
        BlockType::PurpleSpade,
    ] {
        assert_eq!(count(&scene, &cells, suit), 1, "{suit:?}");
    }
    let f = scene.fortress_layout().footprint;
    for (cell, bt) in &t.crests {
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *bt);
        assert!(
            cell.x == f.min_x || cell.z == f.max_z,
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
            if let Some(b) = scene
                .world()
                .get(IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz))
            {
                assert!(
                    !matches!(
                        b.block_type(),
                        BlockType::PurpleHeart
                            | BlockType::PurpleDiamond
                            | BlockType::PurpleClub
                            | BlockType::PurpleSpade
                    ),
                    "{cell:?} touches another suit"
                );
            }
        }
    }
    println!(
        "GATE17 violet tower: shell={} band={} crests={} finials={} extras={} fortress_voxels={} voxels={}",
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
fn violet_obsidian_brick_mycelium_and_amethyst_are_present() {
    let scene = WorldScene::new();
    let t = scene.fortress_tower(Family::Violet).unwrap();
    let cells = t.cells();
    assert_eq!(count(&scene, &cells, BlockType::CryingObsidianViolet), 4);
    assert!(count(&scene, &cells, BlockType::RedBlackDeepslateBricksViolet) >= 8);
    assert_eq!(count(&scene, &cells, BlockType::Mycelium), 1);
    assert_eq!(count(&scene, &cells, BlockType::BuddingAmethyst), 1);
    assert_eq!(count(&scene, &cells, BlockType::AmethystCluster), 1);
    for cell in &cells {
        let bt = scene.world().get(*cell).unwrap().block_type();
        if is_family_brick(bt) {
            assert_eq!(bt, BlockType::RedBlackDeepslateBricksViolet, "{cell:?}");
        }
    }
    // Crystalline but not neon: dark structure still the majority.
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(dark * 2 > cells.len(), "dark {dark} of {}", cells.len());
}

#[test]
fn the_amethyst_grows_toward_the_functional_up() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t = scene.fortress_tower(Family::Violet).unwrap();
    let budding = t
        .extras
        .iter()
        .find(|(_, b)| *b == BlockType::BuddingAmethyst)
        .map(|(c, _)| *c)
        .unwrap();
    let cluster = t
        .extras
        .iter()
        .find(|(_, b)| *b == BlockType::AmethystCluster)
        .map(|(c, _)| *c)
        .unwrap();
    let (cx, cz) = t.core;
    assert_eq!(budding, IVec3::new(cx, l.levels.tower_top_y, cz));
    // The cluster hangs one further toward -Y from its budding block: it
    // grows "up" for the inverted viewer.
    assert_eq!(cluster, IVec3::new(cx, budding.y - 1, cz));
    let b = scene.world().get(cluster).unwrap();
    assert_eq!(b.orientation(), Orientation::Down);
    assert_eq!(
        scene.world().get(budding).unwrap().orientation(),
        Orientation::Down
    );
    // Down geometry: the cluster's prisms rest against the cell's +Y face
    // (the budding block) and point to -Y.
    let geometry = block_geometry(BlockType::AmethystCluster, Orientation::Down);
    let parts = geometry.parts();
    assert!(!parts.is_empty());
    let max_y = parts.iter().map(|p| p.max().y).fold(f32::MIN, f32::max);
    let min_y = parts.iter().map(|p| p.min().y).fold(f32::MAX, f32::min);
    assert!(
        (max_y - 1.0).abs() < 1e-5,
        "the cluster does not touch its +Y base: {max_y}"
    );
    assert!(min_y > 0.0, "the cluster reaches past the cell toward -Y");
    // Mycelium is the core floor.
    assert_eq!(
        scene
            .world()
            .get(IVec3::new(cx, l.levels.ground_y, cz))
            .unwrap()
            .block_type(),
        BlockType::Mycelium
    );
}

#[test]
fn the_tower_is_hollow_down_oriented_and_navigable() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t = scene.fortress_tower(Family::Violet).unwrap();
    let (cx, cz) = t.core;
    // Open from the floor to the budding block that closes the shaft.
    for y in ((l.levels.tower_top_y + 1)..=l.levels.base_y).rev() {
        assert!(!scene.world().contains(IVec3::new(cx, y, cz)));
    }
    for cell in t.cells() {
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down
        );
        assert!(l.violet_tower.bounds.contains(cell.x, cell.z));
    }
    for cell in &t.entrance {
        assert!(!scene.world().contains(*cell));
    }
    let route: Vec<Vec3> = l.interior_routes[3].iter().map(|c| eye(*c)).collect();
    assert_eq!(l.interior_routes[3].last().unwrap().x, cx);
    walk_both_ways(&scene, &route);
    assert!(clear(&scene, eye(IVec3::new(cx, l.levels.ground_y, cz))));
    // Separation: the three towers never share a cell and keep their own colour.
    let families = [Family::Crimson, Family::Orange, Family::Violet];
    for (i, a) in families.iter().enumerate() {
        for b in &families[i + 1..] {
            let ca = scene.fortress_tower(*a).unwrap().cells();
            let cb = scene.fortress_tower(*b).unwrap().cells();
            assert!(
                ca.iter().all(|c| !cb.contains(c)),
                "{a:?} and {b:?} share cells"
            );
        }
    }
}
