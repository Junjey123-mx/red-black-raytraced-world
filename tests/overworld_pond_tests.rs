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
use scene::material_gallery::water_material_id;
use scene::overworld::{POND_MAX_DEPTH, POND_WATER_LEVEL, pond_basin_columns};
use scene::overworld_blocks::sand_material_id;
use scene::terrain::generator::{column_bottom, footprint_contains};
use scene::world::WorldScene;
use std::collections::{HashSet, VecDeque};

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

fn cells_of(scene: &WorldScene, wanted: BlockType) -> Vec<IVec3> {
    let c = *scene.config();
    let b = scene.bounds();
    let mut out = Vec::new();
    for z in 0..c.depth {
        for x in 0..c.width {
            for y in b.min.y..=b.max.y + 16 {
                let cell = IVec3::new(x, y, z);
                if block_type(scene, cell) == Some(wanted) {
                    out.push(cell);
                }
            }
        }
    }
    out
}

#[test]
fn the_pond_holds_water_with_the_definitive_water_material() {
    let scene = WorldScene::new();
    let water = cells_of(&scene, BlockType::Water);
    assert!(water.len() >= 15, "{} water cells", water.len());
    assert_eq!(water.len(), scene.pond().water.len());
    for cell in &water {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            water_material_id()
        );
        assert!(scene.pond().water.contains(cell));
    }
}

#[test]
fn the_water_is_one_connected_pool_at_one_level() {
    let scene = WorldScene::new();
    let water: HashSet<IVec3> = cells_of(&scene, BlockType::Water).into_iter().collect();
    let surface: Vec<&IVec3> = water.iter().filter(|c| c.y == POND_WATER_LEVEL).collect();
    assert!(!surface.is_empty());
    assert!(water.iter().all(|c| c.y <= POND_WATER_LEVEL));
    // Flood fill from one surface cell reaches every water cell.
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([*surface[0]]);
    while let Some(c) = queue.pop_front() {
        if !seen.insert(c) {
            continue;
        }
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            if water.contains(&n) && !seen.contains(&n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(
        seen.len(),
        water.len(),
        "the water is split into several pools"
    );
}

#[test]
fn the_basin_is_one_to_two_blocks_deep_and_nothing_floats() {
    let scene = WorldScene::new();
    let mut depths = HashSet::new();
    for (x, z) in pond_basin_columns() {
        let column: Vec<i32> = (POND_WATER_LEVEL - POND_MAX_DEPTH..=POND_WATER_LEVEL)
            .filter(|&y| block_type(&scene, IVec3::new(x, y, z)) == Some(BlockType::Water))
            .collect();
        assert!(!column.is_empty(), "dry basin column ({x}, {z})");
        assert!(column.len() as i32 <= POND_MAX_DEPTH);
        assert!(column.contains(&POND_WATER_LEVEL));
        depths.insert(column.len());
        // Solid floor under the deepest water cell, water or air above.
        let floor = IVec3::new(x, column[0] - 1, z);
        let floor_type = block_type(&scene, floor).expect("floor");
        assert!(
            matches!(floor_type, BlockType::Sand | BlockType::Stone),
            "{floor_type:?}"
        );
        assert_eq!(
            block_type(&scene, IVec3::new(x, POND_WATER_LEVEL + 1, z)),
            None
        );
        // Under the floor the geological mass continues unbroken.
        for y in column_bottom(scene.config(), x, z)..floor.y {
            assert!(
                scene.world().contains(IVec3::new(x, y, z)),
                "hole under the pond at ({x}, {y}, {z})"
            );
        }
    }
    assert!(depths.contains(&1) && depths.contains(&2), "{depths:?}");
}

#[test]
fn sand_lines_part_of_the_shore_and_grass_stays_dominant() {
    let scene = WorldScene::new();
    let basin = pond_basin_columns();
    let sand_top: Vec<IVec3> = scene.pond().sand.clone();
    assert!(sand_top.len() >= 8, "{} shore cells", sand_top.len());
    for cell in &sand_top {
        assert_eq!(block_type(&scene, *cell), Some(BlockType::Sand));
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            sand_material_id()
        );
        assert_eq!(scene.column_top(cell.x, cell.z), Some(cell.y));
        let touches_basin = (-1..=1)
            .flat_map(|dz| (-1..=1).map(move |dx| (cell.x + dx, cell.z + dz)))
            .any(|c| basin.contains(&c));
        assert!(touches_basin, "sand away from the pond at {cell:?}");
    }
    // Sand is localized: the grass surface remains far larger.
    let c = *scene.config();
    let mut grass_tops = 0;
    let mut sand_tops = 0;
    for z in 0..c.depth {
        for x in 0..c.width {
            if let Some(y) = scene.column_top(x, z) {
                match block_type(&scene, IVec3::new(x, y, z)) {
                    Some(BlockType::Grass) => grass_tops += 1,
                    Some(BlockType::Sand) => sand_tops += 1,
                    _ => {}
                }
            }
        }
    }
    assert!(
        grass_tops > sand_tops * 6,
        "grass {grass_tops} vs sand {sand_tops}"
    );
    // Not every bank cell is sand: some grass reaches the water.
    let bank_grass = basin
        .iter()
        .flat_map(|&(x, z)| (-1..=1).flat_map(move |dz| (-1..=1).map(move |dx| (x + dx, z + dz))))
        .filter(|c| !basin.contains(c))
        .filter(|&(x, z)| {
            scene
                .column_top(x, z)
                .map(|y| block_type(&scene, IVec3::new(x, y, z)))
                == Some(Some(BlockType::Grass))
        })
        .count();
    assert!(bank_grass > 0);
}

#[test]
fn the_pond_sits_inside_the_footprint_below_its_banks() {
    let scene = WorldScene::new();
    let c = *scene.config();
    for (x, z) in pond_basin_columns() {
        assert!(footprint_contains(&c, x, z));
        assert_eq!(scene.column_top(x, z), Some(POND_WATER_LEVEL));
    }
    // The terrain around the basin is higher than the water: a real basin.
    let basin = pond_basin_columns();
    let mut higher = 0;
    let mut bank = 0;
    for &(x, z) in &basin {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (x + dx, z + dz);
            if !basin.contains(&n) {
                bank += 1;
                if scene.column_top(n.0, n.1).unwrap_or(i32::MIN) > POND_WATER_LEVEL {
                    higher += 1;
                }
            }
        }
    }
    assert_eq!(
        higher, bank,
        "{higher} of {bank} bank cells rise above the water"
    );
}

#[test]
fn the_pond_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.pond(), b.pond());
    assert_eq!(
        cells_of(&a, BlockType::Water),
        cells_of(&b, BlockType::Water)
    );
    assert_eq!(cells_of(&a, BlockType::Sand), cells_of(&b, BlockType::Sand));
}
