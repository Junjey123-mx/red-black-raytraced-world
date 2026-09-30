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
    CAMERA_COLLISION_RADIUS, CameraCollisionConfig, is_camera_solid, is_position_clear,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::cutaway::carve_cutaway;
use scene::descent::{InvertedRouteLayout, ROUTE_CLEARANCE_HEIGHT, build_upper_descent};
use scene::overworld::{build_house, carve_pond, furnish_house, lay_path, plant_trees};
use scene::portal::{build_portal_core, build_portal_frame};
use scene::red_black_maze::build_lower_mass;
use scene::rhombus::{RhombusConfig, build_upper_taper, connected_component};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::voxel_world::VoxelWorld;
use scene::world::WorldScene;

/// The world just before the route is built: everything up to the lower
/// mass, so what the route removes can be compared against real rock.
fn stage_before_route() -> (RhombusConfig, VoxelWorld, scene::red_black_maze::LowerMass) {
    let config = TerrainConfig::official();
    let mut world = VoxelWorld::new();
    generate_terrain(&config, &mut world);
    let pond = carve_pond(&config, &mut world);
    build_house(&config, &mut world);
    furnish_house(&config, &mut world);
    let trees = plant_trees(&config, &mut world, &pond);
    lay_path(&config, &mut world, &pond, &trees);
    let rhombus = RhombusConfig::derive(&config);
    build_upper_taper(&config, &rhombus, &mut world);
    carve_cutaway(&config, &rhombus, &mut world);
    build_upper_descent(&config, &rhombus, &mut world);
    let mut portal = build_portal_frame(&rhombus, &mut world);
    build_portal_core(&rhombus, &mut world, &mut portal);
    let lower = build_lower_mass(&config, &rhombus, &mut world);
    (rhombus, world, lower)
}

fn structural(t: BlockType) -> bool {
    matches!(
        t,
        BlockType::Deepslate | BlockType::SmoothBasalt | BlockType::PolishedBlackstoneBricks
    )
}

#[test]
fn the_old_lower_stairs_are_gone() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    // Gate 13 hung seven Down stairs on the west wall of the lower cutaway,
    // from (min_x - 1, shelf_y - 1, min_z) stepping south and down, over a
    // trench dug to the underside. None of that remains: no stair there
    // and the wall is solid mass again.
    let x = r.cutaway.min_x - 1;
    for k in 0..7 {
        let cell = IVec3::new(x, r.shelf_y - 1 - k, r.cutaway.min_z + k);
        assert_ne!(
            scene.world().get(cell).map(|b| b.block_type()),
            Some(BlockType::WoodStairs),
            "old stair still at {cell:?}"
        );
    }
    // The trench dug for that flight is mass again: the wall right under
    // the shelf is solid along the old route.
    for k in 0..3 {
        let cell = IVec3::new(x, r.shelf_y - 1 - k, r.cutaway.min_z + k);
        assert!(scene.world().contains(cell), "{cell:?} is still a trench");
    }
    // No Down stair anywhere in the lower cutaway wall column.
    for z in r.cutaway.min_z..r.cutaway.min_z + 8 {
        for y in r.lower_tip_y..r.shelf_y {
            if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                assert_ne!(
                    b.block_type(),
                    BlockType::WoodStairs,
                    "old stair at ({x}, {y}, {z})"
                );
            }
        }
    }
    assert!(
        !std::fs::read_to_string("src/scene/descent.rs")
            .unwrap()
            .contains("build_inverted_descent")
    );
}

#[test]
fn the_upper_stairs_are_unchanged() {
    let scene = WorldScene::new();
    let treads: Vec<IVec3> = scene
        .upper_descent()
        .treads
        .iter()
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(treads.len(), 12);
    assert_eq!(treads[0], IVec3::new(13, 3, 17));
    assert_eq!(treads[11], IVec3::new(15, -8, 18));
    for cell in &treads {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::WoodStairs)
        );
    }
    assert_eq!(scene.upper_descent().approach.len(), 6);
    assert_eq!(scene.upper_descent().footings, vec![IVec3::new(15, -9, 18)]);
}

#[test]
fn the_portal_is_unchanged() {
    let scene = WorldScene::new();
    let p = scene.portal();
    assert_eq!((p.frame.len(), p.core.len()), (18, 12));
    for cell in &p.frame {
        assert_eq!(cell.z, 14);
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::PortalFrameRedObsidian)
        );
    }
    for cell in &p.core {
        assert_eq!(cell.z, 14);
        assert!((15..=17).contains(&cell.x) && (-9..=-6).contains(&cell.y));
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::PortalCoreDarkCrimson)
        );
    }
    let route = scene.inverted_route();
    for cell in route.dug.iter().chain(route.treads.iter().map(|(c, _)| c)) {
        assert!(!p.frame.contains(cell) && !p.core.contains(cell));
    }
}

#[test]
fn the_route_starts_right_behind_the_aperture() {
    let scene = WorldScene::new();
    let p = scene.rhombus().portal;
    let layout = scene.inverted_route().layout;
    println!("GATE14 route layout: {layout:?}");
    assert_eq!(layout.x, p.center_x());
    assert_eq!(layout.portal_exit.z, p.wall_z - 1);
    assert!(p.is_opening(layout.portal_exit.x, layout.portal_exit.y));
    assert_eq!(
        (layout.exit_min_x, layout.exit_max_x),
        (p.min_x + 1, p.max_x - 1)
    );
    assert_eq!(
        (layout.exit_min_y, layout.exit_max_y),
        (p.min_y + 1, p.max_y - 1)
    );
    // The whole aperture, one cell behind the membrane, is air now.
    let clearance = layout.exit_clearance();
    assert_eq!(
        clearance.len(),
        (p.width() - 2) as usize * (p.height() - 2) as usize
    );
    for cell in &clearance {
        assert!(
            !scene.world().contains(*cell),
            "{cell:?} still blocks the exit"
        );
        assert!(scene.inverted_route().exit_clearance.contains(cell));
    }
    // The first tread waits one cell further north, at the top row.
    assert_eq!(
        layout.first_step,
        IVec3::new(layout.x, layout.exit_max_y, p.wall_z - 2)
    );
    assert_eq!(layout.direction, IVec3::new(0, 0, -1));
}

#[test]
fn the_route_points_at_the_red_black_underside() {
    let scene = WorldScene::new();
    let layout = scene.inverted_route().layout;
    assert!(layout.step_count >= 5, "{}", layout.step_count);
    assert_eq!(layout.landing, layout.tread(layout.step_count - 1));
    assert!(layout.landing.y < layout.first_step.y);
    assert!(layout.landing.z < layout.first_step.z);
    // Each tread is one cell lower (toward the inverted up, -Y) and one
    // further north.
    for k in 1..layout.step_count {
        let (a, b) = (layout.tread(k - 1), layout.tread(k));
        assert_eq!((b.x, b.y, b.z), (a.x, a.y - 1, a.z - 1));
    }
    // The landing sits at the rim: a surface cell of the underside lies
    // within one column and three rows.
    let l = layout.landing;
    assert!(
        scene
            .lower_surface()
            .cells
            .iter()
            .any(|c| (c.x - l.x).abs() <= 1 && (c.z - l.z).abs() <= 1 && (c.y - l.y).abs() <= 3)
    );
    // The surface connection is a real mass cell beside the landing.
    let s = layout.surface_connection;
    assert!(scene.world().contains(s), "{s:?}");
    assert!((s.x - l.x).abs() <= 1 && s.z == l.z);
    assert_eq!(layout.clearance_height, ROUTE_CLEARANCE_HEIGHT);
}

#[test]
fn the_clearance_is_inside_the_mass_and_the_world_box() {
    let (rhombus, world, lower) = stage_before_route();
    let layout = InvertedRouteLayout::derive(&rhombus, &lower, &world);
    let bounds = world.bounds().unwrap();
    for cell in layout.exit_clearance() {
        assert!(bounds.contains_cell(cell));
        assert!(
            world.contains(cell),
            "{cell:?} was not rock before the route"
        );
        assert!(!rhombus.portal.is_frame(cell.x, cell.y) || cell.z != rhombus.portal.wall_z);
    }
    for tread in layout.treads() {
        assert!(bounds.contains_cell(tread));
        assert!(world.contains(tread));
        for air in
            layout.clearance_under(layout.treads().iter().position(|t| *t == tread).unwrap() as i32)
        {
            assert!(bounds.contains_cell(air));
        }
    }
    let scene = WorldScene::new();
    assert_eq!(scene.inverted_route().layout, layout);
}

#[test]
fn no_focal_family_block_is_removed_for_the_clearance() {
    let (rhombus, world, lower) = stage_before_route();
    let layout = InvertedRouteLayout::derive(&rhombus, &lower, &world);
    for cell in layout.exit_clearance() {
        let t = world.get(cell).unwrap().block_type();
        assert!(structural(t), "{cell:?} is {t:?}");
    }
    // In the finished world every dug cell was structural mass too, and
    // every family symbol, crying obsidian and amethyst is intact.
    let scene = WorldScene::new();
    for family in scene.families() {
        for cell in family.symbols.iter().chain(family.accents.iter()) {
            assert!(
                scene.world().contains(*cell),
                "{cell:?} of {:?} removed",
                family.family
            );
        }
    }
    for cell in &scene.inverted_route().dug {
        assert!(!scene.families().iter().any(|f| f.cells().contains(cell)));
    }
}

#[test]
fn the_macro_mass_stays_one_connected_piece() {
    let scene = WorldScene::new();
    let main = connected_component(scene.world(), scene.silhouette().tip);
    assert_eq!(main.len(), scene.world().len());
    for cell in &scene.inverted_route().exit_clearance {
        assert!(!scene.world().contains(*cell));
    }
}

#[test]
fn the_planned_path_fits_the_collision_radius() {
    let scene = WorldScene::new();
    let layout = scene.inverted_route().layout;
    let config = CameraCollisionConfig::default();
    // The exit clearance: the camera is clear at every cell center of the
    // aperture rows, and a radius away from its floor and walls.
    for cell in layout.exit_clearance() {
        let c = Vec3::new(
            cell.x as f32 + 0.5,
            cell.y as f32 + 0.5,
            cell.z as f32 + 0.5,
        );
        assert!(
            is_position_clear(scene.world(), c, &config, &is_camera_solid),
            "{cell:?}"
        );
    }
    // Crossing the membrane at the middle row lands in clear air.
    let mid = layout.portal_exit;
    let behind = Vec3::new(mid.x as f32 + 0.5, mid.y as f32 + 0.5, mid.z as f32 + 0.5);
    assert!(is_position_clear(
        scene.world(),
        behind,
        &config,
        &is_camera_solid
    ));
    assert!(CAMERA_COLLISION_RADIUS * 2.0 < 1.0);
    // The clearance under each tread is tall enough for eye plus radius.
    assert!(layout.clearance_height as f32 >= 1.0 + 2.0 * CAMERA_COLLISION_RADIUS);
}
