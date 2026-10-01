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

use camera::projection::primary_ray;
use camera::world_collision::{CameraCollisionConfig, is_camera_solid, is_position_clear};
use core::math::{IVec3, Vec3};
use renderer::raytracer::{VoxelScene, nearest_visible_hit};
use scene::block_type::BlockType;
use scene::castle::{MERLON_PERIOD, TOWER_HEIGHT, TOWER_SIDE};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_background,
    world_camera, world_lights, world_materials,
};

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn stone_family(t: BlockType) -> bool {
    // A later stage may set a window bay (Glass) into a shell cell.
    matches!(
        t,
        BlockType::Stone | BlockType::Cobblestone | BlockType::DeepslateBricks | BlockType::Glass
    )
}

/// Columns where a later stage (C193) stands a fence post, rail or lamp.
fn detailed(scene: &WorldScene, x: i32, z: i32) -> bool {
    scene
        .castle_details()
        .cells()
        .iter()
        .any(|c| c.x == x && c.z == z)
}

#[test]
fn the_tower_count_and_sizes_are_valid() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let t = scene.castle_towers();
    assert_eq!(c.towers.len(), 4);
    assert_eq!(t.cores.len(), 4);
    let shell_per_tower = (4 * (TOWER_SIDE - 1)) as usize * TOWER_HEIGHT as usize;
    assert_eq!(t.shells.len(), 4 * shell_per_tower);
    for (i, r) in c.towers.iter().enumerate() {
        let core = t.cores[i];
        assert_eq!(core, ((r.min_x + r.max_x) / 2, (r.min_z + r.max_z) / 2));
        let own: Vec<&IVec3> = t.shells.iter().filter(|s| r.contains(s.x, s.z)).collect();
        assert_eq!(own.len(), shell_per_tower);
        let top = own.iter().map(|s| s.y).max().unwrap();
        assert_eq!(top, c.levels.tower_top_y);
    }
    println!(
        "GATE16 towers: shells={} tower_merlons={} wall_merlons={} castle_voxels={} voxels={}",
        t.shells.len(),
        t.tower_merlons.len(),
        t.wall_merlons.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_towers_are_hollow_stone_shells() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let t = scene.castle_towers();
    for cell in &t.shells {
        let b = scene.world().get(*cell).unwrap();
        assert!(
            stone_family(b.block_type()),
            "{cell:?} is {:?}",
            b.block_type()
        );
        assert!(is_camera_solid(b));
        if cell.y == c.levels.base_y {
            assert_eq!(b.block_type(), BlockType::DeepslateBricks);
        }
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?} floats"
        );
    }
    for (x, z) in &t.cores {
        for y in c.levels.base_y..=c.levels.max_y {
            assert!(
                !scene.world().contains(IVec3::new(*x, y, *z)),
                "({x},{y},{z}) fills the core"
            );
        }
        // The core floor is stone, the shaft is open to the sky.
        assert_eq!(
            scene
                .world()
                .get(IVec3::new(*x, c.levels.ground_y, *z))
                .unwrap()
                .block_type(),
            BlockType::Stone
        );
    }
    // No solid mass: shells are exactly the outline, nothing else was added
    // inside any tower footprint.
    for r in &c.towers {
        for z in r.min_z..=r.max_z {
            for x in r.min_x..=r.max_x {
                let shell = x == r.min_x || x == r.max_x || z == r.min_z || z == r.max_z;
                for y in c.levels.base_y..=c.levels.tower_top_y {
                    assert_eq!(
                        scene.world().contains(IVec3::new(x, y, z)),
                        shell,
                        "({x},{y},{z})"
                    );
                }
            }
        }
    }
}

#[test]
fn battlements_are_present_with_rhythm() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let t = scene.castle_towers();
    assert_eq!(t.tower_merlons.len(), 16);
    for m in &t.tower_merlons {
        assert_eq!(m.y, c.levels.max_y);
        let r = c.tower_at(m.x, m.z).unwrap();
        assert!((m.x == r.min_x || m.x == r.max_x) && (m.z == r.min_z || m.z == r.max_z));
        assert!(stone_family(scene.world().get(*m).unwrap().block_type()));
        assert!(scene.world().contains(IVec3::new(m.x, m.y - 1, m.z)));
    }
    assert!(t.wall_merlons.len() >= 4, "{}", t.wall_merlons.len());
    for m in &t.wall_merlons {
        assert_eq!(m.y, c.levels.wall_top_y + 1);
        assert!(c.is_curtain_column(m.x, m.z));
        assert!(
            c.tower_at(m.x, m.z).is_none()
                && !c.keep.contains(m.x, m.z)
                && !c.gatehouse.contains(m.x, m.z)
        );
        assert!(scene.world().contains(IVec3::new(m.x, m.y - 1, m.z)));
        // Rhythm: the neighbouring wall cell along the wall carries none.
        let along = if m.z == c.footprint.min_z || m.z == c.footprint.max_z {
            IVec3::new(m.x + 1, m.y, m.z)
        } else {
            IVec3::new(m.x, m.y, m.z + 1)
        };
        assert!(
            !t.wall_merlons.contains(&along),
            "{m:?} has a neighbour merlon"
        );
    }
    let _ = MERLON_PERIOD;
}

#[test]
fn the_approach_gate_and_courtyard_stay_open() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = c.levels.ground_y;
    for cell in scene.overworld_approach().cells() {
        assert!(clear(
            &scene,
            Vec3::new(
                cell.x as f32 + 0.5,
                cell.y as f32 + 2.0,
                cell.z as f32 + 0.5
            )
        ));
    }
    for cell in c.gate.cells() {
        assert!(scene.world().get(cell).is_none_or(|b| !is_camera_solid(b)));
    }
    for (x, z) in c.courtyard.iter().chain(c.gatehouse_passage().iter()) {
        if detailed(&scene, *x, *z) {
            continue;
        }
        for dy in 1..=2 {
            // Nothing solid (the gate's door is passable).
            let open = scene
                .world()
                .get(IVec3::new(*x, g + dy, *z))
                .is_none_or(|b| !is_camera_solid(b));
            assert!(open, "({x},{z}) + {dy}");
        }
    }
    for cell in scene.castle_towers().cells() {
        assert!(c.footprint.contains(cell.x, cell.z));
        assert!(!c.courtyard.contains(&(cell.x, cell.z)));
    }
}

#[test]
fn the_skyline_and_the_house_stay_visible() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let bounds = scene.world().bounds().unwrap();
    assert_eq!(
        bounds.max_exclusive.y, 13,
        "the castle rises above the world box"
    );
    assert!(c.levels.max_y < bounds.max_exclusive.y);
    let house_top = scene
        .house_exterior()
        .roof
        .iter()
        .map(|(r, _, _)| r.y)
        .max()
        .unwrap();
    assert!(
        c.levels.max_y <= house_top,
        "towers {} vs house roof {house_top}",
        c.levels.max_y
    );
    // From the official framing both the house roof and a tower are seen.
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let camera = world_camera(4.0 / 3.0);
    let view = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR,
        background: world_background(),
        texture_manager: &manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    let roof: Vec<IVec3> = scene
        .house_exterior()
        .roof
        .iter()
        .map(|(r, _, _)| *r)
        .collect();
    let shells = &scene.castle_towers().shells;
    let (w, h) = (160, 120);
    let (mut roof_hits, mut tower_hits) = (0, 0);
    for y in 0..h {
        for x in 0..w {
            let ray = primary_ray(&camera, x, y, w, h);
            if let Some(hit) = nearest_visible_hit(&view, &ray, WORLD_MAX_DISTANCE) {
                if roof.contains(&hit.cell) {
                    roof_hits += 1;
                }
                if shells.contains(&hit.cell) {
                    tower_hits += 1;
                }
            }
        }
    }
    assert!(roof_hits > 20, "house roof barely visible: {roof_hits}");
    assert!(tower_hits > 20, "towers barely visible: {tower_hits}");
    println!("GATE16 framing: roof_hits={roof_hits} tower_hits={tower_hits}");
}
