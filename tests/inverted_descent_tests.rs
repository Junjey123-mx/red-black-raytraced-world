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
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::overworld_blocks::wood_stairs_material_id;
use scene::world::WorldScene;

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn the_inverted_flight_starts_right_under_the_portal_side() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let d = scene.inverted_descent();
    let first = d.first().unwrap();
    assert_eq!(first.y, r.shelf_y - 1);
    assert_eq!(first.x, r.cutaway.min_x - 1);
    assert_eq!(first.z, r.cutaway.min_z);
    // Under the portal's frame column and beside the cutaway.
    assert!(first.x >= r.portal.min_x && first.x <= r.portal.max_x);
    assert!(first.y < r.portal.min_y);
    assert!(r.cutaway.contains_column(first.x + 1, first.z));
    assert!(d.treads.len() >= 5, "{} treads", d.treads.len());
}

#[test]
fn every_inverted_tread_is_a_down_stair_hanging_from_the_mass() {
    let scene = WorldScene::new();
    let d = scene.inverted_descent();
    let mut previous: Option<IVec3> = None;
    for (cell, orientation) in &d.treads {
        assert_eq!(*orientation, Orientation::Down);
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.material_id(), wood_stairs_material_id());
        assert_eq!(b.orientation(), Orientation::Down);
        // Ceiling above (the inverted ground), open below.
        let above = block_type(&scene, IVec3::new(cell.x, cell.y + 1, cell.z));
        assert!(
            matches!(
                above,
                Some(
                    BlockType::Deepslate
                        | BlockType::SmoothBasalt
                        | BlockType::PolishedBlackstoneBricks
                        | BlockType::DeepslateBricks
                )
            ),
            "{cell:?} hangs from {above:?}"
        );
        for y in (scene.rhombus().lower_tip_y - 1)..cell.y {
            assert!(
                !scene.world().contains(IVec3::new(cell.x, y, cell.z)),
                "{cell:?} has mass under it at y={y}"
            );
        }
        if let Some(p) = previous {
            assert_eq!(cell.y, p.y - 1);
            assert_eq!(cell.z, p.z + 1);
            assert_eq!(cell.x, p.x);
        }
        previous = Some(*cell);
    }
}

#[test]
fn the_flipped_geometry_is_raised_toward_the_previous_step() {
    // A Down stair mirrors the canonical one top-to-bottom: its full-height
    // half sits on the north (previous step) side and hangs from the top.
    let geometry = block_geometry(BlockType::WoodStairs, Orientation::Down);
    let printed = format!("{geometry:?}");
    match geometry {
        BlockGeometry::Composite(parts) => {
            assert_eq!(parts.len(), 2);
            let hanging = parts
                .iter()
                .find(|p| p.min().y > 0.4)
                .expect("a hanging half");
            assert!(
                hanging.max().y > 0.99,
                "the hanging half touches the ceiling"
            );
            let lower = parts
                .iter()
                .find(|p| p.min().y < 0.1)
                .expect("a full-height or low half");
            assert!(lower.max().z <= 0.51 || hanging.max().z <= 0.51);
            for p in parts {
                assert!(p.is_within_unit_cell());
            }
        }
        other => panic!("{other:?}"),
    }
    let up = block_geometry(BlockType::WoodStairs, Orientation::Up);
    assert_ne!(format!("{up:?}"), printed);
}

#[test]
fn the_flight_never_touches_the_core_and_ends_on_the_lower_surface() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let d = scene.inverted_descent();
    for (cell, _) in &d.treads {
        assert!(!scene.portal().core.contains(cell));
        assert!(!scene.portal().frame.contains(cell));
        assert!(cell.y < r.portal.min_y);
        assert!(
            r.contains(scene.config(), cell.x, cell.y + 1, cell.z),
            "{cell:?} outside the diamond"
        );
    }
    // The last tread is the lowest cell of its column: it is part of the
    // -Y-facing surface.
    let last = d.last().unwrap();
    for y in r.lower_tip_y..last.y {
        assert!(!scene.world().contains(IVec3::new(last.x, y, last.z)));
    }
    assert!(
        scene
            .world()
            .contains(IVec3::new(last.x, last.y + 1, last.z))
    );
    // Beyond the last tread the mass has no cell under the flight line:
    // the route has emerged under the world.
    let next = IVec3::new(last.x, last.y - 1, last.z + 1);
    assert!(!r.contains(scene.config(), next.x, next.y, next.z) || !scene.world().contains(next));
}

#[test]
fn the_flight_is_visible_through_the_cutaway_and_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.inverted_descent(), b.inverted_descent());
    let r = a.rhombus();
    for (cell, _) in &a.inverted_descent().treads {
        // The cell east of every tread lies in the cut quadrant and is air.
        assert!(r.cutaway.contains_column(cell.x + 1, cell.z));
        assert!(!a.world().contains(IVec3::new(cell.x + 1, cell.y, cell.z)));
    }
    assert!(!a.inverted_descent().dug.is_empty());
}
