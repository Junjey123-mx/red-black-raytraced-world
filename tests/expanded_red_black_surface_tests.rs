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
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_collision::{
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::hit::Face;
use core::math::{IVec3, Vec2, Vec3};
use scene::block_type::BlockType;
use scene::geometry_orientation::functional_face_uv;
use scene::orientation::Orientation;
use scene::world::WorldScene;
use std::collections::{BTreeMap, BTreeSet};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn family_of(t: BlockType) -> Option<&'static str> {
    match t {
        BlockType::RedBlackDeepslateBricksCrimson => Some("crimson"),
        BlockType::RedBlackDeepslateBricksOrange => Some("orange"),
        BlockType::RedBlackDeepslateBricksViolet => Some("violet"),
        _ => None,
    }
}

#[test]
fn the_functional_top_faces_minus_y() {
    let scene = WorldScene::new();
    let s = scene.red_black_surface_extension();
    assert!(s.cells.len() >= 150, "{}", s.cells.len());
    for cell in &s.cells {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down, "{cell:?}");
        // The ground cell is the lowest of its column: its -Y face is open.
        assert!(
            !scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?} is not the ground"
        );
        assert!(
            matches!(
                b.block_type(),
                BlockType::Mycelium
                    | BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::RedBlackDeepslateBricksCrimson
                    | BlockType::RedBlackDeepslateBricksOrange
                    | BlockType::RedBlackDeepslateBricksViolet
            ),
            "{cell:?} is {:?}",
            b.block_type()
        );
    }
}

#[test]
fn mycelium_shows_its_top_on_the_minus_y_face() {
    let scene = WorldScene::new();
    let mycelium: Vec<IVec3> = scene
        .red_black_surface_extension()
        .cells
        .iter()
        .copied()
        .filter(|c| scene.world().get(*c).unwrap().block_type() == BlockType::Mycelium)
        .collect();
    assert!(!mycelium.is_empty());
    // A Down block maps its geometric -Y face to the functional top (+Y).
    let (face, _) = functional_face_uv(Face::NegativeY, Vec2::new(0.3, 0.6), Orientation::Down);
    assert_eq!(face, Face::PositiveY);
    for cell in &mycelium {
        assert_eq!(
            scene.world().get(*cell).unwrap().orientation(),
            Orientation::Down
        );
    }
}

#[test]
fn the_new_surface_has_several_heights() {
    let scene = WorldScene::new();
    let heights: BTreeSet<i32> = scene
        .red_black_surface_extension()
        .cells
        .iter()
        .map(|c| c.y)
        .collect();
    assert!(heights.len() >= 4, "{heights:?}");
    let raised = &scene.red_black_surface_extension().raised;
    assert!(!raised.is_empty());
    let l = scene.expansion_layout();
    for cell in raised {
        assert!(
            !l.red_black_fortress_pad.contains(cell.x, cell.z),
            "relief on the pad at {cell:?}"
        );
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "{cell:?} floats"
        );
    }
    println!(
        "GATE15 lower surface: cells={} raised={} heights={heights:?} voxels={}",
        scene.red_black_surface_extension().cells.len(),
        raised.len(),
        scene.world().len()
    );
}

#[test]
fn the_fortress_pad_is_navigable_and_level() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let pad = l.red_black_fortress_pad;
    let y = l.fortress_pad_bottom_y as f32 - 1.5;
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            assert_eq!(
                scene.red_black_expansion().column(x, z).unwrap().bottom_y,
                l.fortress_pad_bottom_y
            );
            assert!(clear(&scene, v(x as f32 + 0.5, y, z as f32 + 0.5)));
        }
    }
    let mid_z = (pad.min_z + pad.max_z) as f32 / 2.0 + 0.5;
    let from = v(pad.min_x as f32 + 0.5, y, mid_z);
    let to = v(pad.max_x as f32 + 0.5, y, mid_z);
    let reached = resolve_camera_motion(
        scene.world(),
        from,
        to - from,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    );
    assert!((reached - to).length() < 1e-3, "{reached:?}");
    let across = v(from.x, y, pad.max_z as f32 + 0.5);
    let reached = resolve_camera_motion(
        scene.world(),
        from,
        across - from,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    );
    assert!((reached - across).length() < 1e-3, "{reached:?}");
}

#[test]
fn family_accents_are_balanced_and_restrained() {
    let scene = WorldScene::new();
    let s = scene.red_black_surface_extension();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for cell in &s.cells {
        if let Some(f) = family_of(scene.world().get(*cell).unwrap().block_type()) {
            *counts.entry(f).or_insert(0) += 1;
        }
    }
    println!("GATE15 family bricks on the new surface: {counts:?}");
    for f in ["crimson", "orange", "violet"] {
        assert!(
            counts.get(f).copied().unwrap_or(0) >= 2,
            "{f} missing: {counts:?}"
        );
    }
    let coloured: usize = counts.values().sum();
    assert!(
        coloured * 2 <= s.cells.len(),
        "the expansion is saturated with colour: {coloured} of {}",
        s.cells.len()
    );
    assert!(
        coloured * 10 >= s.cells.len(),
        "too few accents: {coloured}"
    );
}

#[test]
fn there_are_no_hard_family_borders() {
    let scene = WorldScene::new();
    let s = scene.red_black_surface_extension();
    let family_at = |x: i32, z: i32| {
        s.cells
            .iter()
            .find(|c| c.x == x && c.z == z)
            .and_then(|c| family_of(scene.world().get(*c).unwrap().block_type()))
    };
    // No family forms a solid block of colour: every coloured cell has at
    // least one uncoloured or differently coloured neighbour, and each
    // family's cells spread over a box much larger than their count.
    for cell in &s.cells {
        let Some(f) = family_at(cell.x, cell.z) else {
            continue;
        };
        let same = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|(dx, dz)| family_at(cell.x + dx, cell.z + dz) == Some(f))
            .count();
        assert!(same < 4, "{cell:?} sits inside a solid {f} block");
    }
    for f in ["crimson", "orange", "violet"] {
        let cells: Vec<IVec3> = s
            .cells
            .iter()
            .copied()
            .filter(|c| family_at(c.x, c.z) == Some(f))
            .collect();
        if cells.len() < 3 {
            continue;
        }
        let (minx, maxx) = (
            cells.iter().map(|c| c.x).min().unwrap(),
            cells.iter().map(|c| c.x).max().unwrap(),
        );
        let (minz, maxz) = (
            cells.iter().map(|c| c.z).min().unwrap(),
            cells.iter().map(|c| c.z).max().unwrap(),
        );
        let area = ((maxx - minx + 1) * (maxz - minz + 1)) as usize;
        assert!(
            area >= 2 * cells.len(),
            "{f} is a filled rectangle: {} cells in {area}",
            cells.len()
        );
    }
}

#[test]
fn the_existing_route_remains_clear() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    for (tread, _) in &route.treads {
        let eye = v(
            tread.x as f32 + 0.5,
            tread.y as f32 - 1.5,
            tread.z as f32 + 0.5,
        );
        assert!(clear(&scene, eye), "{tread:?}");
    }
    let l = route.layout;
    for cell in l.platform_cells() {
        assert!(scene.world().contains(cell));
        assert!(clear(
            &scene,
            v(
                cell.x as f32 + 0.5,
                cell.y as f32 - 1.5,
                cell.z as f32 + 0.5
            )
        ));
    }
    // The new surface never touches the route or the certified surface.
    let s = scene.red_black_surface_extension();
    for cell in &s.cells {
        assert!(!route.dug.contains(cell) && !route.treads.iter().any(|(t, _)| t == cell));
        assert!(!scene.lower_surface().cells.contains(cell));
    }
    for cell in &scene.lower_surface().cells {
        assert!(scene.world().contains(*cell));
    }
}
