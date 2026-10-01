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

use camera::world_collision::{CameraCollisionConfig, is_camera_solid, is_position_clear};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::expansion::{
    FOCAL_CLEARANCE, SCENERY_MAX_AMETHYST, SCENERY_MAX_BASALT_OUTCROPS,
    SCENERY_MAX_CRYING_OBSIDIAN, SCENERY_MAX_LEDGES, SCENERY_MAX_MYCELIUM,
    SCENERY_MAX_STONE_CLUSTERS, SCENERY_MAX_TREES, VOXEL_BUDGET_SOFT_MAX,
};
use scene::light::Light;
use scene::orientation::Orientation;
use scene::world::{WorldScene, world_lights};

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn upper_cells(scene: &WorldScene) -> Vec<IVec3> {
    scene.expansion_scenery().upper.cells()
}

fn lower_cells(scene: &WorldScene) -> Vec<IVec3> {
    let l = &scene.expansion_scenery().lower;
    l.added_cells()
        .into_iter()
        .chain(l.mycelium.iter().copied())
        .collect()
}

#[test]
fn the_castle_pad_remains_open() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.overworld_castle_pad;
    // Gate 16 built the castle on it: the pad row is its foundation.
    pad_is_castle_ground(&scene);
    for c in upper_cells(&scene) {
        assert!(!pad.contains(c.x, c.z), "{c:?} on the pad");
    }
    let s = &scene.expansion_scenery().upper;
    assert!(
        !s.trees.is_empty() && s.trees.len() <= SCENERY_MAX_TREES,
        "{}",
        s.trees.len()
    );
    assert!(s.stones.len() <= 2 * SCENERY_MAX_STONE_CLUSTERS && !s.stones.is_empty());
    assert!(!s.markers.is_empty() && s.markers.len() <= 3);
    assert!(s.ledges.len() <= SCENERY_MAX_LEDGES);
    println!(
        "GATE15 upper scenery: trees={} stones={} markers={} ledges={} cells={}",
        s.trees.len(),
        s.stones.len(),
        s.markers.len(),
        s.ledges.len(),
        s.cells().len()
    );
}

#[test]
fn the_fortress_pad_remains_open() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    // Gate 17 built the fortress on it: the pad row is its foundation.
    pad_is_fortress_ground(&scene);
    for c in lower_cells(&scene) {
        assert!(!pad.contains(c.x, c.z), "{c:?} under the pad");
    }
    let s = &scene.expansion_scenery().lower;
    assert!(!s.basalt.is_empty() && s.basalt.len() <= 2 * SCENERY_MAX_BASALT_OUTCROPS);
    assert!(!s.mycelium.is_empty() && s.mycelium.len() <= SCENERY_MAX_MYCELIUM);
    assert!(s.amethyst.len() >= 2 && s.amethyst.len() <= SCENERY_MAX_AMETHYST + 1);
    assert!(
        !s.crying_obsidian.is_empty() && s.crying_obsidian.len() <= SCENERY_MAX_CRYING_OBSIDIAN
    );
    assert!(
        s.amethyst
            .iter()
            .any(|c| scene.world().get(*c).unwrap().block_type() == BlockType::AmethystCluster)
    );
    for c in s.added_cells().iter().chain(s.mycelium.iter()) {
        assert_eq!(
            scene.world().get(*c).unwrap().orientation(),
            Orientation::Down,
            "{c:?}"
        );
    }
    println!(
        "GATE15 lower scenery: basalt={} mycelium={} amethyst={} obsidian={} added={}",
        s.basalt.len(),
        s.mycelium.len(),
        s.amethyst.len(),
        s.crying_obsidian.len(),
        s.added_cells().len()
    );
}

#[test]
fn both_approach_paths_stay_clear() {
    let scene = WorldScene::new();
    let upper = scene.overworld_approach();
    for c in upper.cells() {
        for dy in 1..=2 {
            assert!(
                !scene.world().contains(IVec3::new(c.x, c.y + dy, c.z)),
                "{c:?} + {dy}"
            );
        }
        assert!(
            clear(
                &scene,
                Vec3::new(c.x as f32 + 0.5, c.y as f32 + 2.0, c.z as f32 + 0.5)
            ),
            "{c:?}"
        );
    }
    let lower = scene.red_black_approach();
    for c in &lower.main {
        for dy in 1..=2 {
            assert!(
                !scene.world().contains(IVec3::new(c.x, c.y - dy, c.z)),
                "{c:?} - {dy}"
            );
        }
        assert!(
            clear(
                &scene,
                Vec3::new(c.x as f32 + 0.5, c.y as f32 - 1.5, c.z as f32 + 0.5)
            ),
            "{c:?}"
        );
    }
    let upper_paved: Vec<(i32, i32)> = upper
        .cells()
        .iter()
        .chain(upper.accents.iter())
        .map(|c| (c.x, c.z))
        .collect();
    let lower_paved: Vec<(i32, i32)> = lower.main.iter().map(|c| (c.x, c.z)).collect();
    for c in upper_cells(&scene) {
        // Scenery never stands on a paved column (canopies may hang high above it).
        let leaf = scene.world().get(c).unwrap().block_type() == BlockType::Leaves;
        assert!(
            leaf || !upper_paved.contains(&(c.x, c.z)),
            "{c:?} on a path column"
        );
    }
    for c in lower_cells(&scene) {
        assert!(
            !lower_paved.contains(&(c.x, c.z)),
            "{c:?} under a path column"
        );
    }
}

#[test]
fn no_point_light_was_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    let points = lights
        .iter()
        .filter(|l| !matches!(l, Light::Directional(_)))
        .count();
    assert_eq!(points, 4);
    let world = std::fs::read_to_string("src/scene/world.rs").unwrap();
    let expansion = std::fs::read_to_string("src/scene/expansion.rs").unwrap();
    assert!(!expansion.contains("PointLight"));
    assert_eq!(world.matches("PointLight::new(").count(), 0);
}

#[test]
fn the_portal_is_not_interfered_with() {
    let scene = WorldScene::new();
    let portal = scene.portal();
    assert_eq!(portal.frame.len(), 18);
    assert_eq!(portal.core.len(), 12);
    for c in &portal.frame {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PortalFrameRedObsidian
        );
    }
    for c in &portal.core {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::PortalCoreDarkCrimson
        );
    }
    let route = scene.inverted_route();
    for cell in route.layout.exit_clearance() {
        assert!(!scene.world().contains(cell));
    }
    let keep: Vec<IVec3> = portal
        .frame
        .iter()
        .chain(portal.core.iter())
        .chain(route.treads.iter().map(|(t, _)| t))
        .chain(route.supports.iter())
        .copied()
        .collect();
    for c in upper_cells(&scene).iter().chain(lower_cells(&scene).iter()) {
        assert!(!keep.contains(c), "{c:?} is a portal or route cell");
        assert!(
            keep.iter()
                .all(|k| (k.x - c.x).abs() + (k.y - c.y).abs() + (k.z - c.z).abs() >= 2),
            "{c:?} touches the portal route"
        );
    }
}

#[test]
fn the_scenery_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.expansion_scenery(), b.expansion_scenery());
    assert_eq!(a.world().len(), b.world().len());
    for c in upper_cells(&a).iter().chain(lower_cells(&a).iter()) {
        assert_eq!(a.world().get(*c), b.world().get(*c));
    }
    let added = upper_cells(&a).len() + a.expansion_scenery().lower.added_cells().len();
    assert!(added <= 80, "{added} scenery cells");
    // The Gate 16 castle has its own budget on top of the Gate 15 masses.
    assert!(
        a.world().len() - a.architecture_voxels() <= VOXEL_BUDGET_SOFT_MAX,
        "{}",
        a.world().len()
    );
    println!("GATE15 scenery added={added} voxels={}", a.world().len());
}

#[test]
fn no_focal_scene_is_cluttered() {
    let scene = WorldScene::new();
    let counts: Vec<(usize, usize)> = scene
        .families()
        .iter()
        .map(|f| (f.symbols.len(), f.accents.len()))
        .collect();
    assert_eq!(counts, vec![(25, 12), (21, 6), (23, 10)]);
    let focal: Vec<IVec3> = scene
        .families()
        .iter()
        .flat_map(|f| f.symbols.iter().chain(f.accents.iter()).copied())
        .collect();
    for c in &focal {
        assert!(scene.world().contains(*c));
    }
    for c in lower_cells(&scene) {
        assert!(
            focal
                .iter()
                .all(|f| (f.x - c.x).abs() >= FOCAL_CLEARANCE
                    || (f.z - c.z).abs() >= FOCAL_CLEARANCE),
            "{c:?} crowds a focal cell"
        );
    }
    // The original Overworld landmarks keep every cell.
    let house = scene.house_exterior();
    let originals: Vec<IVec3> = scene
        .trees()
        .iter()
        .flat_map(|t| t.trunk.iter().chain(t.leaves.iter()).copied())
        .chain(scene.pond().water.iter().copied())
        .chain(
            house
                .door
                .iter()
                .chain(house.glass.iter())
                .chain(house.fence.iter())
                .copied(),
        )
        .collect();
    assert_eq!(scene.trees().len(), 4);
    for c in &originals {
        assert!(scene.world().contains(*c), "{c:?} lost");
    }
    for c in upper_cells(&scene) {
        assert!(!originals.contains(&c));
        assert!(c.x >= 19, "{c:?} is west of the lobe");
    }
    for t in &scene.expansion_scenery().upper.trees {
        assert_eq!(t.trunk_height, 3);
        assert!(
            scene
                .trees()
                .iter()
                .all(|o| (o.base.x - t.base.x).abs() >= 3 || (o.base.z - t.base.z).abs() >= 3)
        );
    }
}

/// Gate 16 built the castle on the pad: its ground row is the castle
/// foundation, level with the pad surface.
fn pad_is_castle_ground(scene: &WorldScene) {
    let l = scene.expansion_layout();
    let pad = l.overworld_castle_pad;
    assert_eq!(scene.castle_layout().footprint, pad);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let ground = IVec3::new(x, l.castle_pad_surface_y, z);
            let t = scene.world().get(ground).map(|b| b.block_type());
            assert!(
                matches!(
                    t,
                    Some(
                        BlockType::DeepslateBricks
                            | BlockType::Cobblestone
                            | BlockType::Stone
                            | BlockType::WoodPlanks
                    )
                ),
                "({x},{z}) is {t:?}"
            );
            assert!(scene.world().contains(IVec3::new(x, ground.y - 1, z)));
        }
    }
}

/// Gate 17 built the fortress on the lower pad: its ground row is the dark
/// foundation, level with the pad surface, on the Gate 15 shell columns.
fn pad_is_fortress_ground(scene: &WorldScene) {
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    assert_eq!(scene.fortress_layout().footprint, pad);
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            let ground = IVec3::new(x, l.fortress_pad_bottom_y, z);
            let b = scene
                .world()
                .get(ground)
                .unwrap_or_else(|| panic!("({x},{z}) missing"));
            assert_eq!(b.orientation(), Orientation::Down, "({x},{z})");
            assert!(
                matches!(
                    b.block_type(),
                    BlockType::PolishedBlackstoneBricks
                        | BlockType::SmoothBasalt
                        | BlockType::Deepslate
                        | BlockType::NetherWartBlock
                        | BlockType::Mycelium
                ),
                "({x},{z}) is {:?}",
                b.block_type()
            );
            assert!(scene.red_black_expansion().column(x, z).is_some());
        }
    }
}
