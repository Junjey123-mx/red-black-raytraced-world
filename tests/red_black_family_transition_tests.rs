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
use scene::orientation::Orientation;
use scene::red_black_maze::{Family, family_zone, is_boundary_column};
use scene::world::WorldScene;
use std::collections::{HashMap, HashSet, VecDeque};

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

fn is_symbol(t: BlockType) -> bool {
    matches!(
        t,
        BlockType::CrimsonHeart
            | BlockType::CrimsonDiamond
            | BlockType::OrangeClub
            | BlockType::OrangeSpade
            | BlockType::PurpleHeart
            | BlockType::PurpleDiamond
            | BlockType::PurpleClub
            | BlockType::PurpleSpade
    )
}

#[test]
fn every_family_touches_another_family() {
    let scene = WorldScene::new();
    let (c, r) = (scene.config(), scene.rhombus());
    let mut pairs: HashSet<(Family, Family)> = HashSet::new();
    for &(x, z) in &scene.transitions().boundary {
        let own = family_zone(c, r, x, z);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let other = family_zone(c, r, x + dx, z + dz);
            if other != own && scene.lower_surface().cell_at(x + dx, z + dz).is_some() {
                pairs.insert((own, other));
            }
        }
    }
    for f in Family::ALL {
        assert!(
            pairs.iter().any(|(a, _)| *a == f),
            "{f:?} touches no other family: {pairs:?}"
        );
    }
    assert!(scene.transitions().boundary.len() > 30);
}

#[test]
fn the_lower_world_is_one_connected_component_without_islands() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let lower: HashSet<IVec3> = (-4..28)
        .flat_map(|z| {
            (-4..28)
                .flat_map(move |x| (r.lower_tip_y - 3..r.shelf_y).map(move |y| IVec3::new(x, y, z)))
        })
        .filter(|c| scene.world().contains(*c))
        .collect();
    let start = *lower.iter().min_by_key(|c| (c.y, c.x, c.z)).unwrap();
    let mut seen: HashSet<IVec3> = HashSet::new();
    let mut queue = VecDeque::from([start]);
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
            if lower.contains(&n) && !seen.contains(&n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(
        seen.len(),
        lower.len(),
        "{} lower cells are disconnected",
        lower.len() - seen.len()
    );
    // Every family cell is in that component.
    for f in scene.families() {
        for cell in f.cells() {
            assert!(
                seen.contains(&cell),
                "{cell:?} of {:?} is an island",
                f.family
            );
        }
    }
}

#[test]
fn boundaries_use_bridge_materials_and_no_symbols() {
    let scene = WorldScene::new();
    let t = scene.transitions();
    assert_eq!(t.bridged.len(), t.boundary.len());
    // Boundary columns hosting a hanging crystal are left alone.
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for cell in &t.bridged {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        assert!(
            !is_symbol(b.block_type()),
            "symbol {:?} on a border at {cell:?}",
            b.block_type()
        );
        assert!(
            matches!(
                b.block_type(),
                BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::Mycelium
                    | BlockType::RedBlackDeepslateBricksCrimson
                    | BlockType::RedBlackDeepslateBricksOrange
                    | BlockType::RedBlackDeepslateBricksViolet
                    | BlockType::CryingObsidianCrimson
                    | BlockType::CryingObsidianOrange
                    | BlockType::CryingObsidianViolet
            ),
            "{:?} is not a bridge material",
            b.block_type()
        );
        *counts.entry(b.block_type()).or_insert(0) += 1;
        assert!(is_boundary_column(
            scene.config(),
            scene.rhombus(),
            cell.x,
            cell.z
        ));
    }
    assert!(
        counts[&BlockType::SmoothBasalt] > 5 && counts[&BlockType::PolishedBlackstoneBricks] > 5
    );
    let crying = counts
        .iter()
        .filter(|(t, _)| {
            matches!(
                t,
                BlockType::CryingObsidianCrimson
                    | BlockType::CryingObsidianOrange
                    | BlockType::CryingObsidianViolet
            )
        })
        .map(|(_, n)| n)
        .sum::<usize>();
    assert!(
        crying * 8 < t.bridged.len(),
        "crying obsidian saturates the borders"
    );
    // Symbols still exist away from the borders.
    let symbols = scene
        .families()
        .iter()
        .flat_map(|f| f.symbols.iter())
        .filter(|c| block_type(&scene, **c).map(is_symbol).unwrap_or(false))
        .count();
    assert!(symbols > 30, "{symbols} symbols survive");
}

#[test]
fn the_borders_are_not_straight_lines() {
    let scene = WorldScene::new();
    // Boundary columns spread over many rows and columns rather than a few
    // fixed x or z values.
    let xs: HashSet<i32> = scene
        .transitions()
        .boundary
        .iter()
        .map(|(x, _)| *x)
        .collect();
    let zs: HashSet<i32> = scene
        .transitions()
        .boundary
        .iter()
        .map(|(_, z)| *z)
        .collect();
    assert!(
        xs.len() >= 10 && zs.len() >= 10,
        "{} xs, {} zs",
        xs.len(),
        zs.len()
    );
}

#[test]
fn the_transitions_are_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.transitions(), b.transitions());
}
