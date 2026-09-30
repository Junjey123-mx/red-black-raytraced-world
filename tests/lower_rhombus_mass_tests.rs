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
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
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

use core::math::IVec3;
use scene::block_type::{BlockFamily, BlockType};
use scene::cutaway::carve_cutaway;
use scene::descent::build_upper_descent;
use scene::orientation::Orientation;
use scene::overworld::{build_house, carve_pond, furnish_house, lay_path, plant_trees};
use scene::portal::{build_portal_core, build_portal_frame};
use scene::red_black_maze::{LowerMass, build_lower_mass};
use scene::rhombus::{RhombusConfig, build_upper_taper};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;
use std::collections::HashMap;

/// The lower-mass stage: everything up to `build_lower_mass`, before the
/// inverted descent and the Red-Black surface edit the lower half.
struct Stage {
    config: TerrainConfig,
    rhombus: RhombusConfig,
    lower_mass: LowerMass,
    world: VoxelWorld,
}

impl Stage {
    fn new() -> Self {
        let config = TerrainConfig::official();
        let rhombus = RhombusConfig::derive(&config);
        let mut world = VoxelWorld::new();
        generate_terrain(&config, &mut world);
        let pond = carve_pond(&config, &mut world);
        build_house(&config, &mut world);
        furnish_house(&config, &mut world);
        let trees = plant_trees(&config, &mut world, &pond);
        lay_path(&config, &mut world, &pond, &trees);
        build_upper_taper(&config, &rhombus, &mut world);
        carve_cutaway(&config, &rhombus, &mut world);
        build_upper_descent(&config, &rhombus, &mut world);
        let mut portal = build_portal_frame(&rhombus, &mut world);
        build_portal_core(&rhombus, &mut world, &mut portal);
        let lower_mass = build_lower_mass(&config, &rhombus, &mut world);
        Self {
            config,
            rhombus,
            lower_mass,
            world,
        }
    }

    fn world(&self) -> &VoxelWorld {
        &self.world
    }

    fn rhombus(&self) -> &RhombusConfig {
        &self.rhombus
    }

    fn config(&self) -> &TerrainConfig {
        &self.config
    }

    fn lower_mass(&self) -> &LowerMass {
        &self.lower_mass
    }
}

fn area(scene: &Stage, y: i32) -> usize {
    (-4..28)
        .flat_map(|z| (-4..28).map(move |x| (x, z)))
        .filter(|&(x, z)| scene.world().contains(IVec3::new(x, y, z)))
        .count()
}

fn census(scene: &Stage, min_y: i32, max_y: i32) -> HashMap<BlockType, usize> {
    let mut counts = HashMap::new();
    for z in -4..28 {
        for x in -4..28 {
            for y in min_y..=max_y {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    *counts.entry(b.block_type()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

#[test]
fn the_lower_mass_connects_to_the_waist_in_the_same_world() {
    let scene = Stage::new();
    let r = scene.rhombus();
    let m = scene.lower_mass();
    assert_eq!(m.top_y, r.shelf_y - 1);
    assert_eq!(m.bottom_y, r.lower_tip_y);
    assert!(m.cells.len() > 1000, "{} lower cells", m.cells.len());
    // Under every shelf/waist cell outside the cut there is lower mass.
    let mut joined = 0;
    for z in -4..28 {
        for x in -4..28 {
            let shelf = IVec3::new(x, r.shelf_y, z);
            let below = IVec3::new(x, r.shelf_y - 1, z);
            if scene.world().contains(shelf)
                && r.contains(scene.config(), x, r.shelf_y - 1, z)
                && !r.is_cut(x, r.shelf_y - 1, z)
            {
                assert!(
                    scene.world().contains(below),
                    "gap under the shelf at ({x}, {z})"
                );
                joined += 1;
            }
        }
    }
    assert!(joined > 50);
    // One VoxelWorld holds a grass block of the Overworld and a tip block.
    let full = WorldScene::new();
    let grass = full.world().get(full.trees()[0].base).unwrap();
    assert_eq!(grass.block_type(), BlockType::Grass);
    assert!(
        full.world()
            .contains(IVec3::new(r.center_x, r.lower_tip_y, r.center_z))
    );
    assert!(area(&scene, r.lower_tip_y) >= 1);
    let src = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(src.matches("world: VoxelWorld,").count(), 1);
    assert!(!src.contains("SecondWorld") && !src.contains("LowerScene"));
}

#[test]
fn the_section_widens_under_the_waist_and_tapers_to_the_tip() {
    let scene = Stage::new();
    let r = scene.rhombus();
    let waist = area(&scene, r.waist_y);
    let widest = area(&scene, r.lower_widest_y);
    assert!(widest > waist, "widest {widest} vs waist {waist}");
    // Monotonic widening then narrowing (allowing the cut quadrant's
    // missing cells, the trend is checked on the un-cut section areas).
    let section = |y: i32| r.section_area(scene.config(), y);
    for y in (r.lower_widest_y + 1)..=r.waist_y {
        assert!(section(y - 1) >= section(y), "row {y} does not widen");
    }
    for y in (r.lower_tip_y + 1)..=r.lower_widest_y {
        assert!(section(y - 1) <= section(y), "row {y} does not narrow");
    }
    let tip = area(&scene, r.lower_tip_y);
    assert!((1..=4).contains(&tip), "tip has {tip} cells");
    assert_eq!(area(&scene, r.lower_tip_y - 1), 0);
    assert!(widest > tip * 20);
}

#[test]
fn the_lower_mass_is_solid_and_reaches_the_tip() {
    let scene = Stage::new();
    let r = scene.rhombus();
    for y in r.lower_tip_y..r.shelf_y {
        for z in -4..28 {
            for x in -4..28 {
                let inside = r.contains(scene.config(), x, y, z) && !r.is_cut(x, y, z);
                assert_eq!(
                    scene.world().contains(IVec3::new(x, y, z)),
                    inside,
                    "({x}, {y}, {z})"
                );
            }
        }
    }
    assert!(
        scene
            .world()
            .contains(IVec3::new(r.center_x, r.lower_tip_y, r.center_z))
    );
}

#[test]
fn the_lower_mass_uses_dark_structural_materials_oriented_down() {
    let scene = Stage::new();
    let r = scene.rhombus();
    let counts = census(&scene, r.lower_tip_y, r.shelf_y - 1);
    for (t, n) in &counts {
        assert!(
            matches!(
                t,
                BlockType::Deepslate
                    | BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
            ),
            "{t:?} x{n} in the lower mass"
        );
    }
    assert!(counts[&BlockType::Deepslate] > 0);
    assert!(counts[&BlockType::SmoothBasalt] > 0);
    assert!(counts[&BlockType::PolishedBlackstoneBricks] > 0);
    for cell in &scene.lower_mass().cells {
        assert_eq!(
            scene.world().get(*cell).unwrap().orientation(),
            Orientation::Down
        );
    }
    // No suit symbols or other family decoration yet.
    for t in counts.keys() {
        assert!(!matches!(
            t.family(),
            BlockFamily::Crimson | BlockFamily::Orange | BlockFamily::Violet
        ));
    }
}

#[test]
fn the_lower_mass_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.lower_mass(), b.lower_mass());
}
