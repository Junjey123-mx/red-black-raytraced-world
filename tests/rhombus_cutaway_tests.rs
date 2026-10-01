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
use scene::block_type::BlockType;
use scene::cutaway::carve_cutaway;
use scene::overworld::{HOUSE_FOOTPRINT, pond_basin_columns};
use scene::overworld_blocks::{deepslate_material_id, dirt_material_id, stone_block_material_id};
use scene::rhombus::{RhombusConfig, build_upper_taper};
use scene::scene::grass_material_id;
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{WorldScene, WorldTextures, world_materials};
use std::collections::HashSet;

#[test]
fn the_cutaway_region_is_the_deterministic_south_east_quadrant() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.cutaway(), b.cutaway());
    let r = a.rhombus();
    assert_eq!(a.cutaway().region, r.cutaway);
    assert_eq!(a.cutaway().shelf_y, r.shelf_y);
    assert!(r.cutaway.min_x > r.center_x && r.cutaway.min_z > r.center_z);
    // Removed cells stay removed unless the descent deliberately re-uses
    // them (its exit stair and footing stand inside the cut).
    let descent: HashSet<IVec3> = a
        .upper_descent()
        .treads
        .iter()
        .map(|(c, _)| *c)
        .chain(a.upper_descent().footings.iter().copied())
        .collect();
    for cell in &a.cutaway().removed {
        assert!(r.is_cut(cell.x, cell.y, cell.z));
        assert!(
            !a.world().contains(*cell) || descent.contains(cell),
            "{cell:?} still exists"
        );
        assert_ne!(cell.y, r.shelf_y);
    }
    assert!(a.cutaway().removed_count() > 200);
}

#[test]
fn the_house_and_the_pond_are_untouched() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    for cell in scene.house().floor.iter().chain(scene.house().walls.iter()) {
        assert!(scene.world().contains(*cell));
        assert!(!r.cutaway.contains_column(cell.x, cell.z));
    }
    for (cell, _, _) in &scene.house_exterior().roof {
        assert!(scene.world().contains(*cell));
    }
    let h = HOUSE_FOOTPRINT.grown(2);
    for cell in &scene.cutaway().removed {
        assert!(!h.contains(cell.x, cell.z));
    }
    for cell in &scene.pond().water {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::Water)
        );
    }
    for (x, z) in pond_basin_columns() {
        assert!(!r.cutaway.contains_column(x, z));
    }
    assert_eq!(scene.pond().water.len(), 26);
}

#[test]
fn the_cut_walls_expose_dirt_stone_and_deepslate() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let mut exposed: HashSet<BlockType> = HashSet::new();
    // West wall of the cut (x = min_x - 1) and north wall (z = min_z - 1).
    for y in r.shelf_y..=r.upper_surface_reference {
        for z in r.cutaway.min_z..28 {
            let wall = IVec3::new(r.cutaway.min_x - 1, y, z);
            let air = IVec3::new(r.cutaway.min_x, y, z);
            if let Some(b) = scene.world().get(wall) {
                let descent_cell = scene.upper_descent().treads.iter().any(|(c, _)| *c == air)
                    || scene.upper_descent().footings.contains(&air);
                if y != r.shelf_y && !descent_cell {
                    assert!(!scene.world().contains(air), "{air:?} should be cut");
                }
                exposed.insert(b.block_type());
            }
        }
        for x in r.cutaway.min_x..28 {
            let wall = IVec3::new(x, y, r.cutaway.min_z - 1);
            if let Some(b) = scene.world().get(wall) {
                exposed.insert(b.block_type());
            }
        }
    }
    for t in [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Deepslate,
    ] {
        assert!(exposed.contains(&t), "{t:?} is not exposed by the cut");
    }
}

#[test]
fn most_of_the_mass_remains_solid() {
    let scene = WorldScene::new();
    let removed = scene.cutaway().removed_count();
    let remaining = scene.world().len();
    assert!(
        remaining > removed * 3,
        "removed {removed} of {}",
        removed + remaining
    );
    // The shelf survives inside the quadrant as the cut's floor.
    let r = scene.rhombus();
    let shelf = (r.cutaway.min_z..28)
        .flat_map(|z| (r.cutaway.min_x..28).map(move |x| (x, z)))
        .filter(|&(x, z)| scene.world().contains(IVec3::new(x, r.shelf_y, z)))
        .count();
    assert!(shelf >= 8, "{shelf} shelf cells");
}

#[test]
fn the_portal_anchor_is_open_to_the_cut_side() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let p = r.portal;
    // The wall cells where the frame will stand exist, and everything in
    // front of the opening (toward +Z) is air up to the diorama's edge.
    for x in p.min_x..=p.max_x {
        for y in p.min_y..=p.max_y {
            if p.is_frame(x, y) {
                assert!(
                    scene.world().contains(IVec3::new(x, y, p.wall_z)),
                    "wall ({x}, {y}) missing"
                );
            }
        }
    }
    // The three rows in front of the opening are clear at every opening
    // height, and the upper rows stay clear all the way out.
    for x in (p.min_x + 1)..p.max_x {
        for y in (p.min_y + 1)..p.max_y {
            for z in (p.wall_z + 1)..=(p.wall_z + 3) {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x}, {y}, {z}) blocks the portal"
                );
            }
        }
        for y in (p.max_y - 2)..=p.max_y {
            for z in (p.wall_z + 1)..30 {
                assert!(
                    !scene.world().contains(IVec3::new(x, y, z)),
                    "({x}, {y}, {z}) blocks the view"
                );
            }
        }
    }
}

#[test]
fn no_transparency_trick_and_no_material_change() {
    // The cut is made of absences: the same materials as before, opaque.
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    for id in [
        grass_material_id(),
        dirt_material_id(),
        stone_block_material_id(),
        deepslate_material_id(),
    ] {
        let m = materials.get(id).unwrap();
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.alpha_mode, core::material::AlphaMode::Ignore);
    }
    // Carving on a fresh terrain removes cells and adds none.
    let terrain = TerrainConfig::official();
    let r = RhombusConfig::derive(&terrain);
    let mut world = VoxelWorld::new();
    generate_terrain(&terrain, &mut world);
    build_upper_taper(&terrain, &r, &mut world);
    let before = world.len();
    let cut = carve_cutaway(&terrain, &r, &mut world);
    assert_eq!(world.len() + cut.removed_count(), before);
    for cell in &cut.removed {
        assert!(world.get(*cell).is_none());
    }
    let src = std::fs::read_to_string("src/scene/cutaway.rs").unwrap();
    assert!(!src.contains("transparency") || src.contains("No material is made transparent"));
    assert!(src.contains("world.remove("));
    assert!(!src.contains("world.insert("));
}
