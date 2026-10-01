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
use scene::castle::{GATE_WIDTH, OverworldCastleLayout, PASSAGE_HEIGHT, TOWER_SIDE, WALL_HEIGHT};
use scene::orientation::Orientation;
use scene::overworld::{ColumnRect, HOUSE_FOOTPRINT};
use scene::world::WorldScene;

fn inside(r: &ColumnRect, pad: &ColumnRect) -> bool {
    r.min_x >= pad.min_x && r.max_x <= pad.max_x && r.min_z >= pad.min_z && r.max_z <= pad.max_z
}

#[test]
fn the_castle_fits_the_pad_and_validates() {
    let scene = WorldScene::new();
    let e = scene.expansion_layout();
    let c = scene.castle_layout();
    assert_eq!(c.validate(e), Ok(()));
    assert_eq!(c.footprint, e.overworld_castle_pad);
    let pad = &c.footprint;
    assert!(inside(&c.keep, pad) && inside(&c.gatehouse, pad) && inside(&c.hall, pad));
    assert!(c.towers.iter().all(|t| inside(t, pad)));
    for (x, z) in c.courtyard.iter().chain(c.stair_well.iter()) {
        assert!(pad.contains(*x, *z));
    }
    for cell in c.windows.iter().chain(c.interior_route.iter().skip(2)) {
        assert!(pad.contains(cell.x, cell.z), "{cell:?} off the pad");
    }
    assert_eq!(c.levels.ground_y, e.castle_pad_surface_y);
    assert_eq!(c.levels.base_y, c.levels.ground_y + 1);
    assert_eq!(c.levels.wall_top_y, c.levels.ground_y + WALL_HEIGHT);
    assert!(c.levels.upper_floor_y > c.levels.base_y + 1);
    assert!(c.levels.keep_roof_y >= c.levels.upper_floor_y + 3);
    assert!(c.levels.tower_top_y > c.levels.keep_roof_y);
    assert!(c.levels.max_y <= 12, "{}", c.levels.max_y);
    println!(
        "GATE16 layout: pad {:?} keep {:?} hall {:?} gatehouse {:?} towers {:?}",
        c.footprint, c.keep, c.hall, c.gatehouse, c.towers
    );
}

#[test]
fn the_layout_is_the_same_from_the_pad_alone() {
    let scene = WorldScene::new();
    let derived = OverworldCastleLayout::derive(scene.expansion_layout());
    assert_eq!(&derived, scene.castle_layout());
}

#[test]
fn the_castle_does_not_block_the_approach() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let approach = scene.overworld_approach();
    for cell in approach.cells().iter().chain(approach.accents.iter()) {
        assert!(
            !c.footprint.contains(cell.x, cell.z),
            "{cell:?} under the castle"
        );
    }
    // The route starts on the paved approach and enters through the gate.
    let first = c.interior_route[0];
    assert!(
        approach
            .cells()
            .iter()
            .any(|f| f.x == first.x && f.z == first.z),
        "{first:?}"
    );
    let gate_cols = c.gate.columns();
    assert!(
        c.interior_route
            .iter()
            .any(|r| gate_cols.contains(&(r.x, r.z)))
    );
    // Past the forecourt the route lies on the pad, never on the path.
    for r in c.interior_route.iter().skip(2) {
        assert!(c.footprint.contains(r.x, r.z), "{r:?}");
    }
}

#[test]
fn the_castle_does_not_overlap_the_house_or_the_pond() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = c.footprint;
    let house = HOUSE_FOOTPRINT.grown(2);
    assert!(house.max_x < f.min_x, "house {house:?} vs castle {f:?}");
    for (x, z) in &scene.pond().basin {
        assert!(!f.contains(*x, *z));
    }
    for w in &scene.pond().water {
        assert!(!f.contains(w.x, w.z));
    }
    for t in scene.trees() {
        assert!(!f.contains(t.base.x, t.base.z));
    }
    // The portal and its cutaway are far to the south-west of the pad.
    let portal = scene.portal();
    for cell in portal.frame.iter().chain(portal.core.iter()) {
        assert!(!f.contains(cell.x, cell.z));
    }
}

#[test]
fn the_gate_faces_the_approach() {
    let scene = WorldScene::new();
    let e = scene.expansion_layout();
    let c = scene.castle_layout();
    assert_eq!(c.gate.facing, Orientation::West);
    assert_eq!(c.gate.base.x, c.footprint.min_x);
    assert_eq!(c.gate.width, GATE_WIDTH);
    let target = e.overworld_path_target;
    assert!(c.gate.columns().contains(&(target.x, target.z)));
    assert_eq!(c.gate.cells().len(), (GATE_WIDTH * PASSAGE_HEIGHT) as usize);
    assert_eq!(c.gate.base.y, c.levels.base_y);
    // The gatehouse wraps the gate with one wall row on each side.
    assert!(
        c.gatehouse.min_z == c.gate.base.z - 1 && c.gatehouse.max_z == c.gate.base.z + GATE_WIDTH
    );
    assert_eq!(c.gatehouse.min_x, c.footprint.min_x);
    let passage = c.gatehouse_passage();
    assert!(c.gate.columns().iter().all(|col| passage.contains(col)));
}

#[test]
fn rooms_have_usable_dimensions() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    assert!(c.hall.width() >= 3 && c.hall.depth() >= 3, "{:?}", c.hall);
    assert_eq!(c.hall.width(), c.keep.width() - 2);
    assert_eq!(c.hall.depth(), c.keep.depth() - 2);
    assert!(
        c.keep.width() * c.keep.depth() > HOUSE_FOOTPRINT.width() * HOUSE_FOOTPRINT.depth() - 10
    );
    assert!(c.courtyard.len() >= 8, "{}", c.courtyard.len());
    // The courtyard touches both the gatehouse passage and the keep door.
    let door = c.keep_door;
    assert!(c.courtyard.contains(&(door.base.x - 1, door.base.z)));
    assert!(
        c.courtyard
            .contains(&(c.gatehouse.max_x + 1, c.gate.base.z))
    );
    // Head room: the upper floor leaves two cells over the ground floor
    // and the roof two over the upper floor.
    assert!(c.levels.upper_floor_y - c.levels.ground_y >= 3);
    assert!(c.levels.keep_roof_y - c.levels.upper_floor_y >= 3);
    // The stair climbs one per tread and ends below the upper floor.
    assert_eq!(c.stair.len(), 2);
    assert_eq!(c.stair[0].0.y, c.levels.base_y);
    assert_eq!(c.stair[1].0.y, c.levels.base_y + 1);
    assert_eq!(c.stair[1].0.y + 1, c.levels.upper_floor_y);
    for (cell, o) in &c.stair {
        assert!(c.hall.contains(cell.x, cell.z));
        assert_eq!(*o, Orientation::North);
        assert!(c.stair_well.contains(&(cell.x, cell.z)));
    }
    assert!(c.interior_route.last().unwrap().y == c.levels.upper_floor_y);
}

#[test]
fn tower_bounds_are_valid() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let f = c.footprint;
    assert_eq!(c.towers.len(), 4);
    for t in &c.towers {
        assert_eq!((t.width(), t.depth()), (TOWER_SIDE, TOWER_SIDE));
        assert!(t.min_x == f.min_x || t.max_x == f.max_x);
        assert!(t.min_z == f.min_z || t.max_z == f.max_z);
        assert!(!t.contains(c.gate.base.x, c.gate.base.z));
        for (x, z) in &c.courtyard {
            assert!(!t.contains(*x, *z));
        }
    }
    let corners = [
        (f.min_x, f.min_z),
        (f.max_x, f.min_z),
        (f.min_x, f.max_z),
        (f.max_x, f.max_z),
    ];
    for (x, z) in corners {
        assert!(c.tower_at(x, z).is_some() && c.is_tower_shell(x, z));
    }
    assert!(c.is_curtain_column(f.min_x, c.gate.base.z));
    assert!(!c.is_curtain_column(f.min_x + 1, c.gate.base.z));
    assert!(
        c.windows.len() >= 12 && c.windows.len() <= 30,
        "{}",
        c.windows.len()
    );
    for w in &c.windows {
        let on_keep = c.is_keep_shell(w.x, w.z);
        let on_tower = c.is_tower_shell(w.x, w.z);
        assert!(on_keep || on_tower, "{w:?}");
        assert!(w.y > c.levels.base_y && w.y < c.levels.tower_top_y);
        assert!(!c.gate.cells().contains(w) && !c.keep_door.cells().contains(w));
    }
    let _: IVec3 = c.interior_route[0];
}
