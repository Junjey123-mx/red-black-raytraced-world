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
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
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
use scene::overworld::{build_house, carve_pond, furnish_house, lay_path, plant_trees};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;
use std::collections::HashMap;

/// The Gate 12 Overworld built through the same feature functions, so the
/// scene can be compared cell by cell above the terrain bottom.
fn gate12_world() -> VoxelWorld {
    let config = TerrainConfig::official();
    let mut world = VoxelWorld::new();
    generate_terrain(&config, &mut world);
    let pond = carve_pond(&config, &mut world);
    build_house(&config, &mut world);
    furnish_house(&config, &mut world);
    let trees = plant_trees(&config, &mut world, &pond);
    lay_path(&config, &mut world, &pond, &trees);
    world
}

/// Cells the upper descent is allowed to edit: its dug pit, treads,
/// footings, shelf plate and brick approach.
fn descent_cells(scene: &WorldScene) -> std::collections::HashSet<IVec3> {
    let d = scene.upper_descent();
    d.dug
        .iter()
        .copied()
        .chain(d.treads.iter().map(|(c, _)| *c))
        .chain(d.footings.iter().copied())
        .chain(d.shelf.iter().copied())
        .chain(d.approach.iter().copied())
        .collect()
}

fn census(world: &VoxelWorld, min_y: i32, max_y: i32) -> HashMap<BlockType, usize> {
    let mut counts = HashMap::new();
    for z in -4..28 {
        for x in -4..28 {
            for y in min_y..=max_y {
                if let Some(b) = world.get(IVec3::new(x, y, z)) {
                    *counts.entry(b.block_type()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

#[test]
fn the_gate12_surface_and_landmarks_are_preserved_exactly() {
    let scene = WorldScene::new();
    let reference = gate12_world();
    let r = scene.rhombus();
    let descent = descent_cells(&scene);
    // Every cell from the terrain's rim bottom up is identical outside the
    // physical cutaway quadrant (which removes cells, never edits them).
    for z in -4..28 {
        for x in -4..28 {
            if r.cutaway.contains_column(x, z) {
                continue;
            }
            for y in r.upper_taper_top..=24 {
                let cell = IVec3::new(x, y, z);
                if descent.contains(&cell) {
                    continue;
                }
                assert_eq!(
                    scene.world().get(cell),
                    reference.get(cell),
                    "cell {cell:?} changed"
                );
            }
        }
    }
    assert_eq!(scene.house().floor.len(), 35);
    assert_eq!(scene.house_exterior().lamps.len(), 2);
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(scene.trees().len(), 4);
    assert_eq!(scene.path().len(), 10);
    let above = census(scene.world(), r.upper_taper_top, 24);
    let above_ref = census(&reference, r.upper_taper_top, 24);
    for (t, n) in &above_ref {
        // Only the descent's stairs may grow above the terrain bottom.
        if *t == BlockType::WoodStairs {
            continue;
        }
        assert!(above.get(t).copied().unwrap_or(0) <= *n, "{t:?} grew");
    }
    assert_eq!(
        above[&BlockType::WoodPlanks],
        above_ref[&BlockType::WoodPlanks]
    );
    assert_eq!(above[&BlockType::Water], above_ref[&BlockType::Water]);
    assert_eq!(above[&BlockType::Log], above_ref[&BlockType::Log]);
}

#[test]
fn the_section_narrows_row_by_row_toward_the_waist() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let terrain = scene.config();
    let area = |y: i32| {
        (-4..28)
            .flat_map(|z| (-4..28).map(move |x| (x, z)))
            .filter(|&(x, z)| scene.world().contains(IVec3::new(x, y, z)))
            .count()
    };
    let mut previous = area(r.upper_taper_top);
    for y in (r.waist_y..r.upper_taper_top).rev() {
        let current = area(y);
        assert!(current < previous, "row {y}: {current} vs {previous} above");
        previous = current;
    }
    // The waist is much narrower than the surface but not a needle.
    let surface = area(r.upper_taper_top);
    let waist = area(r.waist_y);
    assert!(waist * 2 < surface, "waist {waist} vs surface {surface}");
    assert!(waist > 100);
    assert!(r.section_width(terrain, r.waist_y) < r.section_width(terrain, r.upper_taper_top));
}

#[test]
fn the_taper_reaches_the_shelf_as_a_solid_mass() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let taper = scene.upper_taper();
    assert_eq!(taper.top_y, r.upper_taper_top - 1);
    assert_eq!(taper.bottom_y, r.shelf_y);
    assert!(taper.cells.len() > 800, "{} taper cells", taper.cells.len());
    let descent = descent_cells(&scene);
    for cell in &taper.cells {
        assert!(cell.y <= taper.top_y && cell.y >= taper.bottom_y);
        assert!(
            scene.world().contains(*cell)
                || r.is_cut(cell.x, cell.y, cell.z)
                || descent.contains(cell)
        );
        assert!(r.contains(scene.config(), cell.x, cell.y, cell.z));
    }
    // Solid: every cell inside the section between the shelf and the
    // terrain bottom exists, so there are no cavities.
    for y in r.shelf_y..r.upper_taper_top {
        for z in -4..28 {
            for x in -4..28 {
                if r.contains(scene.config(), x, y, z)
                    && !r.is_cut(x, y, z)
                    && !descent.contains(&IVec3::new(x, y, z))
                {
                    assert!(
                        scene.world().contains(IVec3::new(x, y, z)),
                        "cavity at ({x}, {y}, {z})"
                    );
                }
            }
        }
    }
    // Nothing yet under the shelf.
    for y in r.lower_tip_y..r.shelf_y {
        for z in -4..28 {
            for x in -4..28 {
                assert!(!scene.world().contains(IVec3::new(x, y, z)));
            }
        }
    }
}

#[test]
fn the_added_mass_is_geological_and_turns_to_deepslate_downward() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let counts = census(scene.world(), r.shelf_y, r.upper_taper_top - 1);
    for (t, n) in &counts {
        // The descent's own blocks (stairs, bricks) pass through the mass.
        if matches!(t, BlockType::WoodStairs | BlockType::DeepslateBricks) {
            continue;
        }
        assert!(
            matches!(
                t,
                BlockType::Stone | BlockType::Deepslate | BlockType::Dirt | BlockType::Sand
            ),
            "{t:?} x{n} under the terrain"
        );
        assert_eq!(t.family(), BlockFamily::OverworldTerrain);
    }
    assert!(counts[&BlockType::Deepslate] > counts.get(&BlockType::Stone).copied().unwrap_or(0));
    assert!(
        counts.get(&BlockType::Stone).copied().unwrap_or(0) > 0,
        "no stone veins"
    );
    // No grass buried in the mass, deepslate dominant near the waist.
    assert!(!counts.contains_key(&BlockType::Grass));
    let waist = census(scene.world(), r.waist_y, r.waist_y);
    assert!(waist[&BlockType::Deepslate] * 10 > waist.values().sum::<usize>() * 9);
}

#[test]
fn no_portal_or_red_black_block_exists_yet() {
    let scene = WorldScene::new();
    let counts = census(scene.world(), -40, 24);
    for t in counts.keys() {
        assert!(
            matches!(
                t.family(),
                BlockFamily::OverworldTerrain | BlockFamily::OverworldArchitecture
            ),
            "{t:?}"
        );
    }
    assert!(!counts.contains_key(&BlockType::PortalFrameRedObsidian));
    assert!(!counts.contains_key(&BlockType::PortalCoreDarkCrimson));
}

#[test]
fn the_taper_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.upper_taper(), b.upper_taper());
    assert_eq!(a.world().len(), b.world().len());
}
