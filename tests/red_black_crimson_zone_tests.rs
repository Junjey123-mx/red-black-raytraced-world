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
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    crimson_diamond_material_id, crimson_heart_material_id, crying_obsidian_crimson_material_id,
    nether_wart_block_material_id, red_black_deepslate_bricks_crimson_material_id,
};
use scene::red_black_maze::{Family, FamilyLayout, family_zone};
use scene::texture_manager::TextureManager;
use scene::world::{WorldScene, WorldTextures, world_materials};
use std::collections::{HashMap, HashSet, VecDeque};

fn crimson(scene: &WorldScene) -> &FamilyLayout {
    scene
        .families()
        .iter()
        .find(|f| f.family == Family::Crimson)
        .expect("crimson family")
}

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn all_five_crimson_block_types_are_present() {
    let scene = WorldScene::new();
    let f = crimson(&scene);
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for cell in f.cells() {
        if scene.transitions().bridged.contains(&cell) {
            continue;
        }
        *counts.entry(block_type(&scene, cell).unwrap()).or_insert(0) += 1;
    }
    for t in [
        BlockType::CrimsonHeart,
        BlockType::CrimsonDiamond,
        BlockType::CryingObsidianCrimson,
        BlockType::NetherWartBlock,
        BlockType::RedBlackDeepslateBricksCrimson,
    ] {
        assert!(
            counts.get(&t).copied().unwrap_or(0) >= 3,
            "{t:?}: {counts:?}"
        );
    }
    assert!(
        !f.symbols.is_empty()
            && !f.terraces.is_empty()
            && !f.ground.is_empty()
            && !f.accents.is_empty()
    );
}

#[test]
fn every_crimson_block_is_down_oriented_inside_the_crimson_sector() {
    let scene = WorldScene::new();
    let f = crimson(&scene);
    let r = scene.rhombus();
    for cell in f.cells() {
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        assert_eq!(
            family_zone(scene.config(), r, cell.x, cell.z),
            Family::Crimson
        );
        assert!(cell.y < r.shelf_y, "{cell:?} is not in the lower world");
        assert!(!r.cutaway.contains_column(cell.x, cell.z));
    }
}

#[test]
fn the_crimson_zone_is_part_of_the_lower_world_not_an_island() {
    let scene = WorldScene::new();
    let f = crimson(&scene);
    // Every placed cell touches the mass: a surface cell it replaced or the
    // surface cell it hangs from.
    for cell in f.cells() {
        let touching = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, 0, 1), (0, 0, -1)]
            .iter()
            .any(|(dx, dy, dz)| {
                scene
                    .world()
                    .contains(IVec3::new(cell.x + dx, cell.y + dy, cell.z + dz))
            });
        assert!(touching, "{cell:?} hangs in the air");
    }
    // Flood fill from one crimson cell through the world reaches a portal
    // frame cell: the zone is connected to the rest of the diamond.
    let start = f.symbols[0];
    let goal: HashSet<IVec3> = scene.portal().frame.iter().copied().collect();
    let mut seen: HashSet<IVec3> = HashSet::new();
    let mut queue = VecDeque::from([start]);
    let mut reached = false;
    while let Some(c) = queue.pop_front() {
        if goal.contains(&c) {
            reached = true;
            break;
        }
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
            if scene.world().contains(n) && !seen.contains(&n) {
                queue.push_back(n);
            }
        }
    }
    assert!(reached, "the crimson zone is not connected to the portal");
}

#[test]
fn symbols_face_the_inverted_world_and_the_materials_are_the_catalog_ones() {
    let scene = WorldScene::new();
    let f = crimson(&scene);
    for cell in f
        .symbols
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        let b = scene.world().get(*cell).unwrap();
        assert!(matches!(
            b.block_type(),
            BlockType::CrimsonHeart | BlockType::CrimsonDiamond
        ));
        let expected = if b.block_type() == BlockType::CrimsonHeart {
            crimson_heart_material_id()
        } else {
            crimson_diamond_material_id()
        };
        assert_eq!(b.material_id(), expected);
        // The symbol is the lowest cell of its column: visible from -Y.
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
    }
    for cell in f
        .ground
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            nether_wart_block_material_id()
        );
    }
    for cell in f
        .accents
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            crying_obsidian_crimson_material_id()
        );
    }
    for cell in f
        .terraces
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            red_black_deepslate_bricks_crimson_material_id()
        );
    }
    let src = std::fs::read_to_string("src/scene/red_black_maze.rs").unwrap();
    assert!(
        !src.contains("Material::new"),
        "no material is defined in the lower world"
    );
}

#[test]
fn wart_does_not_emit_and_crying_emission_is_localized() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let lib = world_materials(&textures);
    let wart = lib.get(nether_wart_block_material_id()).unwrap();
    assert_eq!(wart.emission_strength, 0.0);
    assert!(wart.emissive_texture.is_none());
    let crying = lib.get(crying_obsidian_crimson_material_id()).unwrap();
    assert!(crying.emission_strength > 0.0);
    assert!(
        crying.emissive_texture.is_some(),
        "emission comes from a mask, not the whole block"
    );
    let heart = lib.get(crimson_heart_material_id()).unwrap();
    assert!(heart.emissive_texture.is_some());
}

#[test]
fn the_crimson_family_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(crimson(&a), crimson(&b));
}
