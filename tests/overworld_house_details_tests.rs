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
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::light::Light;
use scene::material_gallery::{glass_material_id, redstone_lamp_material_id};
use scene::orientation::Orientation;
use scene::overworld::{
    HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL, HOUSE_FOOTPRINT, HOUSE_LAMPS, HOUSE_RIDGE_Z,
    HOUSE_ROOF_OVERHANG, HOUSE_WALL_HEIGHT, HOUSE_WINDOWS, roof_stair_orientation,
};
use scene::overworld_blocks::{
    fence_material_id, wood_door_bottom_material_id, wood_door_top_material_id,
    wood_stairs_material_id,
};
use scene::terrain::generator::footprint_contains;
use scene::world::{WorldScene, world_lights};
use std::collections::HashMap;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

fn count_types(scene: &WorldScene) -> HashMap<BlockType, usize> {
    let c = *scene.config();
    let b = scene.bounds();
    let mut counts = HashMap::new();
    for z in -2..c.depth + 2 {
        for x in -2..c.width + 2 {
            for y in b.min.y..=b.max.y + 16 {
                if let Some(t) = block_type(scene, IVec3::new(x, y, z)) {
                    *counts.entry(t).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

#[test]
fn the_roof_is_made_of_oriented_stairs_rising_to_a_ridge() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    let stairs: Vec<_> = ext
        .roof
        .iter()
        .filter(|(_, t, _)| *t == BlockType::WoodStairs)
        .collect();
    assert!(stairs.len() >= 20, "{} stairs", stairs.len());
    let wall_top = HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT;
    for (cell, t, orientation) in &ext.roof {
        assert!(cell.y > wall_top && cell.y <= wall_top + 3);
        let block = scene.world().get(*cell).unwrap();
        assert_eq!(block.block_type(), *t);
        assert_eq!(block.orientation(), *orientation);
        if *t == BlockType::WoodStairs {
            assert_eq!(block.material_id(), wood_stairs_material_id());
            // Eaves face away from the ridge so the raised half climbs it.
            assert_eq!(*orientation, roof_stair_orientation(cell.z));
            assert!(matches!(
                orientation,
                Orientation::North | Orientation::South
            ));
            let expected = if cell.z > HOUSE_RIDGE_Z {
                Orientation::South
            } else {
                Orientation::North
            };
            assert_eq!(*orientation, expected);
        }
    }
    // Each layer is narrower than the one below and the ridge is capped.
    let width_at = |y: i32| {
        ext.roof
            .iter()
            .filter(|(c, _, _)| c.y == y && c.x == HOUSE_FOOTPRINT.min_x)
            .count()
    };
    assert!(width_at(wall_top + 1) > width_at(wall_top + 2));
    assert!(width_at(wall_top + 2) > width_at(wall_top + 3));
    assert_eq!(
        block_type(
            &scene,
            IVec3::new(HOUSE_DOOR_X, wall_top + 3, HOUSE_RIDGE_Z)
        ),
        Some(BlockType::DoubleWoodSlab)
    );
    // The overhang really hangs past the walls.
    assert!(
        block_type(
            &scene,
            IVec3::new(
                HOUSE_FOOTPRINT.min_x - HOUSE_ROOF_OVERHANG,
                wall_top + 1,
                HOUSE_RIDGE_Z
            )
        )
        .is_some()
    );
    // The roof seals the house: nothing looks down into the interior.
    for z in HOUSE_FOOTPRINT.min_z..=HOUSE_FOOTPRINT.max_z {
        for x in HOUSE_FOOTPRINT.min_x..=HOUSE_FOOTPRINT.max_x {
            assert!(
                block_type(&scene, IVec3::new(x, wall_top + 1, z)).is_some(),
                "open roof at ({x}, {z})"
            );
        }
    }
}

#[test]
fn the_door_fills_the_entrance_standing_on_the_floor() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    let z = HOUSE_FOOTPRINT.max_z;
    assert_eq!(ext.door.len(), 2);
    let bottom = IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 1, z);
    let top = IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 2, z);
    assert_eq!(ext.door, vec![bottom, top]);
    let b = scene.world().get(bottom).unwrap();
    let t = scene.world().get(top).unwrap();
    assert_eq!(b.block_type(), BlockType::WoodDoor);
    assert_eq!(t.block_type(), BlockType::WoodDoor);
    assert_eq!(b.material_id(), wood_door_bottom_material_id());
    assert_eq!(t.material_id(), wood_door_top_material_id());
    assert_eq!(b.orientation(), Orientation::South);
    assert_eq!(t.orientation(), Orientation::South);
    // Not floating: planks under it, wall over it.
    assert_eq!(
        block_type(&scene, IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL, z)),
        Some(BlockType::WoodPlanks)
    );
    assert!(block_type(&scene, IVec3::new(HOUSE_DOOR_X, HOUSE_FLOOR_LEVEL + 3, z)).is_some());
    // The leaf sits against the outer (south) face of its cell.
    if let BlockGeometry::Prism(leaf) = block_geometry(BlockType::WoodDoor, Orientation::South) {
        assert!(leaf.min().z > 0.75);
    } else {
        panic!("the door is a single thin prism");
    }
}

#[test]
fn every_window_holds_glass() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    assert_eq!(ext.glass.len(), HOUSE_WINDOWS.len());
    for (x, z) in HOUSE_WINDOWS {
        let cell = IVec3::new(x, HOUSE_FLOOR_LEVEL + 2, z);
        assert!(ext.glass.contains(&cell));
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Glass);
        assert_eq!(b.material_id(), glass_material_id());
    }
}

#[test]
fn a_restrained_fence_stands_on_the_porch_ground() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    assert!(!ext.fence.is_empty());
    assert!(ext.fence.len() <= 8, "{} fence blocks", ext.fence.len());
    for cell in &ext.fence {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::Fence);
        assert_eq!(b.material_id(), fence_material_id());
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "floating fence at {cell:?}"
        );
        assert!(
            block_type(&scene, IVec3::new(cell.x, cell.y + 1, cell.z)).is_none()
                || cell.y + 1 > HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT
        );
        assert_ne!(cell.x, HOUSE_DOOR_X, "the fence blocks the door");
    }
}

#[test]
fn two_lit_redstone_lamps_flank_the_entrance() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    assert_eq!(ext.lamps.len(), 2);
    assert_eq!(HOUSE_LAMPS.len(), 2);
    for cell in &ext.lamps {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::RedstoneLampLit);
        assert_eq!(b.material_id(), redstone_lamp_material_id());
        assert_eq!((cell.x - HOUSE_DOOR_X).abs(), 1);
        assert_eq!(cell.z, HOUSE_FOOTPRINT.max_z);
    }
    // Daylight stays dominant: the world's only light is still the sun.
    let lights = world_lights();
    assert_eq!(lights.len(), 1);
    assert!(matches!(lights[0], Light::Directional(_)));
}

#[test]
fn the_finished_house_uses_only_authorized_blocks_inside_the_terrain() {
    let scene = WorldScene::new();
    let r = HOUSE_FOOTPRINT.grown(HOUSE_ROOF_OVERHANG);
    for z in r.min_z..=r.max_z + 1 {
        for x in r.min_x..=r.max_x {
            assert!(footprint_contains(scene.config(), x, z));
            for y in HOUSE_FLOOR_LEVEL..=HOUSE_FLOOR_LEVEL + HOUSE_WALL_HEIGHT + 3 {
                if let Some(t) = block_type(&scene, IVec3::new(x, y, z)) {
                    assert!(
                        matches!(
                            t,
                            BlockType::WoodPlanks
                                | BlockType::DoubleWoodSlab
                                | BlockType::WoodStairs
                                | BlockType::Fence
                                | BlockType::Glass
                                | BlockType::WoodDoor
                                | BlockType::RedstoneLampLit
                                | BlockType::Log
                                | BlockType::Grass
                                | BlockType::Dirt
                                | BlockType::Leaves
                        ),
                        "{t:?} at ({x}, {y}, {z})"
                    );
                }
            }
        }
    }
    let counts = count_types(&scene);
    for t in [
        BlockType::WoodStairs,
        BlockType::WoodDoor,
        BlockType::Glass,
        BlockType::Fence,
        BlockType::RedstoneLampLit,
        BlockType::DoubleWoodSlab,
        BlockType::WoodPlanks,
    ] {
        assert!(counts.get(&t).copied().unwrap_or(0) > 0, "no {t:?}");
    }
    assert_eq!(counts[&BlockType::RedstoneLampLit], 2);
    assert_eq!(counts[&BlockType::WoodDoor], 2);
}

#[test]
fn the_partial_geometries_are_valid_in_their_placed_orientations() {
    let scene = WorldScene::new();
    let ext = scene.house_exterior();
    let placed = ext
        .roof
        .iter()
        .filter(|(_, t, _)| *t == BlockType::WoodStairs)
        .map(|(c, _, _)| *c)
        .chain(ext.door.iter().copied())
        .chain(ext.fence.iter().copied());
    for cell in placed {
        let b = scene.world().get(cell).unwrap();
        let geometry = block_geometry(b.block_type(), b.orientation());
        assert!(
            !geometry.is_full_cube(),
            "{:?} is a full cube",
            b.block_type()
        );
        for part in geometry.parts() {
            assert!(
                part.is_within_unit_cell(),
                "{:?} at {cell:?}",
                b.block_type()
            );
        }
    }
}
