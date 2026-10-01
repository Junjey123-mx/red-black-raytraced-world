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
    crying_obsidian_orange_material_id, orange_club_material_id, orange_spade_material_id,
    polished_blackstone_bricks_material_id, red_black_deepslate_bricks_orange_material_id,
    smooth_basalt_material_id,
};
use scene::red_black_maze::{Family, FamilyLayout, family_zone};
use scene::world::WorldScene;
use std::collections::{HashMap, HashSet, VecDeque};

fn orange(scene: &WorldScene) -> &FamilyLayout {
    scene
        .families()
        .iter()
        .find(|f| f.family == Family::Orange)
        .expect("orange family")
}

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn all_six_orange_block_types_are_present() {
    let scene = WorldScene::new();
    let f = orange(&scene);
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for cell in f.cells() {
        if scene.transitions().bridged.contains(&cell) {
            continue;
        }
        *counts.entry(block_type(&scene, cell).unwrap()).or_insert(0) += 1;
    }
    for t in [
        BlockType::OrangeClub,
        BlockType::OrangeSpade,
        BlockType::SmoothBasalt,
        BlockType::PolishedBlackstoneBricks,
        BlockType::CryingObsidianOrange,
        BlockType::RedBlackDeepslateBricksOrange,
    ] {
        assert!(
            counts.get(&t).copied().unwrap_or(0) >= 3,
            "{t:?}: {counts:?}"
        );
    }
}

#[test]
fn clubs_and_spades_are_visible_from_below() {
    let scene = WorldScene::new();
    let f = orange(&scene);
    let mut clubs = 0;
    let mut spades = 0;
    for cell in f
        .symbols
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        let b = scene.world().get(*cell).unwrap();
        match b.block_type() {
            BlockType::OrangeClub => {
                clubs += 1;
                assert_eq!(b.material_id(), orange_club_material_id());
            }
            BlockType::OrangeSpade => {
                spades += 1;
                assert_eq!(b.material_id(), orange_spade_material_id());
            }
            other => panic!("{other:?} as an orange symbol"),
        }
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?} hidden"
        );
    }
    assert!(clubs >= 3 && spades >= 3, "{clubs} clubs, {spades} spades");
}

#[test]
fn basalt_columns_and_blackstone_paving_are_structural() {
    let scene = WorldScene::new();
    let f = orange(&scene);
    assert!(!f.structures.is_empty());
    for cell in f
        .structures
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::SmoothBasalt);
        assert_eq!(b.material_id(), smooth_basalt_material_id());
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "{cell:?} floats"
        );
    }
    for cell in f
        .ground
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::PolishedBlackstoneBricks);
        assert_eq!(b.material_id(), polished_blackstone_bricks_material_id());
    }
    for cell in f
        .accents
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            crying_obsidian_orange_material_id()
        );
    }
    for cell in f
        .terraces
        .iter()
        .filter(|c| !scene.transitions().bridged.contains(c))
    {
        assert_eq!(
            scene.world().get(*cell).unwrap().material_id(),
            red_black_deepslate_bricks_orange_material_id()
        );
    }
}

#[test]
fn the_orange_zone_is_connected_and_down_oriented() {
    let scene = WorldScene::new();
    let f = orange(&scene);
    let r = scene.rhombus();
    for cell in f.cells() {
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        assert_eq!(
            family_zone(scene.config(), r, cell.x, cell.z),
            Family::Orange
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
    assert!(reached, "the orange zone is not connected to the diamond");
    // It differs from Crimson: no crimson block inside the orange sector.
    for cell in f.cells() {
        assert!(!matches!(
            block_type(&scene, cell),
            Some(BlockType::CrimsonHeart | BlockType::CrimsonDiamond | BlockType::NetherWartBlock)
        ));
    }
}

#[test]
fn the_orange_family_reuses_materials_and_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(orange(&a), orange(&b));
    let src = std::fs::read_to_string("src/scene/red_black_maze.rs").unwrap();
    assert!(!src.contains("Material::new"));
    assert!(
        src.contains("orange_club_material_id()") && src.contains("orange_spade_material_id()")
    );
}
