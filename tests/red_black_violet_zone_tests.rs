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
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    amethyst_cluster_material_id, budding_amethyst_material_id, crying_obsidian_violet_material_id,
    mycelium_material_id, red_black_deepslate_bricks_violet_material_id,
};
use scene::red_black_maze::{Family, FamilyLayout, family_zone};
use scene::texture_manager::TextureManager;
use scene::world::{WorldScene, WorldTextures, world_lights, world_materials};
use std::collections::{HashMap, HashSet, VecDeque};

fn violet(scene: &WorldScene) -> &FamilyLayout {
    scene
        .families()
        .iter()
        .find(|f| f.family == Family::Violet)
        .expect("violet family")
}

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn the_four_violet_suits_and_every_violet_block_are_present() {
    let scene = WorldScene::new();
    let f = violet(&scene);
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for cell in f.cells() {
        if scene.transitions().bridged.contains(&cell) {
            continue;
        }
        *counts.entry(block_type(&scene, cell).unwrap()).or_insert(0) += 1;
    }
    for t in [
        BlockType::PurpleHeart,
        BlockType::PurpleDiamond,
        BlockType::PurpleClub,
        BlockType::PurpleSpade,
        BlockType::Mycelium,
        BlockType::BuddingAmethyst,
        BlockType::AmethystCluster,
        BlockType::CryingObsidianViolet,
        BlockType::RedBlackDeepslateBricksViolet,
    ] {
        assert!(
            counts.get(&t).copied().unwrap_or(0) >= 2,
            "{t:?}: {counts:?}"
        );
    }
}

#[test]
fn clusters_grow_downward_from_budding_amethyst() {
    let scene = WorldScene::new();
    let f = violet(&scene);
    let clusters: Vec<IVec3> = f.structures.iter().copied().collect();
    assert!(!clusters.is_empty());
    for cell in &clusters {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::AmethystCluster);
        assert_eq!(b.material_id(), amethyst_cluster_material_id());
        assert_eq!(b.orientation(), Orientation::Down);
        let host = scene
            .world()
            .get(IVec3::new(cell.x, cell.y + 1, cell.z))
            .unwrap();
        assert_eq!(host.block_type(), BlockType::BuddingAmethyst);
        assert_eq!(host.material_id(), budding_amethyst_material_id());
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
    }
    // Down growth: the crystals' base touches the top face, tips hang low.
    match block_geometry(BlockType::AmethystCluster, Orientation::Down) {
        BlockGeometry::Composite(parts) => {
            assert!(
                parts.iter().all(|p| p.max().y > 0.99),
                "every crystal is rooted at +Y"
            );
            assert!(
                parts.iter().any(|p| p.min().y < 0.2),
                "the tallest crystal reaches down"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn mycelium_ground_faces_down_and_the_zone_is_connected() {
    let scene = WorldScene::new();
    let f = violet(&scene);
    let r = scene.rhombus();
    let mut mycelium = 0;
    for cell in f
        .ground
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        if b.block_type() == BlockType::Mycelium {
            mycelium += 1;
            assert_eq!(b.material_id(), mycelium_material_id());
            assert!(
                !scene
                    .world()
                    .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
            );
        }
    }
    assert!(mycelium >= 5);
    for cell in f.cells() {
        assert_eq!(
            family_zone(scene.config(), r, cell.x, cell.z),
            Family::Violet
        );
        assert!(cell.y < r.shelf_y);
    }
    let goal: HashSet<IVec3> = scene.portal().frame.iter().copied().collect();
    let mut seen: HashSet<IVec3> = HashSet::new();
    let mut queue = VecDeque::from([f.symbols[0]]);
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
    assert!(reached);
}

#[test]
fn violet_emission_stays_controlled() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let lib = world_materials(&textures);
    let cluster = lib.get(amethyst_cluster_material_id()).unwrap();
    assert!(cluster.emission_strength > 0.0 && cluster.emission_strength < 1.0);
    let budding = lib.get(budding_amethyst_material_id()).unwrap();
    assert!(budding.emission_strength <= 0.2);
    let mycelium = lib.get(mycelium_material_id()).unwrap();
    assert_eq!(mycelium.emission_strength, 0.0);
    let crying = lib.get(crying_obsidian_violet_material_id()).unwrap();
    assert!(crying.emissive_texture.is_some());
    let bricks = lib
        .get(red_black_deepslate_bricks_violet_material_id())
        .unwrap();
    assert!(bricks.emission_strength < 0.5);
    // The family adds at most one accent light: emission does the rest.
    assert!(world_lights().len() <= 5);
}

#[test]
fn the_violet_family_is_deterministic_and_blends_with_its_neighbors() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(violet(&a), violet(&b));
    // Some violet ground cells have a non-violet family cell as neighbor:
    // the zone touches the others on the same surface.
    let others: HashSet<(i32, i32)> = a
        .families()
        .iter()
        .filter(|f| f.family != Family::Violet)
        .flat_map(|f| f.cells())
        .map(|c| (c.x, c.z))
        .collect();
    let touching = violet(&a)
        .cells()
        .iter()
        .filter(|c| {
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dz)| others.contains(&(c.x + dx, c.z + dz)))
        })
        .count();
    assert!(touching > 0);
}
