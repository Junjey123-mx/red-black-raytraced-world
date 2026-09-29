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
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
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
use scene::overworld::{
    DESCENT_ENDPOINT, HOUSE_DOOR_X, HOUSE_FOOTPRINT, PATH_MAX_STEP, PATH_RESERVE, PATH_START,
    ground_top, pond_basin_columns,
};
use scene::overworld_blocks::cobblestone_material_id;
use scene::world::WorldScene;
use std::collections::HashSet;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

fn all_cobblestone(scene: &WorldScene) -> Vec<IVec3> {
    let c = *scene.config();
    let b = scene.bounds();
    let mut out = Vec::new();
    for z in 0..c.depth {
        for x in 0..c.width {
            for y in b.min.y..=b.max.y + 16 {
                let cell = IVec3::new(x, y, z);
                if block_type(scene, cell) == Some(BlockType::Cobblestone) {
                    out.push(cell);
                }
            }
        }
    }
    out
}

#[test]
fn the_path_is_cobblestone_from_the_door_to_the_descent_endpoint() {
    let scene = WorldScene::new();
    let path = scene.path();
    assert!(!path.is_empty());
    let start = path.start().unwrap();
    let end = path.end().unwrap();
    assert_eq!((start.x, start.z), PATH_START);
    assert_eq!(
        (start.x, start.z),
        (HOUSE_DOOR_X, HOUSE_FOOTPRINT.max_z + 1)
    );
    assert_eq!((end.x, end.z), DESCENT_ENDPOINT);
    for cell in path.cells.iter().chain(path.landing.iter()) {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Cobblestone);
        assert_eq!(b.material_id(), cobblestone_material_id());
        // On the surface: the cell above is air.
        assert_eq!(
            block_type(&scene, IVec3::new(cell.x, cell.y + 1, cell.z)),
            None,
            "{cell:?} is buried"
        );
        assert_eq!(
            ground_top(scene.config(), scene.world(), cell.x, cell.z),
            Some(cell.y)
        );
    }
    // The path reaches the door threshold: the door sits right behind it.
    assert_eq!(
        block_type(
            &scene,
            IVec3::new(
                HOUSE_DOOR_X,
                scene.house().floor_level() + 1,
                HOUSE_FOOTPRINT.max_z
            )
        ),
        Some(BlockType::WoodDoor)
    );
}

#[test]
fn the_path_is_connected_and_follows_the_relief_gently() {
    let scene = WorldScene::new();
    let path = scene.path();
    let mut climbs = 0;
    for pair in path.cells.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert_eq!(
            (a.x - b.x).abs() + (a.z - b.z).abs(),
            1,
            "{a:?} -> {b:?} is not adjacent"
        );
        assert!((a.y - b.y).abs() <= PATH_MAX_STEP, "{a:?} -> {b:?} jumps");
        if a.y != b.y {
            climbs += 1;
        }
    }
    // It really rises and falls with the terrain instead of being flat.
    assert!(climbs >= 2, "{climbs} height changes");
    let heights: HashSet<i32> = path.cells.iter().map(|c| c.y).collect();
    assert!(heights.len() >= 2);
}

#[test]
fn the_path_avoids_water_trees_and_the_house() {
    let scene = WorldScene::new();
    let basin = pond_basin_columns();
    for cell in scene.path().cells.iter().chain(scene.path().landing.iter()) {
        assert!(
            !basin.contains(&(cell.x, cell.z)),
            "path through the pond at {cell:?}"
        );
        assert!(
            !HOUSE_FOOTPRINT.contains(cell.x, cell.z),
            "path through the house at {cell:?}"
        );
        assert!(PATH_RESERVE.contains(cell.x, cell.z));
        for tree in scene.trees() {
            assert!(
                (tree.base.x, tree.base.z) != (cell.x, cell.z),
                "path through a trunk"
            );
            assert!(!tree.trunk.contains(&IVec3::new(cell.x, cell.y + 1, cell.z)));
        }
        for cell_water in &scene.pond().water {
            assert_ne!(cell, cell_water);
        }
    }
}

#[test]
fn the_path_is_one_to_two_blocks_wide_and_does_not_invade_the_surface() {
    let scene = WorldScene::new();
    let path = scene.path();
    let cells: HashSet<(i32, i32)> = path.cells.iter().map(|c| (c.x, c.z)).collect();
    // A one-block trail: every trail cell has at most two trail neighbors.
    for &(x, z) in &cells {
        let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|(dx, dz)| cells.contains(&(x + dx, z + dz)))
            .count();
        assert!(neighbors <= 2, "({x}, {z}) has {neighbors} trail neighbors");
    }
    assert!(path.landing.len() <= 3);
    let cobblestone = all_cobblestone(&scene);
    assert_eq!(cobblestone.len(), path.cells.len() + path.landing.len());
    assert!(
        cobblestone.len() < 40,
        "{} cobblestone blocks",
        cobblestone.len()
    );
    let manhattan =
        (DESCENT_ENDPOINT.0 - PATH_START.0).abs() + (DESCENT_ENDPOINT.1 - PATH_START.1).abs();
    assert!(path.cells.len() as i32 <= 2 * manhattan + 1);
}

#[test]
fn the_descent_endpoint_is_reserved_and_no_portal_exists_yet() {
    let scene = WorldScene::new();
    let (x, z) = DESCENT_ENDPOINT;
    let y = ground_top(scene.config(), scene.world(), x, z).unwrap();
    assert_eq!(
        block_type(&scene, IVec3::new(x, y, z)),
        Some(BlockType::Cobblestone)
    );
    assert!(!scene.path().landing.is_empty());
    // Nothing of Gate 13 is built: no stairs down, no portal, no Red-Black.
    let c = *scene.config();
    let b = scene.bounds();
    for zz in 0..c.depth {
        for xx in 0..c.width {
            for yy in b.min.y..=b.max.y + 16 {
                if let Some(t) = block_type(&scene, IVec3::new(xx, yy, zz)) {
                    assert!(
                        !matches!(
                            t,
                            BlockType::PortalFrameRedObsidian | BlockType::PortalCoreDarkCrimson
                        ),
                        "{t:?} at ({xx}, {yy}, {zz})"
                    );
                    assert!(
                        matches!(
                            t.family(),
                            scene::block_type::BlockFamily::OverworldTerrain
                                | scene::block_type::BlockFamily::OverworldArchitecture
                        ),
                        "{t:?} at ({xx}, {yy}, {zz})"
                    );
                }
            }
        }
    }
    // The endpoint stays on the surface: no staircase dug under it.
    assert!(block_type(&scene, IVec3::new(x, y - 1, z)).is_some());
}

#[test]
fn the_path_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.path(), b.path());
}
