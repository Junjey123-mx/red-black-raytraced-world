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

use core::hit::Face;
use core::math::{IVec3, Vec2, Vec3};
use scene::block_shape_factory::block_geometry;
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::geometry_orientation::functional_face_uv;
use scene::light::Light;
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
use scene::rhombus::connected_component;
use scene::world::{WorldScene, world_background, world_camera, world_lights};
use std::collections::HashMap;

fn census(scene: &WorldScene) -> HashMap<BlockType, usize> {
    let r = scene.rhombus();
    let mut counts = HashMap::new();
    for z in -6..30 {
        for x in -6..30 {
            for y in (r.lower_tip_y - 4)..=24 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    *counts.entry(b.block_type()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

#[test]
fn one_voxel_world_holds_the_preserved_overworld_and_the_upper_mass() {
    let scene = WorldScene::new();
    // 1. single VoxelWorld (one field, no second scene).
    let src = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(src.matches("world: VoxelWorld,").count(), 1);
    assert!(!src.contains("LowerScene") && !src.contains("SecondWorld"));
    // 2. Overworld preserved.
    assert_eq!(scene.house().floor.len(), 35);
    assert_eq!(scene.house_exterior().lamps.len(), 2);
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(scene.trees().len(), 4);
    assert_eq!(scene.path().len(), 10);
    // 3. upper mass.
    let r = scene.rhombus();
    assert!(scene.upper_taper().cells.len() > 800);
    assert!(
        scene
            .world()
            .contains(IVec3::new(r.center_x, r.waist_y, r.center_z))
    );
}

#[test]
fn cutaway_upper_descent_and_portal_are_integrated() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    // 4. cutaway.
    assert!(scene.cutaway().removed_count() > 200);
    assert!(!scene.world().contains(IVec3::new(
        r.cutaway.min_x + 1,
        r.waist_y,
        r.cutaway.min_z + 1
    )));
    // 5. upper descent: from the landing to the shelf without clipping.
    let d = scene.upper_descent();
    assert!(d.treads.len() >= 12);
    for (cell, orientation) in &d.treads {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.orientation(), *orientation);
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "{cell:?} clipped from above"
        );
    }
    // 6-7. portal frame and core.
    assert_eq!(scene.portal().frame.len(), 18);
    assert_eq!(scene.portal().core.len(), 12);
    for cell in &scene.portal().core {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::PortalCoreDarkCrimson
        );
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y, cell.z + 1)),
            "portal overlap at {cell:?}"
        );
    }
    // 8. portal light.
    let points = world_lights()
        .iter()
        .filter(|l| matches!(l, Light::Point(_)))
        .count();
    assert!(points >= 1 && points <= 4);
}

#[test]
fn down_face_mapping_lower_mass_inverted_descent_and_surface_hold() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    // 9. Down face mapping.
    assert_eq!(
        functional_face_uv(Face::NegativeY, Vec2::new(0.2, 0.3), Orientation::Down).0,
        Face::PositiveY
    );
    assert_eq!(
        functional_face_uv(Face::PositiveY, Vec2::new(0.2, 0.3), Orientation::Up),
        (Face::PositiveY, Vec2::new(0.2, 0.3))
    );
    // 10. lower mass.
    assert!(scene.lower_mass().cells.len() > 1000);
    // 11. inverted route (rebuilt behind the portal in Gate 14): every
    //     tread is a Down stair hanging from the mass with air below it.
    let inv = scene.inverted_route();
    assert!(inv.layout.step_count >= 5);
    for (cell, o) in &inv.treads {
        assert_eq!(*o, Orientation::Down);
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z))
        );
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        );
        assert!(!scene.portal().core.contains(cell));
        assert!(!block_geometry(BlockType::WoodStairs, Orientation::Down).is_full_cube());
    }
    // 12. lower surface faces -Y.
    for cell in &scene.lower_surface().cells {
        assert_eq!(
            scene.world().get(*cell).unwrap().orientation(),
            Orientation::Down
        );
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
                || scene
                    .families()
                    .iter()
                    .any(
                        |f| f.terraces.contains(&IVec3::new(cell.x, cell.y - 1, cell.z))
                            || f.structures
                                .contains(&IVec3::new(cell.x, cell.y - 1, cell.z))
                    )
        );
        assert!(cell.y < r.shelf_y);
    }
}

#[test]
fn the_three_families_their_transitions_and_the_tip_are_in_place() {
    let scene = WorldScene::new();
    let counts = census(&scene);
    // 13-15. Crimson, Orange, Violet.
    let present = |t: BlockType| counts.get(&t).copied().unwrap_or(0) > 0;
    for t in [
        BlockType::CrimsonHeart,
        BlockType::CrimsonDiamond,
        BlockType::CryingObsidianCrimson,
        BlockType::NetherWartBlock,
        BlockType::RedBlackDeepslateBricksCrimson,
    ] {
        assert!(present(t), "{t:?}");
    }
    for t in [
        BlockType::OrangeClub,
        BlockType::OrangeSpade,
        BlockType::SmoothBasalt,
        BlockType::PolishedBlackstoneBricks,
        BlockType::CryingObsidianOrange,
        BlockType::RedBlackDeepslateBricksOrange,
    ] {
        assert!(present(t), "{t:?}");
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
        assert!(present(t), "{t:?}");
    }
    assert_eq!(scene.families().len(), 3);
    assert_eq!(
        scene
            .families()
            .iter()
            .map(|f| f.family)
            .collect::<Vec<_>>(),
        Family::ALL.to_vec()
    );
    // 16. transitions: one connected lower world, borders bridged.
    assert!(scene.transitions().boundary.len() > 30);
    let main = connected_component(scene.world(), scene.silhouette().tip);
    assert_eq!(main.len(), scene.world().len());
    // 17. lower tip.
    let r = scene.rhombus();
    assert!(
        scene
            .world()
            .contains(IVec3::new(r.center_x, r.lower_tip_y, r.center_z))
    );
    for z in -6..30 {
        for x in -6..30 {
            assert!(!scene.world().contains(IVec3::new(x, r.lower_tip_y - 1, z)));
        }
    }
}

#[test]
fn camera_catalog_and_gate_boundaries_are_respected() {
    let scene = WorldScene::new();
    // 18. camera.
    let camera = world_camera(4.0 / 3.0);
    let r = scene.rhombus();
    assert!(camera.target.y < 0.0 && camera.target.y > r.lower_tip_y as f32);
    assert!(camera.position.y > camera.target.y);
    assert!((camera.position - camera.target).length() > 30.0);
    // 19. Catalog 44/44 (39/39 before Gate 17.5).
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(official_entries().len(), 44);
    assert_eq!(CatalogScene::new().len(), 46);
    // 20. no Gate 14/15 features: sky is still the procedural gradient, no
    // threads, no cubemap, no gameplay.
    assert!(world_background().is_sky());
    assert!(world_background().color(Vec3::new(0.0, 1.0, 0.0)).b > 0.6);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(!app.contains("std::thread") && !app.contains("available_parallelism"));
    assert!(!app.contains("cubemap") && !app.contains("Player") && !app.contains("gravity"));
    let skybox = std::fs::read_to_string("src/renderer/skybox.rs").unwrap();
    assert!(!skybox.contains("Cubemap {"));
    // Gate 13.6 filled `parallel.rs` with scoped std threads; the viewer
    // itself still spawns nothing and no rayon is involved.
    let parallel = std::fs::read_to_string("src/renderer/parallel.rs").unwrap();
    assert!(parallel.contains("std::thread::scope") && !parallel.contains("rayon"));
    // Every block in the world belongs to a known family (no stray type).
    for t in census(&scene).keys() {
        let _ = t.family();
        assert!(
            !matches!(t.family(), BlockFamily::Portal)
                || matches!(
                    t,
                    BlockType::PortalFrameRedObsidian | BlockType::PortalCoreDarkCrimson
                )
        );
    }
}
