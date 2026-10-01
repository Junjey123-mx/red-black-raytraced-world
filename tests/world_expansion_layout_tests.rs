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
use scene::expansion::{
    EXPANSION_DEPTH, EXPANSION_WIDTH, PAD_SIDE, VOXEL_BUDGET_SOFT_MAX, VOXEL_BUDGET_TARGET,
    WorldExpansionLayout,
};
use scene::overworld::{HOUSE_FOOTPRINT, pond_basin_columns};
use scene::world::WorldScene;

#[test]
fn the_layout_is_deterministic_and_derived_from_the_world() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.expansion_layout(), b.expansion_layout());
    let l = *a.expansion_layout();
    println!("GATE15 layout: {l:?}");
    let derived = WorldExpansionLayout::derive(
        a.config(),
        a.rhombus(),
        &a.inverted_route().layout,
        a.path(),
        a.world(),
    );
    assert_eq!(l, derived);
    // The lobe starts exactly where the terrain footprint ends (one landform).
    assert_eq!(l.overworld_extension.min_x, a.config().width);
    assert_eq!(l.overworld_extension.max_z, a.rhombus().portal.wall_z);
    assert_eq!(
        l.overworld_extension.max_x - l.overworld_extension.min_x + 1,
        EXPANSION_WIDTH
    );
    assert_eq!(
        l.overworld_extension.max_z - l.overworld_extension.min_z + 1,
        EXPANSION_DEPTH
    );
    assert_eq!(l.red_black_extension, l.overworld_extension);
    assert!(
        l.validate(a.rhombus()).is_ok(),
        "{:?}",
        l.validate(a.rhombus())
    );
}

#[test]
fn the_castle_pad_keeps_its_distance_from_the_house() {
    let scene = WorldScene::new();
    let pad = scene.expansion_layout().overworld_castle_pad;
    let house = HOUSE_FOOTPRINT;
    assert!(
        pad.min_x - house.max_x >= 8,
        "pad {pad:?} crowds the house {house:?}"
    );
    for z in pad.min_z..=pad.max_z {
        for x in pad.min_x..=pad.max_x {
            assert!(!house.grown(2).contains(x, z));
        }
    }
    for cell in scene.house().walls.iter().chain(scene.house().floor.iter()) {
        assert!(!pad.contains(cell.x, cell.z));
    }
}

#[test]
fn the_castle_pad_does_not_touch_the_pond() {
    let scene = WorldScene::new();
    let pad = scene.expansion_layout().overworld_castle_pad;
    let ext = scene.expansion_layout().overworld_extension;
    for (x, z) in pond_basin_columns() {
        assert!(!pad.contains(x, z) && !ext.contains(x, z));
    }
    for cell in &scene.pond().water {
        assert!(!ext.contains(cell.x, cell.z));
    }
}

#[test]
fn the_fortress_pad_stays_clear_of_the_portal_and_cutaway() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    let r = scene.rhombus();
    for cell in scene
        .portal()
        .frame
        .iter()
        .chain(scene.portal().core.iter())
    {
        assert!(!l.red_black_fortress_pad.contains(cell.x, cell.z));
        assert!(!l.red_black_extension.contains(cell.x, cell.z));
    }
    for z in l.red_black_extension.min_z..=l.red_black_extension.max_z {
        for x in l.red_black_extension.min_x..=l.red_black_extension.max_x {
            assert!(!r.cutaway.contains_column(x, z));
        }
    }
    // The tunnel and landing of Gate 14 are untouched by the lobe.
    for cell in scene
        .inverted_route()
        .dug
        .iter()
        .chain(scene.inverted_route().supports.iter())
    {
        assert!(!l.red_black_extension.contains(cell.x, cell.z));
    }
}

#[test]
fn the_fortress_pad_does_not_cover_family_focal_blocks() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    for family in scene.families() {
        for cell in family.cells() {
            assert!(
                !l.red_black_fortress_pad.contains(cell.x, cell.z),
                "{cell:?} of {:?} under the pad",
                family.family
            );
            assert!(
                !l.red_black_extension.contains(cell.x, cell.z),
                "{cell:?} of {:?} under the lobe",
                family.family
            );
        }
    }
}

#[test]
fn both_paths_have_valid_endpoints() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    // The upper approach leaves from a real cell of the existing path.
    assert!(scene.path().cells.contains(&l.overworld_path_anchor));
    // The path cell nearest the lobe: just before the pit, on the porch path.
    assert_eq!(l.overworld_path_anchor, IVec3::new(13, 4, 14));
    assert!(
        l.overworld_castle_pad
            .contains(l.overworld_path_target.x, l.overworld_path_target.z)
    );
    assert_eq!(l.overworld_path_target.y, l.castle_pad_surface_y);
    // The lower approach leaves from the Gate 14 landing platform.
    assert!(
        scene
            .inverted_route()
            .layout
            .platform_cells()
            .contains(&l.red_black_path_anchor)
    );
    assert!(scene.world().contains(l.red_black_path_anchor));
    assert!(
        l.red_black_fortress_pad
            .contains(l.red_black_path_target.x, l.red_black_path_target.z)
    );
    assert_eq!(l.red_black_path_target.y, l.fortress_pad_bottom_y);
    // The terrace runs from the lobe's seam to the pad's west edge.
    assert!(l.fortress_terrace_min_x < l.fortress_terrace_max_x);
    assert_eq!(l.fortress_terrace_max_x, l.red_black_fortress_pad.min_x - 1);
}

#[test]
fn the_pads_have_useful_footprints() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    assert_eq!(PAD_SIDE, 10);
    assert!(l.castle_pad_area() >= 100 && l.fortress_pad_area() >= 100);
    assert_eq!(
        l.overworld_castle_pad.max_x - l.overworld_castle_pad.min_x + 1,
        PAD_SIDE
    );
    assert_eq!(
        l.overworld_castle_pad.max_z - l.overworld_castle_pad.min_z + 1,
        PAD_SIDE
    );
    // Levels: the castle stands at the base terrain height; the fortress
    // hangs two cells below the shelf, the same underside level family
    // that the cutaway's ceiling already exposes.
    assert_eq!(l.castle_pad_surface_y, scene.config().base_height);
    assert_eq!(l.fortress_pad_bottom_y, scene.rhombus().shelf_y - 2);
    assert!(VOXEL_BUDGET_TARGET < VOXEL_BUDGET_SOFT_MAX);
}

#[test]
fn the_expansion_lives_in_the_one_world_next_to_the_current_mass() {
    let scene = WorldScene::new();
    let l = scene.expansion_layout();
    // Adjacent to the terrain footprint, never a detached island: the
    // first column of the lobe is the one right after the footprint's east
    // edge, and the lobe's rows lie inside the footprint's rows.
    assert_eq!(l.overworld_extension.min_x, scene.config().width);
    assert!(l.overworld_extension.min_z >= 0 && l.overworld_extension.max_z < scene.config().depth);
    // The world box ends where the lobe ends.
    let bounds = scene.world().bounds().unwrap();
    assert_eq!(bounds.max_exclusive.x, l.overworld_extension.max_x + 1);
    assert_eq!(bounds.max_exclusive.z, scene.config().depth);
}
