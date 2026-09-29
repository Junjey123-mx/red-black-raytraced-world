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
use scene::catalog::{CatalogTextures, catalog_materials};
use scene::orientation::Orientation;
use scene::overworld_blocks::portal_frame_material_id;
use scene::texture_manager::TextureManager;
use scene::world::{WorldScene, WorldTextures, world_materials};
use std::collections::HashSet;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn the_frame_is_red_obsidian_at_the_anchor() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let p = r.portal;
    let frame = &scene.portal().frame;
    let expected = (p.width() * p.height() - (p.width() - 2) * (p.height() - 2)) as usize;
    assert_eq!(frame.len(), expected);
    assert_eq!(frame.len(), 18);
    for cell in frame {
        assert_eq!(cell.z, p.wall_z);
        assert!(p.is_frame(cell.x, cell.y));
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::PortalFrameRedObsidian);
        assert_eq!(b.material_id(), portal_frame_material_id());
        assert_eq!(b.orientation(), Orientation::Up);
    }
    // Centered on the waist row and inside the diamond.
    assert!((p.min_y..=p.max_y).contains(&r.waist_y));
    for cell in frame {
        assert!(r.contains(scene.config(), cell.x, cell.y, cell.z));
    }
}

#[test]
fn the_opening_is_empty_and_the_frame_is_continuous() {
    let scene = WorldScene::new();
    let p = scene.rhombus().portal;
    assert_eq!(
        scene.portal().opening.len(),
        ((p.width() - 2) * (p.height() - 2)) as usize
    );
    // The frame stage leaves the opening empty; the core stage fills it
    // with the membrane, never with frame or rock.
    for cell in &scene.portal().opening {
        assert!(p.is_opening(cell.x, cell.y));
        assert!(
            matches!(
                block_type(&scene, *cell),
                None | Some(BlockType::PortalCoreDarkCrimson)
            ),
            "{cell:?} should be open"
        );
    }
    let mut fresh = scene::voxel_world::VoxelWorld::new();
    let stage = scene::portal::build_portal_frame(scene.rhombus(), &mut fresh);
    for cell in &stage.opening {
        assert!(!fresh.contains(*cell));
    }
    // Every frame cell touches another frame cell: one closed ring.
    let cells: HashSet<(i32, i32)> = scene.portal().frame.iter().map(|c| (c.x, c.y)).collect();
    for &(x, y) in &cells {
        let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|(dx, dy)| cells.contains(&(x + dx, y + dy)))
            .count();
        assert!(neighbors >= 2, "frame cell ({x}, {y}) is loose");
    }
    // Ring: a walk around it returns to the start.
    let start = *cells.iter().min().unwrap();
    let mut seen = HashSet::new();
    let mut stack = vec![start];
    while let Some((x, y)) = stack.pop() {
        if !seen.insert((x, y)) {
            continue;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if cells.contains(&(x + dx, y + dy)) {
                stack.push((x + dx, y + dy));
            }
        }
    }
    assert_eq!(seen.len(), cells.len());
}

#[test]
fn the_frame_stays_clear_of_the_descent_and_inside_the_walls() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    for (tread, _) in &scene.upper_descent().treads {
        assert!(!scene.portal().frame.contains(tread));
        assert!(!scene.portal().opening.contains(tread));
    }
    for cell in &scene.upper_descent().approach {
        assert!(!scene.portal().frame.contains(cell));
    }
    // Wall behind the frame plane stays solid rock; in front is the cut.
    let p = r.portal;
    for cell in &scene.portal().frame {
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y, p.wall_z - 1))
        );
        assert!(!r.is_cut(cell.x, cell.y, cell.z));
    }
    for cell in &scene.portal().opening {
        assert!(r.cutaway.contains_column(cell.x, cell.z + 1));
    }
}

#[test]
fn the_frame_material_is_the_opaque_non_emissive_catalog_one() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let world_lib = world_materials(&textures);
    let frame = world_lib
        .get(portal_frame_material_id())
        .expect("frame material");
    assert_eq!(frame.transparency, 0.0);
    assert_eq!(frame.emission_strength, 0.0);
    assert!(frame.emissive_texture.is_none());
    assert!(frame.reflectivity < 0.1);
    // Exactly the catalog's material: same id, same profile, no duplicate.
    let mut manager2 = TextureManager::new();
    let catalog_textures = CatalogTextures::load(
        &mut manager2,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let catalog_lib = catalog_materials(&catalog_textures);
    let catalog_frame = catalog_lib.get(portal_frame_material_id()).unwrap();
    assert_eq!(frame.albedo, catalog_frame.albedo);
    assert_eq!(frame.specular, catalog_frame.specular);
    assert_eq!(frame.shininess, catalog_frame.shininess);
    assert_eq!(frame.reflectivity, catalog_frame.reflectivity);
    let src = std::fs::read_to_string("src/scene/portal.rs").unwrap();
    assert!(!src.contains("Material::new"));
}

#[test]
fn the_frame_build_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.portal().frame, b.portal().frame);
    assert_eq!(a.portal().opening, b.portal().opening);
    assert_eq!(a.portal().frame.len(), 18);
}
