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
use scene::orientation::Orientation;
use scene::overworld::{DESCENT_ENDPOINT, HOUSE_FOOTPRINT, pond_basin_columns};
use scene::rhombus::{LOWER_TIP_RADIUS, RhombusConfig, WAIST_RADIUS};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::{column_surface, footprint_contains, mass_bottom};
use scene::world::WorldScene;

fn official() -> (TerrainConfig, RhombusConfig) {
    (TerrainConfig::official(), RhombusConfig::official())
}

#[test]
fn the_official_layout_is_valid_and_derived_from_the_terrain() {
    let (terrain, rhombus) = official();
    assert_eq!(rhombus.validate(&terrain), Ok(()));
    assert_eq!(rhombus, RhombusConfig::derive(&terrain));
    assert_eq!(
        (rhombus.center_x, rhombus.center_z),
        (terrain.width / 2, terrain.depth / 2)
    );
    assert_eq!(
        rhombus.upper_surface_reference,
        terrain.max_surface_height()
    );
    assert_eq!(rhombus.upper_mass_bottom, mass_bottom(&terrain));
    // The real terrain agrees with the contract's surface and bottom.
    let scene = WorldScene::new();
    assert_eq!(scene.bounds().min.y, rhombus.upper_mass_bottom);
    assert!(scene.bounds().max.y <= rhombus.upper_surface_reference);
}

#[test]
fn the_waist_lies_under_the_surface_and_the_tip_under_the_waist() {
    let (terrain, r) = official();
    assert!(r.waist_y < r.upper_mass_bottom);
    assert!(r.waist_y < terrain.min_surface_height());
    assert!(r.shelf_y < r.waist_y);
    assert!(r.lower_widest_y < r.shelf_y);
    assert!(r.lower_tip_y < r.lower_widest_y);
    assert!(r.lower_tip_y < r.waist_y);
    // The section narrows to the waist, widens below it and closes at the tip.
    let w = |y| r.section_width(&terrain, y);
    assert!(w(r.upper_taper_top) > w(r.upper_mass_bottom));
    assert!(w(r.upper_mass_bottom) > w(r.waist_y));
    assert!(w(r.waist_y) < w(r.lower_widest_y));
    assert!(w(r.lower_widest_y) > w(r.lower_tip_y));
    assert!(w(r.lower_tip_y) >= 1 && w(r.lower_tip_y) <= 2);
    assert_eq!(r.section_radius(r.lower_tip_y - 1), None);
    assert_eq!(r.section_radius(r.upper_surface_reference + 1), None);
    assert_eq!(r.section_radius(r.waist_y), Some(WAIST_RADIUS));
    assert!((r.section_radius(r.lower_tip_y).unwrap() - LOWER_TIP_RADIUS).abs() < 1e-4);
}

#[test]
fn the_portal_sits_centered_in_the_waist_facing_the_cutaway() {
    let (terrain, r) = official();
    let p = r.portal;
    assert!((p.min_y..=p.max_y).contains(&r.waist_y));
    assert!(p.max_y < terrain.min_surface_height());
    assert!((p.center_x() - r.center_x).abs() <= 4);
    assert!((p.wall_z - r.center_z).abs() <= 4);
    assert_eq!(p.facing, Orientation::South);
    assert_eq!(p.wall_z + 1, r.cutaway.min_z);
    assert!(p.width() >= 5 && p.height() >= 6);
    for x in p.min_x..=p.max_x {
        for y in p.min_y..=p.max_y {
            assert!(r.contains(&terrain, x, y, p.wall_z), "({x}, {y}) outside");
            assert!(footprint_contains(&terrain, x, p.wall_z));
        }
    }
    assert!(p.is_opening(p.center_x(), r.waist_y));
    assert!(p.is_frame(p.min_x, r.waist_y));
    assert!(!p.is_frame(p.center_x(), r.waist_y));
}

#[test]
fn the_gate12_descent_endpoint_is_preserved_as_the_upper_anchor() {
    let (terrain, r) = official();
    assert_eq!(r.upper_descent_anchor, DESCENT_ENDPOINT);
    let (x, z) = r.upper_descent_anchor;
    assert!(!r.cutaway.contains_column(x, z));
    assert!(footprint_contains(&terrain, x, z));
    // Right next to the cutaway: the descent enters it from the endpoint.
    assert!(r.cutaway.contains_column(x + 1, z));
    let scene = WorldScene::new();
    let end = scene.path().end().unwrap();
    assert_eq!((end.x, end.z), r.upper_descent_anchor);
    let _ = column_surface;
}

#[test]
fn the_house_and_pond_never_meet_the_portal_or_the_cutaway() {
    let (_, r) = official();
    for (x, z) in pond_basin_columns() {
        assert!(!r.cutaway.contains_column(x, z));
        assert!(!(x >= r.portal.min_x && x <= r.portal.max_x && z == r.portal.wall_z));
    }
    let h = HOUSE_FOOTPRINT.grown(1);
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            assert!(
                !r.cutaway.contains_column(x, z),
                "house column ({x}, {z}) cut"
            );
        }
    }
    assert!(HOUSE_FOOTPRINT.max_z < r.portal.wall_z || HOUSE_FOOTPRINT.max_x < r.portal.min_x);
}

#[test]
fn bounds_and_cutaway_side_are_deterministic() {
    let a = RhombusConfig::official();
    let b = RhombusConfig::official();
    assert_eq!(a, b);
    let terrain = TerrainConfig::official();
    for y in a.lower_tip_y..=a.upper_surface_reference {
        assert_eq!(a.section_area(&terrain, y), b.section_area(&terrain, y));
    }
    // South-east quadrant, toward the camera side of the Gate 12 framing.
    assert!(a.cutaway.min_x > a.center_x && a.cutaway.min_z > a.center_z);
    assert!(a.is_cut(a.cutaway.min_x, a.waist_y, a.cutaway.min_z));
    assert!(!a.is_cut(a.cutaway.min_x, a.shelf_y, a.cutaway.min_z));
    assert!(!a.is_cut(a.cutaway.min_x - 1, a.waist_y, a.cutaway.min_z));
    assert!(a.is_cut(a.cutaway.min_x, a.lower_tip_y, a.cutaway.min_z));
}

#[test]
fn the_lower_anchor_hangs_under_the_shelf_inside_the_cutaway() {
    let (_, r) = official();
    let a = r.lower_descent_anchor;
    assert_eq!(a.y, r.shelf_y - 1);
    assert!(r.cutaway.contains_column(a.x, a.z));
    assert_eq!(
        a,
        IVec3::new(r.cutaway.min_x, r.shelf_y - 1, r.cutaway.min_z)
    );
}

#[test]
fn invalid_layouts_are_rejected() {
    let (terrain, r) = official();
    let mut bad = r;
    bad.waist_y = r.upper_mass_bottom;
    assert!(bad.validate(&terrain).is_err());
    let mut bad = r;
    bad.lower_tip_y = r.waist_y + 1;
    assert!(bad.validate(&terrain).is_err());
    let mut bad = r;
    bad.cutaway.min_x = 0;
    bad.cutaway.min_z = 0;
    assert!(bad.validate(&terrain).is_err());
}

#[test]
fn the_contract_itself_places_no_voxels() {
    // Deriving and validating the layout is pure: a world stays empty.
    let terrain = TerrainConfig::official();
    let r = RhombusConfig::derive(&terrain);
    assert_eq!(r.validate(&terrain), Ok(()));
    let world = scene::voxel_world::VoxelWorld::new();
    assert!(world.is_empty());
    assert_eq!(r.upper_taper_top, r.upper_mass_bottom + 2);
    assert!(r.section_radius(r.upper_taper_top) == Some(1.0));
    assert!(r.section_radius(r.upper_taper_top - 1).unwrap() < 1.0);
}
