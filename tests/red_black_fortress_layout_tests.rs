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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
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
use scene::fortress::{FORTRESS_TOWER_SIDE, FORTRESS_WALL_COURSES, RedBlackFortressLayout, above};
use scene::orientation::Orientation;
use scene::overworld::ColumnRect;
use scene::red_black_maze::Family;
use scene::world::WorldScene;

fn inside(r: &ColumnRect, pad: &ColumnRect) -> bool {
    r.min_x >= pad.min_x && r.max_x <= pad.max_x && r.min_z >= pad.min_z && r.max_z <= pad.max_z
}

fn focal(scene: &WorldScene) -> Vec<IVec3> {
    scene.families().iter().flat_map(|f| f.cells()).collect()
}

fn portal_route(scene: &WorldScene) -> Vec<IVec3> {
    let r = scene.inverted_route();
    r.treads
        .iter()
        .map(|(c, _)| *c)
        .chain(r.supports.iter().copied())
        .chain(r.layout.platform_cells())
        .chain(r.layout.exit_clearance())
        .collect()
}

#[test]
fn the_fortress_fits_the_pad_and_validates() {
    let scene = WorldScene::new();
    let e = scene.expansion_layout();
    let f = scene.fortress_layout();
    assert_eq!(f.validate(e, &focal(&scene), &portal_route(&scene)), Ok(()));
    assert_eq!(f.footprint, e.red_black_fortress_pad);
    let pad = &f.footprint;
    assert!(inside(&f.central_keep, pad) && inside(&f.gatehouse, pad) && inside(&f.hall, pad));
    assert!(f.towers().iter().all(|t| inside(&t.bounds, pad)));
    for (x, z) in f.courtyard.iter().chain(f.stair_well.iter()) {
        assert!(pad.contains(*x, *z));
    }
    for route in &f.interior_routes {
        for cell in route.iter().filter(|c| c.x >= pad.min_x) {
            assert!(pad.contains(cell.x, cell.z), "{cell:?} off the pad");
        }
    }
    assert_eq!(&RedBlackFortressLayout::derive(e), f);
    println!(
        "GATE17 layout: pad {:?} keep {:?} hall {:?} gate {:?} towers {:?}",
        f.footprint,
        f.central_keep,
        f.hall,
        f.gatehouse,
        f.towers()
            .iter()
            .map(|t| (t.family, t.bounds))
            .collect::<Vec<_>>()
    );
}

#[test]
fn every_level_counts_downward_from_the_pad() {
    let scene = WorldScene::new();
    let e = scene.expansion_layout();
    let lv = scene.fortress_layout().levels;
    assert_eq!(lv.ground_y, e.fortress_pad_bottom_y);
    assert_eq!(lv.base_y, lv.ground_y - 1);
    assert_eq!(lv.wall_top_y, lv.ground_y - FORTRESS_WALL_COURSES);
    assert!(
        lv.upper_floor_y < lv.base_y - 1,
        "two cells of head room under the hall"
    );
    assert!(lv.keep_roof_y <= lv.upper_floor_y - 3);
    assert!(lv.tower_top_y < lv.keep_roof_y);
    assert_eq!(lv.max_y, lv.tower_top_y - 1);
    assert!(
        lv.max_y > scene.rhombus().lower_tip_y,
        "{} vs tip {}",
        lv.max_y,
        scene.rhombus().lower_tip_y
    );
    assert_eq!(above(IVec3::new(0, lv.ground_y, 0), 1).y, lv.base_y);
    // The pad's -Y face is the ground: every pad column is level and Down.
    for z in e.red_black_fortress_pad.min_z..=e.red_black_fortress_pad.max_z {
        for x in e.red_black_fortress_pad.min_x..=e.red_black_fortress_pad.max_x {
            let b = scene.world().get(IVec3::new(x, lv.ground_y, z)).unwrap();
            assert_eq!(b.orientation(), Orientation::Down);
        }
    }
}

#[test]
fn the_gate_faces_the_lower_approach() {
    let scene = WorldScene::new();
    let e = scene.expansion_layout();
    let f = scene.fortress_layout();
    assert_eq!(f.gate.facing, Orientation::West);
    assert_eq!(f.gate.base.x, f.footprint.min_x);
    assert_eq!(f.gate.width, 2);
    assert!(
        f.gate
            .columns()
            .contains(&(e.red_black_path_target.x, e.red_black_path_target.z))
    );
    assert_eq!(f.gate.base.y, f.levels.base_y);
    assert!(
        f.gate
            .cells()
            .iter()
            .all(|c| c.y <= f.levels.base_y && c.y > f.levels.wall_top_y)
    );
    let last = *scene.red_black_approach().main.last().unwrap();
    assert_eq!(last.x + 1, f.gate.base.x);
    assert!(
        (f.gate.base.z..f.gate.base.z + f.gate.width).contains(&last.z)
            || (last.z - f.gate.base.z).abs() <= 1
    );
    let corridor = f.gate_corridor();
    assert!(f.gate.columns().iter().all(|c| corridor.contains(c)));
    assert_eq!(f.main_route()[0].x, e.red_black_path_target.x - 2);
}

#[test]
fn the_courtyard_is_usable() {
    let scene = WorldScene::new();
    let f = scene.fortress_layout();
    assert!(f.courtyard.len() >= 8, "{}", f.courtyard.len());
    assert!(
        f.courtyard
            .contains(&(f.gatehouse.max_x + 1, f.gate.base.z))
    );
    assert!(
        f.courtyard
            .contains(&(f.keep_door.base.x - 1, f.keep_door.base.z))
    );
    for t in f.towers() {
        let e = t.entrance.base;
        let outside = if t.entrance.facing == Orientation::East {
            e.x + 1
        } else {
            e.x - 1
        };
        assert!(
            f.courtyard.contains(&(outside, e.z)),
            "{:?} entrance",
            t.family
        );
    }
    // Open: no courtyard column is also a building column.
    for (x, z) in &f.courtyard {
        assert!(
            !f.is_curtain_column(*x, *z)
                && f.tower_at(*x, *z).is_none()
                && !f.central_keep.contains(*x, *z)
                && !f.gatehouse.contains(*x, *z)
        );
    }
}

#[test]
fn tower_bounds_are_valid_and_one_per_family() {
    let scene = WorldScene::new();
    let f = scene.fortress_layout();
    let pad = f.footprint;
    let families: Vec<Family> = f.towers().iter().map(|t| t.family).collect();
    assert_eq!(
        families,
        vec![Family::Crimson, Family::Orange, Family::Violet]
    );
    for t in f.towers() {
        assert_eq!(
            (t.bounds.width(), t.bounds.depth()),
            (FORTRESS_TOWER_SIDE, FORTRESS_TOWER_SIDE)
        );
        assert!(t.bounds.min_x == pad.min_x || t.bounds.max_x == pad.max_x);
        assert!(t.bounds.min_z == pad.min_z || t.bounds.max_z == pad.max_z);
        let (cx, cz) = t.core();
        assert!(t.bounds.contains(cx, cz) && !t.is_shell(cx, cz));
        assert!(t.is_shell(t.entrance.base.x, t.entrance.base.z));
        assert!(!t.bounds.contains(f.gate.base.x, f.gate.base.z));
        for (x, z) in &f.courtyard {
            assert!(!t.bounds.contains(*x, *z));
        }
    }
}

#[test]
fn keep_bounds_are_valid() {
    let scene = WorldScene::new();
    let f = scene.fortress_layout();
    assert!(f.hall.width() >= 3 && f.hall.depth() >= 3);
    assert_eq!(f.hall.width(), f.central_keep.width() - 2);
    assert_eq!(f.hall.depth(), f.central_keep.depth() - 2);
    assert_eq!(
        f.central_keep.max_x, f.footprint.max_x,
        "the keep's east wall is the curtain"
    );
    assert_eq!(f.keep_door.base.x, f.central_keep.min_x);
    assert!(f.hall.contains(f.keep_door.base.x + 1, f.keep_door.base.z));
    // The stair climbs north, one course per tread, into the well.
    assert_eq!(f.stair.len(), 2);
    assert_eq!(f.stair[0].0.y, f.levels.base_y);
    assert_eq!(f.stair[1].0.y, f.levels.base_y - 1);
    assert_eq!(f.stair[1].0.y - 1, f.levels.upper_floor_y);
    assert!(f.stair[1].0.z < f.stair[0].0.z);
    for (cell, o) in &f.stair {
        assert_eq!(*o, Orientation::Down);
        assert!(f.hall.contains(cell.x, cell.z));
        assert!(f.stair_well.contains(&(cell.x, cell.z)));
    }
    assert_eq!(f.main_route().last().unwrap().y, f.levels.upper_floor_y);
}

#[test]
fn the_fortress_overlaps_neither_the_portal_route_nor_the_focal_scenes() {
    let scene = WorldScene::new();
    let f = scene.fortress_layout();
    for cell in portal_route(&scene) {
        assert!(!f.footprint.contains(cell.x, cell.z), "{cell:?}");
    }
    for cell in focal(&scene) {
        assert!(!f.footprint.contains(cell.x, cell.z), "{cell:?}");
    }
    for cell in &scene.lower_surface().cells {
        assert!(!f.footprint.contains(cell.x, cell.z));
    }
    // The approach ends at the gate and never crosses the pad.
    for cell in &scene.red_black_approach().main {
        assert!(!f.footprint.contains(cell.x, cell.z));
    }
    // The Overworld castle stands on the other side of the world.
    let castle = scene.castle_layout();
    assert_eq!(
        castle.footprint, f.footprint,
        "both pads share their columns"
    );
    assert!(castle.levels.ground_y > f.levels.ground_y + 10);
}
