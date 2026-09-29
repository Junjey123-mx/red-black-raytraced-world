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
use scene::overworld::{
    HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL, HOUSE_FOOTPRINT, HOUSE_WALL_HEIGHT, HOUSE_WINDOWS,
    HouseLayout, build_house, carve_pond, column_top, pond_basin_columns,
};
use scene::overworld_blocks::{
    double_wood_slab_material_id, log_material_id, wood_planks_material_id,
};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::{column_bottom, footprint_contains, generate_terrain};
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;

/// The house structure stage: terrain, pond and `build_house`, before the
/// exterior (roof, door, glass, fence, lamps) is added.
struct Structure {
    config: TerrainConfig,
    house: HouseLayout,
    world: VoxelWorld,
}

impl Structure {
    fn new() -> Self {
        let config = TerrainConfig::official();
        let mut world = VoxelWorld::new();
        generate_terrain(&config, &mut world);
        carve_pond(&config, &mut world);
        let house = build_house(&config, &mut world);
        Self {
            config,
            house,
            world,
        }
    }

    fn config(&self) -> &TerrainConfig {
        &self.config
    }

    fn house(&self) -> &HouseLayout {
        &self.house
    }

    fn world(&self) -> &VoxelWorld {
        &self.world
    }

    fn column_top(&self, x: i32, z: i32) -> Option<i32> {
        column_top(
            &self.world,
            x,
            z,
            self.config.max_surface_height() + 16,
            self.config.deepslate_level - 8,
        )
    }
}

fn block_type(scene: &Structure, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

fn house_volume(scene: &Structure) -> Vec<(IVec3, BlockType)> {
    let r = HOUSE_FOOTPRINT;
    let mut out = Vec::new();
    for z in r.min_z..=r.max_z {
        for x in r.min_x..=r.max_x {
            for y in HOUSE_FLOOR_LEVEL..=HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT {
                let cell = IVec3::new(x, y, z);
                if let Some(b) = block_type(scene, cell) {
                    out.push((cell, b));
                }
            }
        }
    }
    out
}

#[test]
fn the_house_stands_inside_the_terrain_on_an_allowed_footprint() {
    let scene = Structure::new();
    let r = HOUSE_FOOTPRINT;
    assert!(
        matches!((r.width(), r.depth()), (6, 5) | (7, 5)),
        "{}x{}",
        r.width(),
        r.depth()
    );
    for z in r.min_z..=r.max_z {
        for x in r.min_x..=r.max_x {
            assert!(footprint_contains(scene.config(), x, z));
        }
    }
    assert_eq!(scene.house().footprint(), r);
    assert_eq!(scene.house().floor_level(), HOUSE_FLOOR_LEVEL);
}

#[test]
fn the_floor_is_planks_supported_by_solid_ground() {
    let scene = Structure::new();
    let r = HOUSE_FOOTPRINT;
    assert_eq!(scene.house().floor.len(), (r.width() * r.depth()) as usize);
    for cell in &scene.house().floor {
        assert_eq!(cell.y, HOUSE_FLOOR_LEVEL);
        let block = scene.world().get(*cell).unwrap();
        assert_eq!(block.block_type(), BlockType::WoodPlanks);
        assert_eq!(block.material_id(), wood_planks_material_id());
        // Solid all the way down: the plateau was leveled, not undermined.
        let bottom = column_bottom(scene.config(), cell.x, cell.z);
        for y in bottom..cell.y {
            let below = block_type(&scene, IVec3::new(cell.x, y, cell.z));
            assert!(
                below.is_some(),
                "hole under the floor at ({}, {y}, {})",
                cell.x,
                cell.z
            );
            assert!(
                matches!(
                    below,
                    Some(BlockType::Dirt | BlockType::Stone | BlockType::Deepslate)
                ),
                "{below:?} under the floor"
            );
        }
    }
}

#[test]
fn the_leveling_stays_local_to_the_footprint() {
    let scene = Structure::new();
    let r = HOUSE_FOOTPRINT;
    // Outside the house the relief is untouched: several heights around.
    let mut heights = std::collections::BTreeSet::new();
    for z in r.min_z - 2..=r.max_z + 2 {
        for x in r.min_x - 2..=r.max_x + 2 {
            if !r.contains(x, z) && footprint_contains(scene.config(), x, z) {
                heights.insert(scene.column_top(x, z).unwrap());
            }
        }
    }
    assert!(heights.len() >= 2, "{heights:?}");
    assert!(
        heights.contains(&(HOUSE_FLOOR_LEVEL - 1)) || heights.contains(&(HOUSE_FLOOR_LEVEL + 1))
    );
}

#[test]
fn the_walls_are_three_courses_of_approved_wood_with_log_corners() {
    let scene = Structure::new();
    let r = HOUSE_FOOTPRINT;
    let house = scene.house();
    assert_eq!(HOUSE_WALL_HEIGHT, 3);
    let perimeter = 2 * (r.width() + r.depth()) - 4;
    let openings = house.door_opening.len() + house.window_openings.len();
    assert_eq!(
        house.walls.len() + house.corners.len() + openings,
        (perimeter * HOUSE_WALL_HEIGHT) as usize
    );
    assert_eq!(house.corners.len(), 4 * HOUSE_WALL_HEIGHT as usize);
    for cell in &house.corners {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Log);
        assert_eq!(b.material_id(), log_material_id());
    }
    let mut slabs = 0;
    for cell in &house.walls {
        assert!(cell.y > HOUSE_FLOOR_LEVEL && cell.y <= HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT);
        let b = scene.world().get(*cell).unwrap();
        match b.block_type() {
            BlockType::WoodPlanks => assert_eq!(b.material_id(), wood_planks_material_id()),
            BlockType::DoubleWoodSlab => {
                assert_eq!(b.material_id(), double_wood_slab_material_id());
                assert_eq!(cell.y, HOUSE_FLOOR_LEVEL + 1);
                slabs += 1;
            }
            other => panic!("{other:?} in a wall"),
        }
    }
    assert!(slabs > 0);
    // Inside the walls there is room: the interior is air above the floor.
    for z in r.min_z + 1..r.max_z {
        for x in r.min_x + 1..r.max_x {
            for course in 1..=HOUSE_WALL_HEIGHT {
                assert_eq!(
                    block_type(&scene, IVec3::new(x, HOUSE_FLOOR_LEVEL + course, z)),
                    None
                );
            }
        }
    }
}

#[test]
fn the_entrance_and_windows_are_open() {
    let scene = Structure::new();
    let house = scene.house();
    let door = [
        IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 1, HOUSE_FOOTPRINT.max_z),
        IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 2, HOUSE_FOOTPRINT.max_z),
    ];
    assert_eq!(house.door_opening, door);
    for cell in door {
        assert_eq!(block_type(&scene, cell), None);
    }
    // Lintel above the door, floor under it.
    assert!(
        block_type(
            &scene,
            IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 3, HOUSE_FOOTPRINT.max_z)
        )
        .is_some()
    );
    assert_eq!(
        block_type(
            &scene,
            IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL, HOUSE_FOOTPRINT.max_z)
        ),
        Some(BlockType::WoodPlanks)
    );
    assert_eq!(house.window_openings.len(), HOUSE_WINDOWS.len());
    for (x, z) in HOUSE_WINDOWS {
        let cell = IVec3::new(x, HOUSE_FLOOR_LEVEL + 2, z);
        assert!(house.window_openings.contains(&cell));
        assert_eq!(block_type(&scene, cell), None);
        // Windows sit in walls, with wall above and below.
        assert!(block_type(&scene, IVec3::new(x, HOUSE_FLOOR_LEVEL + 1, z)).is_some());
        assert!(block_type(&scene, IVec3::new(x, HOUSE_FLOOR_LEVEL + 3, z)).is_some());
    }
}

#[test]
fn the_house_does_not_touch_the_pond_or_the_trees() {
    let scene = WorldScene::new();
    let r = HOUSE_FOOTPRINT;
    for (x, z) in pond_basin_columns() {
        assert!(!r.grown(1).contains(x, z));
    }
    for tree in scene.trees() {
        for cell in tree.trunk.iter().chain(tree.leaves.iter()) {
            assert!(
                !r.contains(cell.x, cell.z),
                "tree cell {cell:?} inside the house"
            );
        }
    }
}

#[test]
fn only_authorized_materials_make_up_the_structure() {
    let scene = Structure::new();
    for (cell, b) in house_volume(&scene) {
        assert!(
            matches!(
                b,
                BlockType::WoodPlanks | BlockType::DoubleWoodSlab | BlockType::Log
            ),
            "{b:?} at {cell:?}"
        );
    }
}

#[test]
fn the_house_is_deterministic() {
    let a = Structure::new();
    let b = Structure::new();
    assert_eq!(a.house(), b.house());
    assert_eq!(house_volume(&a), house_volume(&b));
}
