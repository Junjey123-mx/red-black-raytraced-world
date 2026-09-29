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
use scene::block_shape_factory::block_geometry;
use scene::block_type::{BlockFamily, BlockType};
use scene::descent::{PIT_SERPENTINE, stair_facing_uphill};
use scene::material_gallery::deepslate_bricks_material_id;
use scene::orientation::Orientation;
use scene::overworld::{DESCENT_ENDPOINT, HOUSE_FOOTPRINT, pond_basin_columns};
use scene::overworld_blocks::wood_stairs_material_id;
use scene::world::WorldScene;
use std::collections::HashSet;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn the_descent_starts_beside_the_gate12_endpoint() {
    let scene = WorldScene::new();
    let d = scene.upper_descent();
    let first = d.first().unwrap();
    let (ex, ez) = DESCENT_ENDPOINT;
    // First tread: beside the landing, one row south of the endpoint.
    assert_eq!((first.x, first.z), (ex - 1, ez + 1));
    let landing = IVec3::new(ex, first.y + 1, ez + 1);
    assert_eq!(block_type(&scene, landing), Some(BlockType::Cobblestone));
    assert!(scene.path().landing.contains(&landing));
    assert_eq!(d.treads.len(), PIT_SERPENTINE.len() + 1);
    let end = scene.path().end().unwrap();
    assert_eq!((end.x, end.z), DESCENT_ENDPOINT);
}

#[test]
fn the_descent_ends_at_the_portal_approach_on_the_shelf() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let d = scene.upper_descent();
    let last = d.last().unwrap();
    assert_eq!(last.y, r.shelf_y + 2);
    assert!(r.cutaway.contains_column(last.x, last.z));
    // The last stair stands on a deepslate step over the shelf plate.
    assert_eq!(
        block_type(&scene, IVec3::new(last.x, r.shelf_y + 1, last.z)),
        Some(BlockType::Deepslate)
    );
    assert_eq!(
        block_type(&scene, IVec3::new(last.x, r.shelf_y, last.z)),
        Some(BlockType::Deepslate)
    );
    // The brick landing fills the two rows in front of the opening.
    let p = r.portal;
    assert_eq!(d.approach.len(), ((p.max_x - p.min_x - 1) * 2) as usize);
    for cell in &d.approach {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::DeepslateBricks);
        assert_eq!(b.material_id(), deepslate_bricks_material_id());
        assert_eq!(cell.y, r.shelf_y);
        assert!(cell.z > p.wall_z && cell.x > p.min_x && cell.x < p.max_x);
        assert!(block_type(&scene, IVec3::new(cell.x, cell.y + 1, cell.z)).is_none());
    }
    // A walker can go from the last stair to the approach over air cells
    // one above the shelf.
    let level = r.shelf_y + 1;
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut queue = vec![(last.x, last.z)];
    while let Some((x, z)) = queue.pop() {
        if !seen.insert((x, z)) {
            continue;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            let floor = scene.world().contains(IVec3::new(nx, level - 1, nz));
            let free = block_type(&scene, IVec3::new(nx, level, nz)).is_none()
                && block_type(&scene, IVec3::new(nx, level + 1, nz)).is_none();
            if floor && free && !seen.contains(&(nx, nz)) {
                queue.push((nx, nz));
            }
        }
    }
    for cell in &d.approach {
        assert!(
            seen.contains(&(cell.x, cell.z)),
            "approach {cell:?} unreachable"
        );
    }
}

#[test]
fn every_step_descends_exactly_one_block_and_faces_uphill() {
    let scene = WorldScene::new();
    let d = scene.upper_descent();
    let (ex, ez) = DESCENT_ENDPOINT;
    let mut previous = IVec3::new(ex, d.first().unwrap().y + 1, ez + 1);
    assert_eq!(block_type(&scene, previous), Some(BlockType::Cobblestone));
    for (cell, orientation) in &d.treads {
        assert_eq!(cell.y, previous.y - 1, "{cell:?} after {previous:?}");
        assert_eq!((cell.x - previous.x).abs() + (cell.z - previous.z).abs(), 1);
        assert_eq!(*orientation, stair_facing_uphill(*cell, previous));
        let block = scene.world().get(*cell).unwrap();
        assert_eq!(block.block_type(), BlockType::WoodStairs);
        assert_eq!(block.material_id(), wood_stairs_material_id());
        assert_eq!(block.orientation(), *orientation);
        assert!(!block_geometry(BlockType::WoodStairs, *orientation).is_full_cube());
        previous = *cell;
    }
    assert_eq!(
        stair_facing_uphill(IVec3::new(0, 0, 0), IVec3::new(0, 0, -1)),
        Orientation::South
    );
    assert_eq!(
        stair_facing_uphill(IVec3::new(0, 0, 0), IVec3::new(-1, 0, 0)),
        Orientation::East
    );
}

#[test]
fn every_tread_rests_on_rock_and_is_open_to_the_sky() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    for (cell, _) in &scene.upper_descent().treads {
        let below = block_type(&scene, IVec3::new(cell.x, cell.y - 1, cell.z));
        assert!(
            matches!(
                below,
                Some(
                    BlockType::Dirt
                        | BlockType::Stone
                        | BlockType::Deepslate
                        | BlockType::Grass
                        | BlockType::Sand
                        | BlockType::Cobblestone
                )
            ),
            "{cell:?} floats on {below:?}"
        );
        // Nothing above the tread up to the sky: the pit is open, so the
        // whole route is visible from the cutaway side and from above.
        for y in (cell.y + 1)..=r.upper_surface_reference + 12 {
            assert!(
                !scene.world().contains(IVec3::new(cell.x, y, cell.z)),
                "{cell:?} buried under y={y}"
            );
        }
    }
    assert!(!scene.upper_descent().dug.is_empty());
    assert!(scene.upper_descent().shelf.len() > 5);
    // Footings are few and short: the pit sits on the natural mass.
    assert!(
        scene.upper_descent().footings.len() <= 6,
        "{} footings",
        scene.upper_descent().footings.len()
    );
}

#[test]
fn the_route_keeps_away_from_the_house_and_the_pond() {
    let scene = WorldScene::new();
    let basin = pond_basin_columns();
    for (cell, _) in &scene.upper_descent().treads {
        assert!(!HOUSE_FOOTPRINT.grown(1).contains(cell.x, cell.z));
        assert!(!basin.contains(&(cell.x, cell.z)));
    }
    for cell in &scene.upper_descent().dug {
        assert!(!HOUSE_FOOTPRINT.contains(cell.x, cell.z));
        assert!(!basin.contains(&(cell.x, cell.z)));
    }
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(scene.house_exterior().lamps.len(), 2);
}

#[test]
fn the_route_never_enters_the_portal_wall() {
    // The descent ends in front of the portal anchor, never inside it: the
    // frame and core are built by the portal stages into an intact wall.
    let scene = WorldScene::new();
    let p = scene.rhombus().portal;
    let d = scene.upper_descent();
    for cell in d
        .treads
        .iter()
        .map(|(c, _)| *c)
        .chain(d.dug.iter().copied())
        .chain(d.footings.iter().copied())
        .chain(d.approach.iter().copied())
    {
        assert!(
            !(cell.z == p.wall_z && p.is_frame(cell.x, cell.y)),
            "{cell:?} in the frame"
        );
        assert!(
            !(cell.z == p.wall_z && p.is_opening(cell.x, cell.y)),
            "{cell:?} in the opening"
        );
    }
    // The descent itself uses only Overworld blocks.
    for (cell, _) in &d.treads {
        assert_eq!(
            block_type(&scene, *cell).unwrap().family(),
            BlockFamily::OverworldArchitecture
        );
    }
}

#[test]
fn the_descent_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.upper_descent(), b.upper_descent());
}
