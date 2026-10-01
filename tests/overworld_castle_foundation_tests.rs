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
use scene::castle::{WALL_HEIGHT, build_castle_foundation};
use scene::orientation::Orientation;
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;

fn clear(world: &VoxelWorld, p: Vec3) -> bool {
    is_position_clear(
        world,
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn eye(c: IVec3) -> Vec3 {
    Vec3::new(c.x as f32 + 0.5, c.y as f32 + 2.0, c.z as f32 + 0.5)
}

fn masonry(t: BlockType) -> bool {
    // Later stages may re-lay a curtain cell in deepslate bricks or set a
    // lamp sconce into it.
    matches!(
        t,
        BlockType::Stone
            | BlockType::Cobblestone
            | BlockType::DeepslateBricks
            | BlockType::RedstoneLampLit
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
fn the_foundation_replaces_the_pad_and_is_connected() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = scene.castle_foundation();
    let g = c.levels.ground_y;
    let pad = c.footprint;
    assert_eq!(
        f.footings.len() + f.floors.len(),
        (pad.width() * pad.depth()) as usize,
        "every pad column gets a ground cell"
    );
    for cell in f.footings.iter().chain(f.floors.iter()) {
        assert_eq!(cell.y, g);
        assert!(pad.contains(cell.x, cell.z));
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Up);
        // Resting on the Gate 15 strata, never floating.
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?}"
        );
    }
    for cell in &f.footings {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::DeepslateBricks
        );
        let (x, z) = (cell.x, cell.z);
        assert!(
            c.is_curtain_column(x, z)
                || c.is_tower_shell(x, z)
                || c.is_keep_shell(x, z)
                || c.gatehouse.contains(x, z),
            "{cell:?} is not under an outline"
        );
    }
    for cell in &f.floors {
        // The keep later lays planks over its hall floor.
        let t = scene.world().get(*cell).unwrap().block_type();
        assert!(
            masonry(t) || t == BlockType::WoodPlanks,
            "{cell:?} is {t:?}"
        );
    }
    println!(
        "GATE16 foundation: footings={} floors={} walls={} gate_opening={} voxels={}",
        f.footings.len(),
        f.floors.len(),
        f.walls.len(),
        f.gate_opening.len(),
        scene.world().len()
    );
}

#[test]
fn the_courtyard_is_paved_and_clear() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let g = c.levels.ground_y;
    for (x, z) in c.courtyard.iter().chain(c.gatehouse_passage().iter()) {
        if detailed(&scene, *x, *z) {
            continue;
        }
        let ground = IVec3::new(*x, g, *z);
        assert_eq!(
            scene.world().get(ground).unwrap().block_type(),
            BlockType::Cobblestone,
            "({x},{z})"
        );
        // Head room: nothing solid (the gate's door, added later, is passable).
        for dy in 1..=2 {
            let open = scene
                .world()
                .get(IVec3::new(*x, g + dy, *z))
                .is_none_or(|b| !is_camera_solid(b));
            assert!(open, "({x},{z}) + {dy}");
        }
        assert!(clear(scene.world(), eye(ground)), "({x},{z})");
    }
}

#[test]
fn the_gate_opening_exists_in_the_wall() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = scene.castle_foundation();
    assert_eq!(f.gate_opening, c.gate.cells());
    // Left open by this stage; the gatehouse later hangs the passable door.
    for cell in &f.gate_opening {
        let open = scene.world().get(*cell).is_none_or(|b| !is_camera_solid(b));
        assert!(open, "{cell:?} blocked");
    }
    // Wall above and beside the opening.
    let g = &c.gate;
    let lintel = IVec3::new(g.base.x, g.base.y + 2, g.base.z);
    assert!(
        scene
            .world()
            .get(lintel)
            .is_some_and(|b| masonry(b.block_type()))
    );
    let jambs = [
        IVec3::new(g.base.x, g.base.y, g.base.z - 1),
        IVec3::new(g.base.x, g.base.y, g.base.z + g.width),
    ];
    for j in jambs {
        assert!(
            scene
                .world()
                .get(j)
                .is_some_and(|b| masonry(b.block_type())),
            "{j:?}"
        );
    }
    assert!(clear(
        scene.world(),
        Vec3::new(
            g.base.x as f32 + 0.5,
            g.base.y as f32 + 1.0,
            g.base.z as f32 + 1.0
        )
    ));
}

#[test]
fn the_curtain_wall_is_anchored_and_hollow() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = scene.castle_foundation();
    let pad = c.footprint;
    let perimeter = (2 * (pad.width() + pad.depth()) - 4) as usize;
    assert_eq!(
        f.walls.len(),
        perimeter * WALL_HEIGHT as usize - f.gate_opening.len()
    );
    let mut cobble = 0;
    for cell in &f.walls {
        assert!(c.is_curtain_column(cell.x, cell.z));
        assert!(cell.y >= c.levels.base_y && cell.y <= c.levels.wall_top_y);
        let b = scene.world().get(*cell).unwrap();
        assert!(masonry(b.block_type()));
        assert!(is_camera_solid(b));
        if b.block_type() == BlockType::Cobblestone {
            cobble += 1;
        }
        // Anchored: a cell below down to the footing.
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
                || f.gate_opening
                    .contains(&IVec3::new(cell.x, cell.y - 1, cell.z))
        );
    }
    assert!(
        cobble > f.walls.len() / 6 && cobble < f.walls.len() / 2,
        "{cobble}"
    );
    // Hollow: the column just inside the wall holds nothing at wall height
    // where it is courtyard.
    for (x, z) in &c.courtyard {
        if detailed(&scene, *x, *z) {
            continue;
        }
        for y in c.levels.base_y..=c.levels.wall_top_y {
            assert!(!scene.world().contains(IVec3::new(*x, y, *z)));
        }
    }
}

#[test]
fn the_approach_path_is_unchanged() {
    let scene = WorldScene::new();
    let a = scene.overworld_approach();
    for cell in a.cells() {
        let t = scene.world().get(cell).unwrap().block_type();
        assert!(
            matches!(t, BlockType::Cobblestone | BlockType::Stone),
            "{cell:?}"
        );
        assert!(clear(scene.world(), eye(cell)));
    }
    for cell in &a.accents {
        assert!(matches!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Fence | BlockType::Log
        ));
    }
    // The foundation touches no column outside the pad.
    let pad = scene.castle_layout().footprint;
    for cell in scene.castle_foundation().cells() {
        assert!(pad.contains(cell.x, cell.z));
    }
}

#[test]
fn the_stage_builds_the_same_on_a_bare_pad() {
    // The builder on a copy of the world before the castle yields the
    // recorded stage, deterministically.
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.castle_foundation(), b.castle_foundation());
    let mut world = VoxelWorld::new();
    for cell in a.castle_foundation().cells() {
        world.insert(
            IVec3::new(cell.x, cell.y - 1, cell.z),
            *a.world()
                .get(IVec3::new(cell.x, cell.y - 1, cell.z))
                .unwrap_or(a.world().get(cell).unwrap()),
        );
    }
    let again = build_castle_foundation(a.config(), a.castle_layout(), &mut world);
    assert_eq!(again.walls, a.castle_foundation().walls);
    assert_eq!(again.footings, a.castle_foundation().footings);
}
