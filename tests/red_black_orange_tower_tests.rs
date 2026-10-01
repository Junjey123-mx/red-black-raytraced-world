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
fn clubs_and_spades_are_the_heraldry() {
    let scene = WorldScene::new();
    let t = scene
        .fortress_tower(Family::Orange)
        .expect("the Orange tower");
    let cells = t.cells();
    let clubs = count(&scene, &cells, BlockType::OrangeClub);
    let spades = count(&scene, &cells, BlockType::OrangeSpade);
    assert!(clubs >= 1 && spades >= 1, "clubs {clubs} spades {spades}");
    assert!(clubs + spades <= 6);
    let f = scene.fortress_layout().footprint;
    for (cell, bt) in &t.crests {
        assert!(matches!(bt, BlockType::OrangeClub | BlockType::OrangeSpade));
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *bt);
        assert!(
            cell.x == f.max_x || cell.z == f.min_z,
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
                assert!(!matches!(
                    b.block_type(),
                    BlockType::OrangeClub | BlockType::OrangeSpade
                ));
            }
        }
    }
    println!(
        "GATE17 orange tower: shell={} band={} crests={} finials={} extras={} fortress_voxels={} voxels={}",
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
fn orange_obsidian_and_structural_brick_are_present() {
    let scene = WorldScene::new();
    let t = scene.fortress_tower(Family::Orange).unwrap();
    let cells = t.cells();
    assert_eq!(count(&scene, &cells, BlockType::CryingObsidianOrange), 4);
    assert!(count(&scene, &cells, BlockType::RedBlackDeepslateBricksOrange) >= 8);
    for cell in &cells {
        let bt = scene.world().get(*cell).unwrap().block_type();
        if is_family_brick(bt) {
            assert_eq!(bt, BlockType::RedBlackDeepslateBricksOrange, "{cell:?}");
        }
    }
    for cell in &t.finials {
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z))
        );
    }
}

#[test]
fn the_wing_is_heavier_dark_masonry() {
    let scene = WorldScene::new();
    let orange = scene.fortress_tower(Family::Orange).unwrap();
    let crimson = scene.fortress_tower(Family::Crimson).unwrap();
    let cells = orange.cells();
    let basalt = count(&scene, &cells, BlockType::SmoothBasalt);
    let blackstone = count(&scene, &cells, BlockType::PolishedBlackstoneBricks);
    assert!(basalt >= 1 && blackstone >= 1);
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(dark * 3 > cells.len() * 2, "dark {dark} of {}", cells.len());
    // Stonier than Crimson: more basalt in its shell, no organic blocks.
    let crimson_basalt = count(&scene, &crimson.cells(), BlockType::SmoothBasalt);
    assert!(
        basalt > crimson_basalt,
        "orange {basalt} vs crimson {crimson_basalt}"
    );
    assert!(orange.extras.is_empty());
    assert_eq!(count(&scene, &cells, BlockType::NetherWartBlock), 0);
}

#[test]
fn the_tower_is_hollow_down_oriented_and_navigable() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let t = scene.fortress_tower(Family::Orange).unwrap();
    let (cx, cz) = t.core;
    for y in (l.levels.tower_top_y..=l.levels.base_y).rev() {
        assert!(!scene.world().contains(IVec3::new(cx, y, cz)));
    }
    for cell in t.cells() {
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down
        );
        assert!(l.orange_tower.bounds.contains(cell.x, cell.z));
    }
    for cell in &t.entrance {
        assert!(!scene.world().contains(*cell));
    }
    let route: Vec<Vec3> = l.interior_routes[2].iter().map(|c| eye(*c)).collect();
    assert_eq!(l.interior_routes[2].last().unwrap().x, cx);
    walk_both_ways(&scene, &route);
    assert!(clear(&scene, eye(IVec3::new(cx, l.levels.ground_y, cz))));
    // The shared courtyard stays open between the two towers built so far.
    for (x, z) in &l.courtyard {
        assert!(
            clear(&scene, eye(IVec3::new(*x, l.levels.ground_y, *z))),
            "({x},{z})"
        );
    }
}
