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

use core::math::IVec3;
use scene::block_type::BlockType;
use scene::fortress::{is_dark_structure, is_family_brick};
use scene::light::Light;
use scene::orientation::Orientation;
use scene::overworld_blocks::{
    red_black_deepslate_bricks_crimson_material_id, red_black_deepslate_bricks_orange_material_id,
    red_black_deepslate_bricks_violet_material_id,
};
use scene::world::{WorldScene, world_lights};
use std::collections::BTreeMap;

#[test]
fn all_three_variants_are_present() {
    let scene = WorldScene::new();
    let b = scene.fortress_bricks();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for cell in b.cells() {
        let blk = scene.world().get(cell).unwrap();
        assert!(is_family_brick(blk.block_type()), "{cell:?}");
        assert_eq!(blk.orientation(), Orientation::Down);
        *counts.entry(format!("{:?}", blk.block_type())).or_insert(0) += 1;
    }
    for t in [
        BlockType::RedBlackDeepslateBricksCrimson,
        BlockType::RedBlackDeepslateBricksOrange,
        BlockType::RedBlackDeepslateBricksViolet,
    ] {
        assert!(
            counts.get(&format!("{t:?}")).copied().unwrap_or(0) >= 3,
            "{t:?}: {counts:?}"
        );
    }
    assert!(
        !b.edges.is_empty()
            && !b.bands.is_empty()
            && !b.gate_accents.is_empty()
            && !b.tower_markers.is_empty()
    );
    println!(
        "GATE17 colored bricks: edges={} bands={} gate={} markers={} counts={counts:?} voxels={}",
        b.edges.len(),
        b.bands.len(),
        b.gate_accents.len(),
        b.tower_markers.len(),
        scene.world().len()
    );
}

#[test]
fn the_dark_structure_stays_the_majority() {
    let scene = WorldScene::new();
    let cells = scene.fortress_cells();
    let coloured = cells
        .iter()
        .filter(|c| is_family_brick(scene.world().get(**c).unwrap().block_type()))
        .count();
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(coloured * 2 < dark, "coloured {coloured} vs dark {dark}");
    assert!(
        coloured >= 20,
        "too little colour: {coloured} of {}",
        cells.len()
    );
    // Recolouring adds nothing: the weave is cells of the curtain.
    for cell in scene.fortress_bricks().cells() {
        assert!(
            scene.fortress_foundation().walls.contains(&cell),
            "{cell:?} is not a curtain cell"
        );
    }
}

#[test]
fn emission_stays_localized() {
    let scene = WorldScene::new();
    let coloured = scene.fortress_bricks().cells();
    // Bands and edges: no coloured cell touches more than two others, so
    // the glow reads as lines and corners, never as a lit wall.
    for c in &coloured {
        let touching = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ]
        .iter()
        .filter(|(dx, dy, dz)| coloured.contains(&IVec3::new(c.x + dx, c.y + dy, c.z + dz)))
        .count();
        assert!(touching <= 2, "{c:?} touches {touching} coloured cells");
    }
    // The gate accents: Crimson left, Violet right, Orange lintel.
    let l = scene.fortress_layout();
    let g = l.gate;
    let at = |cell: IVec3| scene.world().get(cell).unwrap().block_type();
    assert_eq!(
        at(IVec3::new(g.base.x, g.base.y, g.base.z - 1)),
        BlockType::RedBlackDeepslateBricksCrimson
    );
    assert_eq!(
        at(IVec3::new(g.base.x, g.base.y, g.base.z + g.width)),
        BlockType::RedBlackDeepslateBricksViolet
    );
    assert_eq!(
        at(IVec3::new(g.base.x, g.base.y - 2, g.base.z)),
        BlockType::RedBlackDeepslateBricksOrange
    );
}

#[test]
fn no_new_material_ids_are_used() {
    let scene = WorldScene::new();
    for cell in scene.fortress_bricks().cells() {
        let b = scene.world().get(cell).unwrap();
        let expected = match b.block_type() {
            BlockType::RedBlackDeepslateBricksCrimson => {
                red_black_deepslate_bricks_crimson_material_id()
            }
            BlockType::RedBlackDeepslateBricksOrange => {
                red_black_deepslate_bricks_orange_material_id()
            }
            BlockType::RedBlackDeepslateBricksViolet => {
                red_black_deepslate_bricks_violet_material_id()
            }
            other => panic!("{other:?}"),
        };
        assert_eq!(b.material_id(), expected);
    }
    let src = std::fs::read_to_string("src/scene/fortress.rs").unwrap();
    assert!(
        !src.contains("MaterialId::new")
            && !src.contains("Material::")
            && !src.contains("register")
    );
    let lib = std::fs::read_to_string("src/scene/material_library.rs").unwrap();
    assert!(!lib.contains("fortress"));
}

#[test]
fn no_new_lights_are_added() {
    let lights = world_lights();
    assert_eq!(lights.len(), 5);
    assert_eq!(
        lights
            .iter()
            .filter(|l| !matches!(l, Light::Directional(_)))
            .count(),
        4
    );
    let src = std::fs::read_to_string("src/scene/fortress.rs").unwrap();
    assert!(!src.contains("PointLight") && !src.contains("Light::"));
}
